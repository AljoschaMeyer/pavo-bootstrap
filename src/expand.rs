use im_rc::{OrdMap as ImOrdMap, Vector as ImVector};

use crate::context::Context;
use crate::gc_foreign::{OrdMap, Vector};
use crate::value::{Value, Id, Fun, _Fun, Builtin};
use crate::{expand_check_eval, E};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ExpandError {
    Arity(Value /* the form with incorrect arity */),
    MacroThrew(Value /* the thrown value */),
    Type(Value /* the macro value that isn't a function */),
    BodyEval(Box<E>),
    Pattern {
        pattern: Value,
        body: Value,
    },
}

impl From<E> for ExpandError {
    fn from(err: E) -> Self {
        ExpandError::BodyEval(Box::new(err))
    }
}

pub fn expand(v: &Value, env: Env, macros: &ImOrdMap<Id, Value>, cx: &mut Context) -> Result<Value, ExpandError> {
    match v {
        Value::Atomic(..) | Value::Id(..) | Value::Fun(..)  => Ok(v.clone()),

        Value::Arr(ref vals) => {
            let mut expanded = Vec::with_capacity(vals.0.len());
            for item in vals.0.iter() {
                expanded.push(expand(item, env.clone(), macros, cx)?);
            }
            return Ok(Value::arr_from_vec(expanded));
        }

        Value::Set(ref vals) => {
            let mut expanded = Vec::with_capacity(vals.0.len());
            for item in vals.0.iter() {
                expanded.push(expand(item, env.clone(), macros, cx)?);
            }
            return Ok(Value::set_from_vec(expanded));
        }

        Value::Map(ref vals) => {
            let mut expanded = Vec::with_capacity(vals.0.len());
            for entry in vals.0.iter() {
                let key = expand(&entry.0, env.clone(), macros, cx)?;
                let val = expand(&entry.1, env.clone(), macros, cx)?;
                expanded.push((key, val));
            }
            return Ok(Value::map_from_vec(expanded));
        }

        Value::App(ref vals) => {
            if vals.0.len() == 0 {
                return Ok(v.clone());
            }

            let fst = &vals.0[0];

            match fst {
                Value::Id(Id::User(id)) if id == "macro" => {
                    if vals.0.len() != 4 {
                        return Err(ExpandError::Arity(v.clone()));
                    }

                    let body = expand_check_eval(&vals.0[2], env.clone(), macros, env.clone(), cx)?;
                    let new_macros = match_macro(&body, &vals.0[1], macros)?;
                    expand(&vals.0[3], env, &new_macros, cx)
                }

                Value::Id(id) => match macros.get(id) {
                    Some(macro_) => {
                        match macro_ {
                            Value::Fun(macro_fun) => {
                                cx.inc_level();
                                let result = macro_fun.compute(
                                    Vector(ImVector::from(
                                        vals.0.iter().map(Clone::clone).skip(1).collect::<Vec<Value>>()
                                    )),
                                    cx
                                );
                                cx.dec_level();

                                match result {
                                    Ok(yay) => return expand(&yay, env, macros, cx),
                                    Err(nay) => return Err(ExpandError::MacroThrew(nay)),
                                }
                            }

                            _ => return Err(ExpandError::Type(macro_.clone())),
                        }
                    }

                    None => {
                        let mut expanded = Vec::with_capacity(vals.0.len());
                        for item in vals.0.iter() {
                            expanded.push(expand(item, env.clone(), macros, cx)?);
                        }
                        return Ok(Value::app_from_vec(expanded));
                    }
                }

                _ => {
                    let mut expanded = Vec::with_capacity(vals.0.len());
                    for item in vals.0.iter() {
                        expanded.push(expand(item, env.clone(), macros, cx)?);
                    }
                    return Ok(Value::app_from_vec(expanded));
                }
            }
        }
    }
}

fn match_macro(body: &Value, pattern: &Value, macros: &ImOrdMap<Id, Value>) -> Result<ImOrdMap<Id, Value>, ExpandError> {
    match pattern {
        Value::Id(id) => Ok(macros.update(id.clone(), body.clone())),

        Value::Arr(pattern_arr) => {
            match body.as_arr() {
                Some(body_arr) => {
                    if pattern_arr.0.len() != body_arr.0.len() {
                        return Err(ExpandError::Pattern { pattern: pattern.clone(), body: body.clone()});
                    }

                    let mut ret = macros.clone();
                    for i in 0..pattern_arr.0.len() {
                        ret = match_macro(&body_arr.0[i], &pattern_arr.0[i], &ret)?;
                    }

                    return Ok(ret);
                }
                None => return Err(ExpandError::Pattern { pattern: pattern.clone(), body: body.clone()}),
            }
        }

        Value::Map(pattern_map) => {
            match body.as_map() {
                Some(body_map) => {
                    let mut ret = macros.clone();

                    for (pattern_key, pattern_val) in pattern_map.0.iter() {
                        match body_map.0.get(pattern_key) {
                            None => return Err(ExpandError::Pattern { pattern: pattern.clone(), body: body.clone()}),
                            Some(body_val) => ret = match_macro(body_val, pattern_val, macros)?,
                        }
                    }

                    return Ok(ret);
                }
                None => return Err(ExpandError::Pattern { pattern: pattern.clone(), body: body.clone()}),
            }
        }

        _ => Err(ExpandError::Pattern { pattern: pattern.clone(), body: body.clone()})
    }
}
