//! Static validity checks performed *after* macro expansion.
//!
//! This checks that:
//! - All unquoted identifiers are either binders, bound, or free but with a special form.
//! - All special forms are well-formed.
//! - The `set!` special form only mutates mutable bindings.

use im_rc::OrdMap;

use crate::env::Env;
use crate::special_forms::{SpecialForm, SpecialFormSyntaxError, special};
use crate::value::{Value, Id};

/// All the things the syntax checker disallows.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum StaticError {
    Free(Id),
    Immutable(Id),
    SpecialFormSyntax(SpecialFormSyntaxError),
}

impl From<SpecialFormSyntaxError> for StaticError {
    fn from(err: SpecialFormSyntaxError) -> Self {
        StaticError::SpecialFormSyntax(err)
    }
}

/// Check the valect `o` in the given environment. Treats all bindings in the environment as
/// immutable.
pub fn check(v: &Value, env: &Env) -> Result<(), StaticError> {
    let mut bindings = OrdMap::new();
    for id in (env.0).0.keys() {
        bindings.insert(id.clone(), false);
    }
    do_check(v, &bindings)
}

fn do_check(
    v: &Value,
    bindings: &OrdMap<Id, bool /*mutability*/>
) -> Result<(), StaticError> {
    match v {
        Value::Atomic(..) | Value::Fun(..) => Ok(()),

        Value::Id(id) => match bindings.get(id) {
            Some(_) => Ok(()),
            None => Err(StaticError::Free(id.clone())),
        }

        Value::Arr(vals) => {
            for val in vals.0.iter() {
                do_check(val, bindings)?
            }
            Ok(())
        }

        Value::Map(m) => {
            for entry in m.0.iter() {
                do_check(&entry.0, bindings)?;
                do_check(&entry.1, bindings)?;
            }
            Ok(())
        }

        Value::Set(vals) => {
            for val in vals.0.iter() {
                do_check(val, bindings)?
            }
            Ok(())
        }

        Value::App(params) => {
            if params.0.len() == 0 {
                return Ok(());
            }

            let fst = &params.0[0];

            match &fst {
                Value::Id(id) => {
                    match bindings.get(id) {
                        Some(_) => {
                            for param in params.0.iter() {
                                do_check(param, bindings)?;
                            }
                            Ok(())
                        }
                        None => {
                            match special(params)? {
                                Some(SpecialForm::Do(stmts)) => {
                                    for stmt in stmts.iter() {
                                        do_check(stmt, bindings)?;
                                    }
                                    Ok(())
                                }
                                Some(SpecialForm::Quote(_)) => Ok(()),
                                Some(SpecialForm::Let(mutable, bound, _, cont)) => {
                                    do_check(cont, &bindings.update(bound.clone(), mutable))
                                }
                                Some(SpecialForm::SetBang(id, _)) => {
                                    match bindings.get(id) {
                                        Some(true) => Ok(()),
                                        Some(false) => Err(StaticError::Immutable(id.clone())),
                                        None => Err(StaticError::Free(id.clone())),
                                    }
                                }
                                Some(SpecialForm::If(cond, then, else_)) => {
                                    do_check(cond, bindings)?;
                                    do_check(then, bindings)?;
                                    do_check(else_, bindings)
                                }
                                Some(SpecialForm::Throw(thrown)) => do_check(thrown, bindings),
                                Some(SpecialForm::Try(try_, mutable, bound, catch)) => {
                                    let _ = do_check(try_, bindings)?;
                                    do_check(catch, &bindings.update(bound.clone(), mutable))
                                }
                                Some(SpecialForm::Lambda(mutable, bound, body)) => {
                                    do_check(body, &bindings.update(bound.clone(), mutable))
                                }
                                Some(SpecialForm::LetFn(funs, cont)) => {
                                    let mut inner_bindings = bindings.clone();
                                    for (name, ..) in funs.iter() {
                                        inner_bindings = inner_bindings.update((*name).clone(), false);
                                    }

                                    for (_, mutable, bound, body) in funs.iter() {
                                        let _ = do_check(body, &inner_bindings.update((*bound).clone(), *mutable))?;
                                    }

                                    do_check(cont, &inner_bindings)
                                }
                                None => Err(StaticError::Free(id.clone())),
                            }
                        }
                    }
                }

                // First element is not an identifier.
                _ => {
                    for param in params.0.iter() {
                        do_check(param, bindings)?;
                    }
                    Ok(())
                },
            }
        }
    }
}

