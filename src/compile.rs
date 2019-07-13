use std::collections::HashMap;
use std::rc::Rc;

use im_rc::Vector as ImVector;

use crate::builtins;
use crate::check::{check_toplevel, BindingError};
use crate::gc_foreign::Vector;
use crate::special_forms::{Code, to_code, SpecialFormSyntaxError};
use crate::value::{Value, Id};
use crate::vm::{Closure, DeBruijn, BindingId, BBId, BB_RETURN, Instruction, IrChunk, Addr, Environment};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum StaticError {
    SpecialFormSyntax(SpecialFormSyntaxError),
    Binding(BindingError),
}

impl From<SpecialFormSyntaxError> for StaticError {
    fn from(err: SpecialFormSyntaxError) -> Self {
        StaticError::SpecialFormSyntax(err)
    }
}

impl From<BindingError> for StaticError {
    fn from(err: BindingError) -> Self {
        StaticError::Binding(err)
    }
}

use Instruction::*;

// A stack of lexical scopes, mapping identifiers to their numeric BindingIds.
struct Stack(Vec<HashMap<Id, BindingId>>);

impl Stack {
    // Create a new, empty Stack.
    fn new() -> Stack {
        Stack(vec![])
    }

    fn push_scope(&mut self) {
        self.0.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.0.pop();
    }

    fn add(&mut self, id: &Id) -> usize {
        let scopes_len = self.0.len();
        let scope = &mut self.0[scopes_len - 1];
        let len = scope.len();
        scope.insert(id.clone(), len);
        len
    }

    fn resolve(&self, id: &Id) -> DeBruijn {
        for (up, env) in self.0.iter().rev().enumerate() {
            if let Some(offset) = env.get(id) {
                return DeBruijn {
                    up,
                    id: *offset,
                };
            }
        }

        println!("{:?}", id);

        unreachable!("Always at least one environment, id can not be unused (caught by static checks)");
    }

    fn from_toplevel(toplevel: &HashMap<Id, (Value, bool)>) -> Stack {
        let mut ret = Stack::new();
        ret.push_scope();

        for (i, (name, _)) in toplevel.iter().enumerate() {
            ret.0[0].insert(name.clone(), i);
        }

        ret
    }
}

// BasicBlockBuilder, a helper for constructing the graph of basic blocks.
//
// It provides a stateful api. There's the `current` block on which to work, and methods to modify
// it.
struct BBB {
    // All basic blocks generated so far.
    blocks: Vec<Vec<Instruction>>,
    // Index of the currently active block.
    current: BBId,
    // Index of the block to which a trap instruction should jump.
    trap_handler: BBId,
}

impl BBB {
    fn new() -> BBB {
        BBB {
            blocks: vec![vec![]],
            current: 0,
            trap_handler: BB_RETURN,
        }
    }

    // Create a new, empty basic block, and return it's id.
    fn new_block(&mut self) -> BBId {
        self.blocks.push(vec![]);
        return self.blocks.len() - 1;
    }

    // Set the block on which the BBB operates.
    fn set_active_block(&mut self, bb: BBId) {
        self.current = bb;
    }

    // Append an instruction to the currently active block.
    fn append(&mut self, inst: Instruction) {
        self.blocks[self.current].push(inst);
    }

    fn push_nil(&mut self) {
        self.append(Literal(Value::nil()))
    }

    // Consume the builder to create an IrChunk.
    fn into_ir(self) -> IrChunk {
        IrChunk {
            basic_blocks: self.blocks,
        }
    }
}

pub fn compile(
    v: &Value,
    toplevel: &HashMap<Id, (Value, bool)>,
) -> Result<Closure, StaticError> {
    compile_code(to_code(v)?, toplevel)
}

pub fn compile_code(
    c: Code,
    toplevel: &HashMap<Id, (Value, bool)>,
) -> Result<Closure, StaticError> {
    check_toplevel(c.clone(), toplevel)?;

    let mut s = Stack::from_toplevel(toplevel);
    let chunk = Rc::new(compile_lambda(Vector(ImVector::new()), c, &mut s));

    return Ok(Closure {
        fun: chunk,
        env: Environment::child(Environment::from_toplevel(toplevel)),
        args: 0,
    });
}

