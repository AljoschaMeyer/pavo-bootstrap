use std::collections::HashMap;

use gc::{Gc, GcCell};
use gc_derive::{Trace, Finalize};
use im_rc::Vector as ImVector;

use crate::builtins::{num_args_error, type_error};
use crate::context::Context;
use crate::gc_foreign::{Vector, OrdMap, OrdSet};
use crate::value::{Value, Fun, Id, Atomic};

pub type BBId = usize;
pub type BindingId = usize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum CompiledPattern {
    Atomic(Atomic),
    Name(DeBruijn),
    Arr(Vector<CompiledPattern>),
    App(Vector<CompiledPattern>),
    Set(OrdSet<Value>),
    Map(OrdMap<Value, CompiledPattern>),
    Named(DeBruijn, Box<CompiledPattern>),
}

// Indicates the path from a bound identifier site to its binder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Trace, Finalize)]
pub struct DeBruijn {
    // How many environments we need to go up to find the binder.
    pub up: usize,
    // The numeric id of the binder within its environment.
    pub id: BindingId,
}

// Addresses a storage slot where a computation can write `Value`s to (or from where to read them).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
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
    /// This can't be done via `Instruction::Literal` since the environmen must be set at runtime.
    FunLiteral(Gc<IrChunk>, usize),
    /// Jump to the given basic block. If the bb is `BB_RETURN`, return from the function instead.
    Jump(BBId),
    /// Pop the topmost stack element. Jump to the first basic block if the value was truthy,
    /// jump to the second block otherwise.
    CondJump(BBId, BBId),
    /// Pop the topmost stack element. Jump to the first basic block if the value matches the
    /// pattern (bringing bindings into scope), jump to the second block otherwise.
    Match(CompiledPattern, BBId, BBId),
    /// Jump to the current catch handler basic block. If the bb is `BB_RETURN`, the function throws.
    Throw,
    /// Set the catch hander basic block.
    SetCatchHandler(BBId),
    /// Push the value at the Addr to the stack.
    Push(Addr),
    /// Pop the stack and write the value to the Addr.
    Pop(Addr),
    /// Double the topmost stack element.
    DoubleTop,
    /// Remove the topmost stack element.
    DropTop,
    /// Call the len+1'th stack element with the topmost len arguments in reverse order. If the
    /// bool is true, push the result onto the stack.
    ///
    /// Example: Call(2, true) on the stack `<top> arg2 arg1 fun foo <bottom>`
    /// computes `(fun arg1 arg2)` and results in the stack `<top> (fun arg1 arg2) foo <bottom>`
    Call(usize /*len*/, bool),
    /// Same as `Call`, but performs tco.
    TailCall(usize /*len*/, bool),
    /// Create a new scope in the environment.
    PushScope,
    /// Pop the newest scope in the environment.
    PopScope,
}
use Instruction::*;

// If this is given as an unconditional jump address, return from the function instead.
pub const BB_RETURN: BBId = std::usize::MAX;

/// A control flow graph of basic blocks, each consisting of a sequence of statements.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
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
    // The environment in which to resume execution after something throws.
    catch_env: Gc<GcCell<Environment>>,
}

