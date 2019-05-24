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

/// Check the value `v` in the given environment. Treats all bindings in the environment as
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
                    match special(params)? {
                        Some(SpecialForm::Do(stmts)) => {
                            for stmt in stmts.iter() {
                                do_check(stmt, bindings)?;
                            }
                            Ok(())
                        }
                        Some(SpecialForm::Quote(_)) => Ok(()),
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
                        None => {
                            match bindings.get(id) {
                                Some(_) => {
                                    for param in params.0.iter() {
                                        do_check(param, bindings)?;
                                    }
                                    Ok(())
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