fn code_to_ir(c: Code, push: bool, bbb: &mut BBB, tail: bool, s: &mut Stack) {
    match c {
        Code::Atomic(a) => {
            if push {
                bbb.append(Literal(Value::Atomic(a.clone())));
            }
        }

        Code::Fun(fun) => {
            if push {
                bbb.append(Literal(Value::Fun(fun.clone())));
            }
        }

        Code::Cell(cell, id) => {
            if push {
                bbb.append(Literal(Value::Cell(cell.clone(), id.clone())));
            }
        }

        Code::Opaque(o, id) => {
            if push {
                bbb.append(Literal(Value::Opaque(o.clone(), id.clone())));
            }
        }

        Code::Arr(inners) => {
            let len = inners.0.len();
            for inner in inners.0.iter() {
                code_to_ir(inner.clone(), push, bbb, false, s);
            }

            if push {
                bbb.append(Arr(len))
            }
        }

        Code::App(app) => {
            let len = app.0.len();

            if len == 0 {
                bbb.append(Literal(builtins::index_error()));
                bbb.append(Throw);
            } else {
                for inner in app.0.iter() {
                    code_to_ir(inner.clone(), true, bbb, false, s);
                }

                if tail {
                    bbb.append(TailCall(len - 1, push));
                } else {
                    bbb.append(Call(len - 1, push));
                }
            }
        }

        Code::Set(inners) => {
            let len = inners.0.len();
            for inner in inners.0.iter() {
                code_to_ir(inner.clone(), push, bbb, false, s);
            }

            if push {
                bbb.append(Set(len))
            }
        }

        Code::Map(entries) => {
            let len = entries.0.len();
            for (key, val) in entries.0.iter() {
                code_to_ir(key.clone(), push, bbb, false, s);
                code_to_ir(val.clone(), push, bbb, false, s);
            }

            if push {
                bbb.append(Map(len))
            }
        }

        Code::Id(id) => {
            let db = s.resolve(&id);
            bbb.append(Push(Addr::env(db)));
        }

        Code::Quote(q) => {
            if push {
                bbb.append(Literal(q));
            }
        }

        Code::Do(stmts) => {
            let len = stmts.0.len();

            if len == 0 {
                if push {
                    bbb.push_nil();
                }
            } else {
                let last = stmts.0[len - 1].clone();
                for stmt in stmts.0.into_iter().take(len - 1) {
                    code_to_ir(stmt, false, bbb, false, s);
                }
                code_to_ir(last, push, bbb, tail, s);
            }
        }

        Code::SetBang(id, rhs) => {
            code_to_ir(*rhs, true, bbb, false, s);

            let db = s.resolve(&id);
            bbb.append(Pop(Addr::env(db)));

            if push {
                bbb.push_nil();
            }
        }

        Code::If(cond, then, else_) => {
            let bb_then = bbb.new_block();
            let bb_else = bbb.new_block();
            let bb_cont = bbb.new_block();

            code_to_ir(*cond, true, bbb, false, s);
            bbb.append(CondJump(bb_then, bb_else));

            bbb.set_active_block(bb_then);
            code_to_ir(*then, push, bbb, tail, s);
            bbb.append(Jump(bb_cont));

            bbb.set_active_block(bb_else);
            code_to_ir(*else_, push, bbb, tail, s);
            bbb.append(Jump(bb_cont));

            bbb.set_active_block(bb_cont);
        }

        Code::Throw(exception) => {
            code_to_ir(*exception, true, bbb, false, s);
            bbb.append(Throw);
        }

        Code::Try(yay, _, binder, nay) => {
            let bb_catch = bbb.new_block();
            let bb_cont = bbb.new_block();

            let prev_trap_handler = bbb.trap_handler;
            bbb.trap_handler = bb_catch;
            bbb.append(SetCatchHandler(bb_catch));
            code_to_ir(*yay, push, bbb, false, s);
            bbb.trap_handler = prev_trap_handler;
            bbb.append(SetCatchHandler(prev_trap_handler));
            bbb.append(Jump(bb_cont));

            bbb.set_active_block(bb_catch);
            bbb.append(SetCatchHandler(prev_trap_handler));
            let db = DeBruijn { up: 0, id: s.add(&binder) };
            bbb.append(Pop(Addr::env(db)));
            code_to_ir(*nay, push, bbb, tail, s);
            bbb.append(Jump(bb_cont));

            bbb.set_active_block(bb_cont);
        }

        Code::Lambda(args, body) => {
            let len = args.0.len();
            let ir_chunk = Rc::new(compile_lambda(args, *body, s));
            bbb.append(FunLiteral(ir_chunk, len));
        }
    }
}

fn compile_lambda(args: Vector<(bool, Id)>, body: Code, s: &mut Stack) -> IrChunk {
    let mut bbb = BBB::new();
    s.push_scope();

    for (_, binder) in args.0.iter() {
        s.add(binder);
    }

    code_to_ir(body, true, &mut bbb, true, s);
    s.pop_scope();

    return bbb.into_ir();
}
