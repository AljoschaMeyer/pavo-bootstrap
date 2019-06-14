use std::collections::HashMap;
use std::rc::Rc;

use gc::{Gc, GcCell};
use gc_derive::{Trace, Finalize};
use im_rc::Vector as ImVector;

use crate::builtins::{num_args_error, type_error};
use crate::context::Context;
use crate::gc_foreign::Vector;
use crate::value::{Value, Fun, _Fun};

pub type BBId = usize;
pub type BindingId = usize;

// Indicates the path from a bound identifier site to its binder.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeBruijn {
    // How many environments we need to go up to find the binder.
    pub up: usize,
    // The numeric id of the binder within its environment.
    pub id: BindingId,
}

// Addresses a storage slot where a computation can write `Value`s to (or from where to read them).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Addr {
    Stack,
    Environment(DeBruijn),
}

impl Addr {
    pub fn env(id: DeBruijn) -> Addr {
        Addr::Environment(id)
    }
}

/// A single instruction of the ir.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Instruction {
    /// Push the value to the stack.
    Literal(Value),
    /// Pop the topmost usize values and push an array containing them in the reverse order.
    Arr(usize),
    /// Pop the topmost usize values and push an application containing them in the reverse order.
    App(usize),
    /// Pop the topmost usize values and push a set containing them.
    Set(usize),
    /// Pop the topmost usize * 2 values and push a map containing them. The stack values
    /// alternate between value and key.
    Map(usize),
    /// Create a closure value with the given IrChunk, push it to the stack.
    /// The BBId is the block at which to begin execution of the chunk.
    /// This can't be done via `Instruction::Literal` since the environmen must be set at runtime.
    FunLiteral(Rc<IrChunk>, BBId, Option<usize>),
    /// Jump to the given basic block. If the bb is `BB_RETURN`, return from the function instead.
    Jump(BBId),
    /// Pop the topmost stack element. Jump to the first basic block if the value was truthy,
    /// jump to the second block otherwise.
    CondJump(BBId, BBId),
    /// Jump to the current catch handler basic block. If the bb is `BB_RETURN`, the function throws.
    Throw,
    /// Set the catch hander basic block.
    SetCatchHandler(BBId),
    /// Push the value at the Addr to the stack.
    Push(Addr),
    /// Pop the stack and write the value to the Addr.
    Pop(Addr),
    /// Swap the position of the topmost two stack elements.
    Swap,
    /// Call the len'th stack element with the topmost len-1 arguments in reverse order. If the
    /// bool is true, push the result onto the stack.
    ///
    /// Example: Call(3, true) on the stack `<top> arg2 arg1 fun foo <bottom>`
    /// computes `(fun arg1 arg2)`
    Call(usize /*len*/, bool),
    /// Reuse the current stack for "calling" the closure at the DeBruijn address, entering at its
    /// entry (read from the closure at runtime, because I'm too lazy to implement this properly),
    /// by jumping there, with the usize topmost arguments in reverse order (just like Call,
    /// except the fun itself is not taken from the stack).
    TailCall(usize, DeBruijn),
    // TODO rework apply as a function that takes a single application as its arg
    /// Invoke the topmost value with the arguments in the array that is the second-to-topmost
    /// value. If the bool is true, push the result onto the stack.
    Apply(bool), // TODO get rid of this? depends on how apply will be implemented
}
use Instruction::*;

// If this is given as an unconditional jump address, return from the function instead.
pub const BB_RETURN: BBId = std::usize::MAX;

/// A control flow graph of basic blocks, each consisting of a sequence of statements.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IrChunk {
    // The ir instructions, as a graph of basic blocks.
    pub basic_blocks: Vec<Vec<Instruction>>,
}

// The local state upon which the instructions to operate. It is local to each invocation of
// `Computation::compute`.
#[derive(Debug)]
struct LocalState {
    // Index into the graph of instructions that indicates which instruction to execute next.
    // "pc" stands for "program counter".
    //
    // First usize is the basic block, second one the offset in the basic block.
    pc: (BBId, usize),
    // Temporary storage slots for `Value`s.
    stack: Vec<Value>,
    // Where to resume execution after something throws. If this is `BB_RETURN`, the function
    // itself throws rather than resuming execution.
    catch_handler: BBId,
}