impl LocalState {
    // Create and initialize a `LocalState` suitable for executing the given chunk.
    fn new(_chunk: &IrChunk, catch_env: Gc<GcCell<Environment>>) -> LocalState {
        LocalState {
            pc: (0, 0),
            stack: vec![],
            catch_handler: BB_RETURN,
            catch_env,
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn args(&mut self, num: usize) -> Vector<Value> {
        let mut vector = ImVector::new();
        for _ in 0..num {
            vector.push_front(self.pop());
        }
        Vector(vector)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Environment {
    // The bindings local to this environment.
    bindings: Vec<Value>,
    // (Mutable) access to the parent binding, which is `None` for the top-level environment.
    parent: Option<Gc<GcCell<Environment>>>,
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Env {{ x: {}, y: {:?} }}", self.bindings.len(), self.parent)
    }
}

impl Environment {
    // Look up the value addressed by the given DeBruijnPair. Panics if the address is invalid
    // (which only happens if compilation is buggy).
    pub fn get(&self, addr: &DeBruijn) -> Value {
        if addr.up == 0 {
            self.bindings[addr.id].clone()
        } else {
            let mut addr = addr.clone();
            addr.up -= 1;
            self.parent.as_ref().unwrap().borrow().get(&addr)
        }
    }

    // Set the value at the given address. Panics if the address is invalid (which only happens if
    // compilation is buggy).
    pub fn set(&mut self, addr: &DeBruijn, val: Value) {
        if addr.up == 0 {
            if addr.id >= self.bindings.len()  {
                self.bindings.resize_with(addr.id + 1, Value::nil);
            }
            self.bindings[addr.id] = val;
        } else {
            let mut addr = addr.clone();
            addr.up -= 1;
            self.parent.as_ref().unwrap().borrow_mut().set(&addr, val);
        }
    }

    pub fn child(parent: Gc<GcCell<Environment>>) -> Gc<GcCell<Environment>> {
        let env = Environment::root();
        env.borrow_mut().parent = Some(parent);
        env
    }

    pub fn pop(env: &Gc<GcCell<Environment>>) -> Gc<GcCell<Environment>> {
        match env.borrow().parent {
            Some(ref parent) => parent.clone(),
            None => panic!("Tried to pop the toplevel environment -> buggy compilation"),
        }
    }

    pub fn root() -> Gc<GcCell<Environment>> {
        Gc::new(GcCell::new(Environment {
            bindings: vec![],
            parent: None,
        }))
    }

    pub fn from_toplevel(toplevel: &HashMap<Id, (Value, bool)>) -> Gc<GcCell<Environment>> {
        let ret = Environment::root();

        for (id, (val, _)) in toplevel.values().enumerate() {
            ret.borrow_mut().set(&DeBruijn { up: 0, id}, val.clone());
        }

        ret
    }
}

// An IrChunk together with an environment. This is a runtime value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub struct Closure {
    pub fun: Gc<IrChunk>,
    pub env: Gc<GcCell<Environment>>,
    pub args: usize,
}

impl Closure {
    fn from_chunk(fun: Gc<IrChunk>, env: Gc<GcCell<Environment>>, args: usize) -> Closure {
        Closure {
            fun,
            env,
            args,
        }
    }
}

impl std::fmt::Debug for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Closure {{ args: {:?}, ir: {:?} }}", self.args, self.fun)
    }
}

impl Addr {
    // Use an `Addr` to retrieve a value. This can not fail, unless we created erroneous ir code.
    fn load(&self, local: &mut LocalState, env: &Gc<GcCell<Environment>>) -> Value {
        match self {
            Addr::Stack => local.stack.pop().unwrap(),
            Addr::Environment(de_bruijn) => env.borrow().get(de_bruijn),
        }
    }

    // Use an `Addr` to store a value. This can not fail, unless we created erroneous vm code.
    fn store(&self, val: Value, local: &mut LocalState, env: &Gc<GcCell<Environment>>) {
        match self {
            Addr::Stack => local.stack.push(val),
            Addr::Environment(de_bruijn) => env.borrow_mut().set(de_bruijn, val),
        }
    }
}

impl Closure {
    // To perform the computation, interpret the instructions of the chunk.
    pub fn compute(&self, args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
        do_compute(self.clone(), args, cx)
    }
}

