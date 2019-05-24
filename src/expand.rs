use im_rc::OrdMap;

use crate::context::Context;
use crate::env::Env;
use crate::value::{Value, Id, Fun, _Fun, Builtin, Closure};

pub fn expand(v: Value, env: Env, macros: OrdMap<Id, Value>, cx: &mut Context) -> Result<Value, Value> {
    match v {
        Value::Atomic(..) | Value::Id(..) | Value::Fun(..)  => Ok(v),

        Value::Arr(ref vals) => {
            let mut evaluated = Vec::with_capacity(vals.0.len());
            for item in vals.0.iter() {
                evaluated.push(expand(item.clone(), env.clone(), macros.clone(), cx)?);
            }
            return Ok(Value::arr_from_vec(evaluated));
        }

        Value::Set(ref vals) => {
            let mut evaluated = Vec::with_capacity(vals.0.len());
            for item in vals.0.iter() {
                evaluated.push(expand(item.clone(), env.clone(), macros.clone(), cx)?);
            }
            return Ok(Value::set_from_vec(evaluated));
        }

        Value::Map(ref vals) => {
            let mut evaluated = Vec::with_capacity(vals.0.len());
            for entry in vals.0.iter() {
                let key = expand(entry.0.clone(), env.clone(), macros.clone(), cx)?;
                let val = expand(entry.1.clone(), env.clone(), macros.clone(), cx)?;
                evaluated.push((key, val));
            }
            return Ok(Value::map_from_vec(evaluated));
        }

        Value::App(ref app) => unimplemented!(),
    }
}
