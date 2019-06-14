use std::collections::{HashSet, HashMap};
use std::rc::Rc;

use im_rc::Vector as ImVector;

use crate::builtins;
use crate::check::{check_toplevel, StaticError};
use crate::context::Context;
use crate::gc_foreign::Vector;
use crate::special_forms::{SpecialForm, Args, SpecialFormSyntaxError, special};
use crate::value::{Value, Id, Builtin};
use crate::vm::{Closure, DeBruijn, BindingId, BBId, BB_RETURN, Instruction, IrChunk, Addr, Environment};

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

    fn add_dont_overwrite(&mut self, id: &Id) -> usize {
        let scopes_len = self.0.len();
        let scope = &mut self.0[scopes_len - 1];
        let len = scope.len();

        match scope.get(id) {
            Some(offset) => return *offset,
            None => {
                scope.insert(id.clone(), len);
                return len;
            }
        }
    }

    fn resolve(&self, id: &Id) -> DeBruijn {
        let num_envs = self.0.len();
        let mut env_level = num_envs - 1;

        for (up, env) in self.0.iter().rev().enumerate() {
            if let Some(offset) = env.get(id) {
                return DeBruijn {
                    up,
                    id: *offset,
                };
            }
        }

        unreachable!("Always at least one environment, id can not be unused (caught by static checks)");
    }

    fn from_toplevel(toplevel: &HashMap<String, Value>) -> Stack {
        let mut ret = Stack::new();
        ret.push_scope();

        for (i, (name, _)) in toplevel.iter().enumerate() {
            ret.0[0].insert(Id::user(name), i);
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
    // Index of the block to which a `break` statement should jump.
    // This has nothing to do with an *actual breakpoint*, but you can't stop me!
    breakpoint: BBId,
    // Index of the block to which a trap instruction should jump.
    trap_handler: BBId,
}

impl BBB {
    fn new() -> BBB {
        BBB {
            blocks: vec![vec![]],
            current: 0,
            breakpoint: BB_RETURN,
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

#[derive(Debug)]
struct Tails(HashSet<DeBruijn>);

impl Tails {
    fn is_tco(&self, fun: DeBruijn) -> bool {
        self.0.contains(&fun)
    }

    fn invalidate(&mut self, fun: DeBruijn) {
        self.0.remove(&fun);
    }

    fn insert(&mut self, fun: DeBruijn) {
        self.0.insert(fun);
    }

    fn empty() -> Tails {
        Tails(HashSet::new())
    }
}

pub fn compile<'a>(
    v: &Value,
    toplevel: &HashMap<String, Value>,
) -> Result<Closure, StaticError> {
    check_toplevel(v, toplevel)?;

    let mut s = Stack::from_toplevel(toplevel);
    let chunk = Rc::new(compile_lambda(&Args::Destructured(vec![]), v, &mut s));

    return Ok(Closure {
        fun: chunk,
        env: Environment::from_toplevel(toplevel),
        entry: 0,
        args: Some(0),
    });
}

fn val_to_ir(v: &Value, push: bool, bbb: &mut BBB, tails: &mut Tails, tail: bool, s: &mut Stack) {
    match v {
        Value::Atomic(..) | Value::Fun(..) => {
            if push {
                bbb.append(Literal(v.clone()));
            }
        }

        Value::Arr(inners) => {
            for inner in inners.0.iter() {
                val_to_ir(inner, push, bbb, tails, false, s);
            }

            if push {
                bbb.append(Arr(inners.0.len()))
            }
        }

        Value::Set(inners) => {
            for inner in inners.0.iter() {
                val_to_ir(inner, push, bbb, tails, false, s);
            }

            if push {
                bbb.append(Set(inners.0.len()))
            }
        }

        Value::Map(entries) => {
            for (key, val) in entries.0.iter() {
                val_to_ir(key, push, bbb, tails, false, s);
                val_to_ir(val, push, bbb, tails, false, s);
            }

            if push {
                bbb.append(Map(entries.0.len()))
            }
        }

        Value::Id(id) => {
            let db = s.resolve(id);
            bbb.append(Push(Addr::env(db)));
        }

        Value::App(app) => {
            match special(app) {
                Err(_) => unreachable!("static checks already discovered this"),

                // ordinary function application
                Ok(None) => {
                    if app.0.len() == 0 {
                        bbb.append(Literal(builtins::index_error(0)));
                        bbb.append(Throw);
                    } else {
                        for inner in app.0.iter() {
                            val_to_ir(inner, true, bbb, tails, false, s);
                        }

                        if tail {
                            match app.0[0].as_id() {
                                Some(id) => {
                                    let db = s.resolve(id);
                                    if tails.is_tco(db) {
                                        bbb.append(TailCall(app.0.len() - 1, db));
                                    } else {
                                        bbb.append(Call(app.0.len(), push));
                                    }
                                }

                                None => bbb.append(Call(app.0.len(), push)),
                            }
                        } else {
                            bbb.append(Call(app.0.len(), push));
                        }
                    }
                }

                Ok(Some(SpecialForm::Quote(quotation))) => {
                    bbb.append(Literal(quotation.clone()));
                }

                Ok(Some(SpecialForm::Do(stmts))) => {
                    if stmts.len() == 0 {
                        if push {
                            bbb.push_nil();
                        }
                    } else {
                        for stmt in stmts.iter().take(stmts.len() - 1) {
                            val_to_ir(stmt, false, bbb, tails, false, s);
                        }
                        val_to_ir(stmts[stmts.len() - 1], push, bbb, tails, tail, s);
                    }
                }

                Ok(Some(SpecialForm::SetBang(id, rhs))) => {
                    val_to_ir(rhs, true, bbb, tails, false, s);

                    let db = s.resolve(id);
                    bbb.append(Pop(Addr::env(db)));

                    if push {
                        bbb.push_nil();
                    }
                }

                Ok(Some(SpecialForm::If(cond, then, else_))) => {
                    let bb_then = bbb.new_block();
                    let bb_else = bbb.new_block();

                    val_to_ir(cond, true, bbb, tails, false, s);
                    bbb.append(CondJump(bb_then, bb_else));

                    bbb.set_active_block(bb_then);
                    val_to_ir(then, push, bbb, tails, tail, s);

                    bbb.set_active_block(bb_else);
                    val_to_ir(else_, push, bbb, tails, tail, s);
                }

                Ok(Some(SpecialForm::Throw(exception))) => {
                    val_to_ir(exception, true, bbb, tails, false, s);
                    bbb.append(Throw);
                }

                Ok(Some(SpecialForm::Try(yay, _, binder, nay))) => {
                    let bb_try = bbb.new_block();
                    let bb_catch = bbb.new_block();

                    bbb.append(Jump(bb_try));
                    let prev_trap_handler = bbb.trap_handler;
                    bbb.set_active_block(bb_try);
                    bbb.trap_handler = bb_catch;
                    bbb.append(SetCatchHandler(bbb.trap_handler));
                    val_to_ir(yay, push, bbb, tails, false, s);
                    bbb.trap_handler = prev_trap_handler;
                    bbb.append(SetCatchHandler(bbb.trap_handler));

                    bbb.set_active_block(bb_catch);
                    bbb.append(SetCatchHandler(bbb.trap_handler));
                    let db = DeBruijn { up: 0, id: s.add(binder) };
                    tails.invalidate(db);
                    bbb.append(Pop(Addr::env(db)));
                    val_to_ir(nay, push, bbb, tails, tail, s);
                }

                Ok(Some(SpecialForm::Lambda(args, body))) => {
                    let ir_chunk = Rc::new(compile_lambda(&args, body, s));
                    bbb.append(FunLiteral(ir_chunk, 0, match args {
                        Args::All(..) => None,
                        Args::Destructured(all) => Some(all.len()),
                    }));
                }

                Ok(Some(SpecialForm::LetFn(defs, cont))) => {
                    let (ir_chunk, entries) = compile_letfn(&defs, s);
                    let ir_chunk = Rc::new(ir_chunk);

                    for (i, (name, args, _)) in defs.iter().enumerate() {
                        let db = DeBruijn { up: 0, id: s.add(name) };
                        tails.invalidate(db);
                        bbb.append(FunLiteral(ir_chunk.clone(), entries[i], match args {
                            Args::All(..) => None,
                            Args::Destructured(all) => Some(all.len()),
                        }));
                        bbb.append(Pop(Addr::env(db)));
                    }

                    val_to_ir(cont, push, bbb, tails, tail, s);
                }
            }
        }
    }
}

fn compile_lambda(args: &Args, body: &Value, s: &mut Stack) -> IrChunk {
    let mut bbb = BBB::new();
    s.push_scope();

    match args {
        Args::All(_, binder) => {
            s.add(binder);
        }
        Args::Destructured(ids) => {
            for (_, binder) in ids {
                s.add(binder);
            }
        }
    }

    val_to_ir(body, true, &mut bbb, &mut Tails::empty(), true, s);
    s.pop_scope();

    return bbb.into_ir();
}

fn compile_letfn(
    defs: &Vec<(&Id, Args, &Value)>,
    s: &mut Stack,
) -> (IrChunk, Vec<BBId>) {
    let names: Vec<&Id> = defs.iter().map(|(name, _, _)| name.clone()).collect();
    let mut bbb = BBB::new();
    let mut bb_ids = Vec::with_capacity(names.len());

    for (_, args, body) in defs {
        s.push_scope();

        match args {
            Args::All(_, binder) => {
                s.add(binder);
            }
            Args::Destructured(ids) => {
                for (_, binder) in ids {
                    s.add(binder);
                }
            }
        }

        let mut tails = Tails::empty();
        for id in names.iter() {
            let offset = s.add_dont_overwrite(id);
            tails.insert(DeBruijn { up: 0, id: offset});
        }

        match args {
            Args::All(..) => {
                tails.invalidate(DeBruijn { up: 0, id: 0 });
            }
            Args::Destructured(ids) => {
                for (i, _) in ids.iter().enumerate() {
                    tails.invalidate(DeBruijn { up: 0, id: i });
                }
            }
        }

        let start_block = bbb.new_block();
        bb_ids.push(start_block);
        bbb.set_active_block(start_block);
        val_to_ir(body, true, &mut bbb, &mut tails, true, s);
        s.pop_scope();
    }

    return (bbb.into_ir(), bb_ids);
}