// Check that all unquoted identifiers are either binders or bound, that all special forms are
// well-formed, and that only mutable bindings are being mutated.
// fn do_check(o: &Value, cx: &mut Context, bindings: &mut BTreeMap<Id, bool>) -> Result<(), StaticError> {
//     match &o.0 {
//         Value::Nil | Value::Bool(..) | Value::Int(..) | Value::Keyword(..)
//         | Value::Closure(..) | Value::Builtin(..) => Ok(()),
//         Value::Id(id) => match bindings.get(id) {
//             Some(_) => Ok(()),
//             None => Err(StaticError::Free(id.clone())),
//         }
//         Value::Arr(vals) => {
//             for val in vals.0.iter() {
//                 do_check(val, cx, bindings)?
//             }
//             Ok(())
//         }
//         Value::Map(m) => {
//             for entry in m.0.iter() {
//                 do_check(&entry.0, cx, bindings)?;
//                 do_check(&entry.1, cx, bindings)?;
//             }
//             Ok(())
//         }
//         Value::App(vals) => {
//             if vals.len() == 0 {
//                 return Ok(());
//             }
//
//             let fst = &vals.0[0];
//
//             match &fst.0 {
//                 Value::Id(id) => {
//                     match bindings.get(id) {
//                         Some(_) => {
//                             for arg in vals.0.iter() {
//                                 do_check(arg, cx, bindings)?;
//                             }
//                             Ok(())
//                         }
//                         None => match &id.chars[..] {
//                             "quote" => {
//                                 if vals.len() == 2 {
//                                     Ok(())
//                                 } else {
//                                     Err(StaticError::QuoteArity(o.clone()))
//                                 }
//                             }
//
//                             "named-lambdas" => {
//                                 let len = vals.len();
//                                 if len != 3 {
//                                     return Err(StaticError::LambdasArity(o.clone()));
//                                 }
//
//                                 let lambdas = vals.0[1].to_arr_vec_ref();
//                                 let lambdas_len = lambdas.len();
//
//                                 let mut names = Vec::with_capacity(lambdas_len);
//                                 for exp in lambdas.0.iter() {
//                                     match &exp.0 {
//                                         Value::Arr(named_lambda) => {
//                                             if named_lambda.len() != 3 && named_lambda.len() != 4 {
//                                                 return Err(StaticError::LambdaArity(exp.clone()));
//                                             }
//
//                                             match &named_lambda.0[0].0 {
//                                                 Value::Id(id) => names.push(id.clone()),
//                                                 _ => return Err(StaticError::LambdaName(exp.clone())),
//                                             }
//                                         }
//                                         _ => return Err(StaticError::LambdaArr(exp.clone())),
//                                     }
//                                 }
//
//                                 for name in names.iter() {
//                                     bindings.insert(name.clone(), false);
//                                 }
//
//                                 for exp in lambdas.0.iter() {
//                                     match &exp.0 {
//                                         Value::Arr(named_lambda) => {
//                                             let binder_id_index = if named_lambda.len() == 4 {
//                                                 match &named_lambda.0[1].0 {
//                                                     Value::Keyword(kw) if kw == "mut" => { /* noop*/ }
//                                                     _ => return Err(StaticError::LambdaMut(exp.clone())),
//                                                 }
//
//                                                 2
//                                             } else {
//                                                 1
//                                             };
//
//                                             let bound = match &named_lambda.0[binder_id_index].0 {
//                                                 Value::Id(id) => id.clone(),
//                                                 _ => return Err(StaticError::LambdaId(exp.clone())),
//                                             };
//
//                                             bindings.insert(bound.clone(), named_lambda.len() == 4);
//                                             do_check(&named_lambda.0[binder_id_index + 1], cx, bindings)?;
//                                             bindings.remove(&bound);
//                                         }
//                                         _ => unreachable!(),
//                                     }
//                                 }
//
//                                 do_check(&vals.0[2], cx, bindings)?;
//
//                                 for name in names.iter() {
//                                     bindings.remove(name);
//                                 }
//
//                                 Ok(())
//                             }
//
//                             "do" => {
//                                 for (i, inner) in vals.0.iter().enumerate() {
//                                     if i != 0 {
//                                         do_check(inner, cx, bindings)?;
//                                     }
//                                 }
//                                 Ok(())
//                             }
//
//                             "if" => {
//                                 if vals.len() != 3 && vals.len() != 4 {
//                                     return Err(StaticError::IfArity(o.clone()));
//                                 }
//
//                                 do_check(&vals.0[1], cx, bindings)?;
//                                 do_check(&vals.0[2], cx, bindings)?;
//
//                                 if vals.len() == 4 {
//                                     do_check(&vals.0[3], cx, bindings)?;
//                                 }
//
//                                 Ok(())
//                             }
//
//                             "set!" => {
//                                 if vals.len() != 3 {
//                                     return Err(StaticError::SetArity(o.clone()));
//                                 }
//
//                                 let snd = &vals.0[1];
//                                 match &snd.0 {
//                                     Value::Id(id) => match bindings.get(id) {
//                                         Some(true) => { /* noop, everything fine */}
//                                         Some(false) => return Err(StaticError::SetImmutable(snd.clone())),
//                                         None => return Err(StaticError::Free(id.clone())),
//                                     }
//                                     _ => return Err(StaticError::SetNotId(snd.clone())),
//                                 }
//
//                                 do_check(&vals.0[2], cx, bindings)
//                             }
//
//                             "throw" => {
//                                 if vals.len() == 2 {
//                                     do_check(&vals.0[1], cx, bindings)
//                                 } else {
//                                     Err(StaticError::ThrowArity(o.clone()))
//                                 }
//                             }
//
//                             "try" => {
//                                 if vals.len() != 4 && vals.len() != 5 {
//                                     return Err(StaticError::TryArity(o.clone()));
//                                 }
//
//                                 do_check(&vals.0[1], cx, bindings)?;
//
//                                 let binder_id_index = if vals.len() == 5 {
//                                     match &vals.0[2].0 {
//                                         Value::Keyword(kw) if kw == "mut" => { /* noop*/ }
//                                         _ => return Err(StaticError::TryMut(o.clone())),
//                                     }
//
//                                     3
//                                 } else {
//                                     2
//                                 };
//
//                                 let bound = match &vals.0[binder_id_index].0 {
//                                     Value::Id(id) => id.clone(),
//                                     _ => return Err(StaticError::TryId(o.clone())),
//                                 };
//
//                                 bindings.insert(bound.clone(), vals.len() == 5);
//                                 do_check(&vals.0[binder_id_index + 1], cx, bindings)?;
//                                 bindings.remove(&bound);
//
//                                 Ok(())
//                             }
//
//                             // special forms dealing with macros are already expanded
//
//                             _ => Err(StaticError::Free(id.clone())),
//                         }
//                     }
//                 }
//
//                 Value::App(_) => {
//                     for inner in vals.0.iter() {
//                         do_check(inner, cx, bindings)?;
//                     }
//                     Ok(())
//                 }
//
//                 _ => Err(StaticError::App(fst.clone())),
//             }
//         }
//     }
// }