impl LocalState {
    // Create and initialize a `LocalState` suitable for executing the given chunk.
    fn new(_chunk: &IrChunk, entry: BBId) -> LocalState {
        LocalState {
            pc: (entry, 0),
            stack: vec![],
            catch_handler: BB_RETURN,
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn args(&mut self, num: usize) -> Vector<Value> {
        let start = self.stack.len() - num;
        Vector(ImVector::from(&self.stack[start..]))
    }

    fn pop_n(&mut self, num: usize) {
        let new_len = self.stack.len() - num;
        self.stack.truncate(new_len);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Environment {
    // The bindings local to this environment.
    bindings: Vec<Value>,
    // (Mutable) access to the parent binding, which is `None` for the top-level environment.
    parent: Option<Gc<GcCell<Environment>>>,
}

impl Environment {
    // Look up the value addressed by the given DeBruijnPair. Panics if the address is invalid
    // (which only happens if compilation is buggy).
    pub fn get(&self, mut addr: DeBruijn) -> Value {
        if addr.up == 0 {
            self.bindings[addr.id].clone()
        } else {
            addr.up -= 1;
            self.parent.as_ref().unwrap().borrow().get(addr)
        }
    }

    // Set the value at the given address. Panics if the address is invalid (which only happens if
    // compilation is buggy).
    pub fn set(&mut self, mut addr: DeBruijn, val: Value) {
        if addr.up == 0 {
            if addr.id >= self.bindings.len()  {
                self.bindings.resize_with(addr.id + 1, Value::nil);
            }
            self.bindings[addr.id] = val;
        } else {
            addr.up -= 1;
            self.parent.as_ref().unwrap().borrow_mut().set(addr, val);
        }
    }

    pub fn child(parent: Gc<GcCell<Environment>>) -> Gc<GcCell<Environment>> {
        let env = Environment::root();
        env.borrow_mut().parent = Some(parent);
        env
    }

    pub fn root() -> Gc<GcCell<Environment>> {
        Gc::new(GcCell::new(Environment {
            bindings: vec![],
            parent: None,
        }))
    }

    pub fn from_toplevel(toplevel: &HashMap<String, Value>) -> Gc<GcCell<Environment>> {
        let ret = Environment::root();

        for (id, val) in toplevel.values().enumerate() {
            ret.borrow_mut().set(DeBruijn { up: 0, id}, val.clone());
        }

        ret
    }
}

// An IrChunk together with an environment. This is a runtime value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Closure {
    #[unsafe_ignore_trace]
    pub fun: Rc<IrChunk>,
    pub env: Gc<GcCell<Environment>>,
    // The basic block at which to begin execution of the `fun`.
    pub entry: usize,
    // How many arguments the closure takes, or `None` if it takes a variable number.
    pub args: Option<usize>,
}

impl Closure {
    // /// Create a closure suitable for executing the main body of a script.
    // ///
    // /// Behaves as if the script was the body of a zero-argument function defined in the lexical
    // /// scope of the (given) top-level environment.
    // pub fn from_script_chunk(script: IrChunk) -> Closure {
    //     Closure::from_chunk(Rc::new(script), Environment::child(top_level()), 0)
    // }

    fn from_chunk(fun: Rc<IrChunk>, env: Gc<GcCell<Environment>>, entry: usize, args: Option<usize>) -> Closure {
        Closure {
            fun,
            env,
            entry,
            args,
        }
    }
}

impl Addr {
    // Use an `Addr` to retrieve a value. This can not fail, unless we created erroneous ir code.
    fn load(self, local: &mut LocalState, env: &Gc<GcCell<Environment>>) -> Value {
        match self {
            Addr::Stack => local.stack.pop().unwrap(),
            Addr::Environment(de_bruijn) => env.borrow().get(de_bruijn),
        }
    }

    // Use an `Addr` to store a value. This can not fail, unless we created erroneous vm code.
    fn store(self, val: Value, local: &mut LocalState, env: &Gc<GcCell<Environment>>) {
        match self {
            Addr::Stack => local.stack.push(val),
            Addr::Environment(de_bruijn) => env.borrow_mut().set(de_bruijn, val),
        }
    }
}

impl Closure {
    // To perform the computation, interpret the instructions of the chunk.
    pub fn compute(&self, args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
        let mut state = LocalState::new(&self.fun, self.entry);

        match self.args {
            None => {
                Addr::env(DeBruijn { up: 0, id: 0 }).store(Value::arr(args), &mut state, &self.env);
            }
            Some(num_args) => {
                if args.0.len() != num_args {
                    return Err(num_args_error(num_args, args.0.len()));
                }

                for (i, arg) in args.0.iter().enumerate() {
                    Addr::env(DeBruijn { up: 0, id: i }).store(arg.clone(), &mut state, &self.env);
                }
            }
        }

        loop {
            state.pc.1 += 1;
            match &self.fun.basic_blocks[state.pc.0][state.pc.1 - 1] {
                Literal(val) => state.push(val.clone()),

                Arr(count) => {
                    let mut tmp = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        tmp.push(state.pop());
                    }
                    tmp.reverse();
                    state.push(Value::arr_from_vec(tmp));
                }

                App(count) => {
                    let mut tmp = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        tmp.push(state.pop());
                    }
                    tmp.reverse();
                    state.push(Value::app_from_vec(tmp));
                }

                Set(count) => {
                    let mut tmp = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        tmp.push(state.pop());
                    }
                    state.push(Value::set_from_vec(tmp));
                }

                Map(count) => {
                    let mut tmp = Vec::with_capacity(*count);
                    for _ in 0..(*count) {
                        let val = state.pop();
                        let key = state.pop();
                        tmp.push((key, val));
                    }
                    tmp.reverse();
                    state.push(Value::map_from_vec(tmp));
                }

                FunLiteral(chunk, entry, args) => state.push(Value::closure(
                    Closure::from_chunk(
                        chunk.clone(),
                        Environment::child(self.env.clone()),
                        *entry,
                        *args
                    ),
                    cx
                )),

                Jump(block) => {
                    if *block == BB_RETURN {
                        return Ok(state.pop());
                    } else {
                        state.pc = (*block, 0);
                    }
                }

                CondJump(then_block, else_block) => {
                    let val = state.pop();
                    if val.truthy() {
                        state.pc = (*then_block, 0);
                    } else {
                        state.pc = (*else_block, 0);
                    }
                }

                Throw => {
                    if state.catch_handler == BB_RETURN {
                        return Err(state.pop());
                    } else {
                        state.pc = (state.catch_handler, 0);
                    }
                }

                SetCatchHandler(bb) => state.catch_handler = *bb,

                Push(addr) => {
                    let val = addr.load(&mut state, &self.env);
                    state.push(val);
                }

                Pop(addr) => {
                    let val = state.pop();
                    addr.store(val, &mut state, &self.env);
                }

                Swap => {
                    let a = state.pop();
                    let b = state.pop();
                    state.push(a);
                    state.push(b);
                }

                Apply(push) => {
                    let fun = state.pop();
                    let args_val = state.pop();

                    match args_val.as_arr() {
                        None => {
                            let err = type_error(&args_val, "array");
                            if state.catch_handler == BB_RETURN {
                                return Err(err);
                            } else {
                                state.push(err);
                                state.pc = (state.catch_handler, 0);
                            }
                        }

                        Some(args) => {
                            match fun.compute(args.clone(), cx) {
                                Ok(val) => {
                                    if *push {
                                        state.push(val);
                                    }
                                }
                                Err(err) => {
                                    if state.catch_handler == BB_RETURN {
                                        return Err(err);
                                    } else {
                                        state.push(err);
                                        state.pc = (state.catch_handler, 0);
                                    }
                                }
                            }
                        }
                    }
                }

                Call(num_args, push) => {
                    let fun = state.pop();
                    let args = state.args(*num_args);

                    match fun.compute(args, cx) {
                        Ok(val) => {
                            state.pop_n(*num_args);
                            if *push {
                                state.push(val);
                            }
                        }
                        Err(err) => {
                            state.pop_n(*num_args);
                            if state.catch_handler == BB_RETURN {
                                return Err(err);
                            } else {
                                state.push(err);
                                state.pc = (state.catch_handler, 0);
                            }
                        }
                    }
                }

                TailCall(num_args, db) => {
                    let args = state.args(*num_args);

                    match self.args {
                        None => {
                            Addr::env(DeBruijn { up: 0, id: 0 }).store(Value::arr(args), &mut state, &self.env);
                        }
                        Some(num_args) => {
                            if args.0.len() != num_args {
                                return Err(num_args_error(num_args, args.0.len()));
                            }

                            for (i, arg) in args.0.iter().enumerate() {
                                Addr::env(DeBruijn { up: 0, id: i }).store(arg.clone(), &mut state, &self.env);
                            }
                        }
                    }
                    state.pop_n(*num_args);

                    let block = match &Addr::env(*db).load(&mut state, &self.env).as_fun() {
                        Some(Fun { fun: _Fun::Closure(c), .. }) => c.entry,
                        _ => unreachable!("TailCall DeBruijn must point to a closure"),
                    };

                    state.pc = (block, 0);
                }
            }
        }
    }
}
