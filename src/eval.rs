use crate::context::Context;
use crate::env::Env;
use crate::value::{Value, Id};

// Takes an already syntactically checked value and reduces it.
pub fn eval(v: &Value, env: &Env, cx: &mut Context) -> Result<Value, Value> {
    unimplemented!()
    // eval_expanded(v, env, cx, true)
}

enum TCO {
    Ok(Value),
    Err(Value),
    TailCall {
        id: Id,
        arg: Value,
    }
}

// If `tail` and v is an application, this returns a TailCall, else it performs regular,
// recursive evaluation.
//
// Lambdas are evaluated in a loop, continuing to loop while their body `eval_tco`s to
// a TailCall.
fn eval_tco(v: &Value, tail: bool, env: &Env, cx: &mut Context) -> TCO {
    unimplemented!();
}