fn do_compute(mut c: Closure, mut args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    let mut state;

    loop {
        state = LocalState::new(&c.fun, c.env.clone());

        if args.0.len() != c.args {
            return Err(num_args_error());
        }

        for (i, arg) in args.0.iter().enumerate() {
            Addr::env(DeBruijn { up: 0, id: i }).store(arg.clone(), &mut state, &c.env);
        }

        loop {
            state.pc.1 += 1;
            match &c.fun.basic_blocks[state.pc.0].get(state.pc.1 - 1) {
                None => return Ok(state.pop()),

                Some(Literal(val)) => state.push(val.clone()),

                Some(Arr(count)) => {
                    let mut tmp = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        tmp.push(state.pop());
                    }
                    tmp.reverse();
                    state.push(Value::arr_from_vec(tmp));
                }

                Some(App(count)) => {
                    let mut tmp = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        tmp.push(state.pop());
                    }
                    tmp.reverse();
                    state.push(Value::app_from_vec(tmp));
                }

                Some(Set(count)) => {
                    let mut tmp = Vec::with_capacity(*count);
                    for _ in 0..*count {
                        tmp.push(state.pop());
                    }
                    state.push(Value::set_from_vec(tmp));
                }

                Some(Map(count)) => {
                    let mut tmp = Vec::with_capacity(*count);
                    for _ in 0..(*count) {
                        let val = state.pop();
                        let key = state.pop();
                        tmp.push((key, val));
                    }
                    tmp.reverse();
                    state.push(Value::map_from_vec(tmp));
                }

                Some(FunLiteral(chunk, args)) => state.push(Value::closure(
                    Closure::from_chunk(
                        chunk.clone(),
                        Environment::child(c.env.clone()),
                        *args
                    ),
                    cx
                )),

                Some(Jump(block)) => {
                    if *block == BB_RETURN {
                        return Ok(state.pop());
                    } else {
                        state.pc = (*block, 0);
                    }
                }

                Some(CondJump(then_block, else_block)) => {
                    let val = state.pop();
                    if val.truthy() {
                        state.pc = (*then_block, 0);
                    } else {
                        state.pc = (*else_block, 0);
                    }
                }

                Some(Match(p, yay, nay)) => {
                    if handle_match(state.pop(), p, &mut state, &c.env) {
                        state.pc = (*yay, 0);
                    } else {
                        state.pc = (*nay, 0);
                    }
                }

                Some(Throw) => {
                    if state.catch_handler == BB_RETURN {
                        c.env = state.catch_env.clone();
                        return Err(state.pop());
                    } else {
                        state.pc = (state.catch_handler, 0);
                        c.env = state.catch_env.clone();
                    }
                }

                Some(SetCatchHandler(bb)) => {
                    state.catch_handler = *bb;
                    state.catch_env = c.env.clone();
                }

                Some(Push(addr)) => {
                    let val = addr.load(&mut state, &c.env);
                    state.push(val);
                }

                Some(Pop(addr)) => {
                    let val = state.pop();
                    addr.store(val, &mut state, &c.env);
                }

                Some(DoubleTop) => {
                    let val = state.pop();
                    state.push(val.clone());
                    state.push(val);
                }

                Some(DropTop) => {
                    let _ = state.pop();
                }

                Some(Call(num_args, push)) => {
                    let args = state.args(*num_args);
                    let fun = state.pop();

                    match fun.compute(args, cx) {
                        Ok(val) => {
                            if *push {
                                state.push(val);
                            }
                        }
                        Err(err) => {
                            if state.catch_handler == BB_RETURN {
                                c.env = state.catch_env.clone();
                                return Err(err);
                            } else {
                                c.env = state.catch_env.clone();
                                state.push(err);
                                state.pc = (state.catch_handler, 0);
                            }
                        }
                    }
                }

                Some(TailCall(num_args, push)) => {
                    let new_args = state.args(*num_args);
                    let fun = state.pop();

                    match &fun {
                        Value::Fun(Fun::Closure(new_c, _)) => {
                            c = new_c.clone();
                            args = new_args;
                            break;
                        }

                        Value::Fun(Fun::Builtin(..)) => {
                            match fun.compute(new_args, cx) {
                                Ok(val) => {
                                    if *push {
                                        state.push(val);
                                    }
                                }
                                Err(err) => {
                                    if state.catch_handler == BB_RETURN {
                                        c.env = state.catch_env.clone();
                                        return Err(err);
                                    } else {
                                        c.env = state.catch_env.clone();
                                        state.push(err);
                                        state.pc = (state.catch_handler, 0);
                                    }
                                }
                            }
                        }

                        _ => {
                            let err = type_error();
                            if state.catch_handler == BB_RETURN {
                                c.env = state.catch_env.clone();
                                return Err(err);
                            } else {
                                c.env = state.catch_env.clone();
                                state.push(err);
                                state.pc = (state.catch_handler, 0);
                            }
                        }
                    }
                }

                Some(PushScope) => {
                    c.env = Environment::child(c.env.clone());
                }

                Some(PopScope) => {
                    c.env = Environment::pop(&c.env)
                }
            }
        }
    }

}

fn handle_match(
    val: Value,
    p: &CompiledPattern,
    state: &mut LocalState,
    env: &Gc<GcCell<Environment>>
) -> bool {
    match p {
        CompiledPattern::Name(db) => {
            Addr::env(db.clone()).store(val, state, env);
            return true;
        }

        CompiledPattern::Atomic(a) => match val.as_atomic() {
            Some(a2) if a2 == a => return true,
            _ => return false,
        }

        CompiledPattern::Set(set) => match val.as_set() {
            Some(set2) if set2 == set => return true,
            _ => return false,
        }

        CompiledPattern::Arr(ps) => match val.as_arr() {
            Some(arr) if arr.0.len() == ps.0.len() => {
                return ps.0.iter().zip(arr.0.iter()).all(|(p_, v_)|
                    handle_match(v_.clone(), p_, state, env)
                );
            }
            _ => return false,
        }

        CompiledPattern::App(ps) => match val.as_app() {
            Some(app) if app.0.len() == ps.0.len() => {
                return ps.0.iter().zip(app.0.iter()).all(|(p_, v_)|
                    handle_match(v_.clone(), p_, state, env)
                );
            }
            _ => return false,
        }

        CompiledPattern::Map(pattern) => match val.as_map() {
            Some(map) => {
                for (key, p_) in pattern.0.iter() {
                    match map.0.get(key) {
                        None => return false,
                        Some(val) => {
                            if !handle_match(val.clone(), p_, state, env) {
                                return false;
                            }
                        }
                    }
                }
                return true;
            }
            None => return false,
        }

        CompiledPattern::Named(db, inner) => {
            Addr::env(db.clone()).store(val.clone(), state, env);
            return handle_match(val, inner, state, env);
        }
    }
}
