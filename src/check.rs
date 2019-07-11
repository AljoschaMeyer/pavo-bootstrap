//! Static validity checks performed *after* macro expansion.
//!
//! This checks that:
//! - All unquoted identifiers are either binders, bound, or free but with a special form.
//! - All special forms are well-formed.
//! - The `set!` special form only mutates mutable bindings.

use std::collections::HashMap;

use im_rc::OrdMap;

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

pub fn check_toplevel(v: &Value, bindings: &HashMap<Id, (Value, bool)>) -> Result<(), StaticError> {
    let mut env = OrdMap::new();

    for (key, (_, mutability)) in bindings.iter() {
        env.insert(key.clone(), *mutability);
    }

    return check(v, &env);
}

pub fn check(
    v: &Value,
    bindings: &OrdMap<Id, bool /*mutability*/>
) -> Result<(), StaticError> {
    match v {
        Value::Atomic(..) | Value::Fun(..) | Value::Cell(..) | Value::Opaque(..) => Ok(()),

        Value::Id(id) => match bindings.get(id) {
            Some(_) => Ok(()),
            None => Err(StaticError::Free(id.clone())),
        }

        Value::Arr(vals) => {
            for val in vals.0.iter() {
                check(val, bindings)?
            }
            Ok(())
        }

        Value::Map(m) => {
            for entry in m.0.iter() {
                check(&entry.0, bindings)?;
                check(&entry.1, bindings)?;
            }
            Ok(())
        }

        Value::Set(vals) => {
            for val in vals.0.iter() {
                check(val, bindings)?
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
                                check(stmt, bindings)?;
                            }
                            Ok(())
                        }
                        Some(SpecialForm::Quote(_)) => Ok(()),
                        Some(SpecialForm::SetBang(id, body)) => {
                            match bindings.get(id) {
                                Some(true) => check(body, bindings),
                                Some(false) => Err(StaticError::Immutable(id.clone())),
                                None => Err(StaticError::Free(id.clone())),
                            }
                        }
                        Some(SpecialForm::If(cond, then, else_)) => {
                            check(cond, bindings)?;
                            check(then, bindings)?;
                            check(else_, bindings)
                        }
                        Some(SpecialForm::Throw(thrown)) => check(thrown, bindings),
                        Some(SpecialForm::Try(try_, mutable, bound, catch)) => {
                            let _ = check(try_, bindings)?;
                            check(catch, &bindings.update(bound.clone(), mutable))
                        }
                        Some(SpecialForm::Lambda(args, body)) => {
                            let mut fn_bindings = bindings.clone();
                            for (mutable, bound) in args {
                                fn_bindings = fn_bindings.update((*bound).clone(), mutable);
                            }
                            check(body, &fn_bindings)
                        }
                        None => {
                            match bindings.get(id) {
                                Some(_) => {
                                    for param in params.0.iter() {
                                        check(param, bindings)?;
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
                        check(param, bindings)?;
                    }
                    Ok(())
                },
            }
        }
    }
}
