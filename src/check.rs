//! Static validity checks performed *after* macro expansion.
//!
//! This checks that:
//! - All unquoted identifiers are either binders, bound, or free but with a special form.
//! - All special forms are well-formed.
//! - The `set!` special form only mutates mutable bindings.

use std::collections::BTreeMap;

use failure_derive::Fail;

use crate::env::Env;
use crate::object::{Object, Id};

/// All the things the syntax checker disallows.
#[derive(PartialEq, Eq, Debug, Clone, Fail)]
pub enum StaticError {
    #[fail(display = "TODO remove this")]
    Foo
}

/// Check the object `o` in the given environment. Treats all bindings in the environment as
/// immutable.
pub fn check(o: &Object, env: &Env) -> Result<(), StaticError> {
    let mut bindings = BTreeMap::new();
    for id in (env.0).0.keys() {
        bindings.insert(id.clone(), false);
    }
    do_check(o, &mut bindings)
}

fn do_check(
    o: &Object,
    bindings: &mut BTreeMap<Id, bool /*mutability*/>
) -> Result<(), StaticError> {
    unimplemented!()
}

// Check that all unquoted identifiers are either binders or bound, that all special forms are
// well-formed, and that only mutable bindings are being mutated.
// fn check_static(o: &Object, cx: &mut Context, bindings: &mut BTreeMap<Id, bool>) -> Result<(), StaticError> {
//     match &o.0 {
//         Value::Nil | Value::Bool(..) | Value::Int(..) | Value::Keyword(..)
//         | Value::Closure(..) | Value::Builtin(..) => Ok(()),
//         Value::Id(id) => match bindings.get(id) {
//             Some(_) => Ok(()),
//             None => Err(StaticError::Free(id.clone())),
//         }
//         Value::Arr(objs) => {
//             for obj in objs.0.iter() {
//                 check_static(obj, cx, bindings)?
//             }
//             Ok(())
//         }
//         Value::Map(m) => {
//             for entry in m.0.iter() {
//                 check_static(&entry.0, cx, bindings)?;
//                 check_static(&entry.1, cx, bindings)?;
//             }
//             Ok(())
//         }
//         Value::App(objs) => {
//             if objs.len() == 0 {
//                 return Ok(());
//             }
//
//             let fst = &objs.0[0];
//
//             match &fst.0 {
//                 Value::Id(id) => {
//                     match bindings.get(id) {
//                         Some(_) => {
//                             for arg in objs.0.iter() {
//                                 check_static(arg, cx, bindings)?;
//                             }
//                             Ok(())
//                         }
//                         None => match &id.chars[..] {
//                             "quote" => {
//                                 if objs.len() == 2 {
//                                     Ok(())
//                                 } else {
//                                     Err(StaticError::QuoteArity(o.clone()))
//                                 }
//                             }
//
//                             "named-lambdas" => {
//                                 let len = objs.len();
//                                 if len != 3 {
//                                     return Err(StaticError::LambdasArity(o.clone()));
//                                 }
//
//                                 let lambdas = objs.0[1].to_arr_vec_ref();
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
//                                             check_static(&named_lambda.0[binder_id_index + 1], cx, bindings)?;
//                                             bindings.remove(&bound);
//                                         }
//                                         _ => unreachable!(),
//                                     }
//                                 }
//
//                                 check_static(&objs.0[2], cx, bindings)?;
//
//                                 for name in names.iter() {
//                                     bindings.remove(name);
//                                 }
//
//                                 Ok(())
//                             }
//
//                             "do" => {
//                                 for (i, inner) in objs.0.iter().enumerate() {
//                                     if i != 0 {
//                                         check_static(inner, cx, bindings)?;
//                                     }
//                                 }
//                                 Ok(())
//                             }
//
//                             "if" => {
//                                 if objs.len() != 3 && objs.len() != 4 {
//                                     return Err(StaticError::IfArity(o.clone()));
//                                 }
//
//                                 check_static(&objs.0[1], cx, bindings)?;
//                                 check_static(&objs.0[2], cx, bindings)?;
//
//                                 if objs.len() == 4 {
//                                     check_static(&objs.0[3], cx, bindings)?;
//                                 }
//
//                                 Ok(())
//                             }
//
//                             "set!" => {
//                                 if objs.len() != 3 {
//                                     return Err(StaticError::SetArity(o.clone()));
//                                 }
//
//                                 let snd = &objs.0[1];
//                                 match &snd.0 {
//                                     Value::Id(id) => match bindings.get(id) {
//                                         Some(true) => { /* noop, everything fine */}
//                                         Some(false) => return Err(StaticError::SetImmutable(snd.clone())),
//                                         None => return Err(StaticError::Free(id.clone())),
//                                     }
//                                     _ => return Err(StaticError::SetNotId(snd.clone())),
//                                 }
//
//                                 check_static(&objs.0[2], cx, bindings)
//                             }
//
//                             "throw" => {
//                                 if objs.len() == 2 {
//                                     check_static(&objs.0[1], cx, bindings)
//                                 } else {
//                                     Err(StaticError::ThrowArity(o.clone()))
//                                 }
//                             }
//
//                             "try" => {
//                                 if objs.len() != 4 && objs.len() != 5 {
//                                     return Err(StaticError::TryArity(o.clone()));
//                                 }
//
//                                 check_static(&objs.0[1], cx, bindings)?;
//
//                                 let binder_id_index = if objs.len() == 5 {
//                                     match &objs.0[2].0 {
//                                         Value::Keyword(kw) if kw == "mut" => { /* noop*/ }
//                                         _ => return Err(StaticError::TryMut(o.clone())),
//                                     }
//
//                                     3
//                                 } else {
//                                     2
//                                 };
//
//                                 let bound = match &objs.0[binder_id_index].0 {
//                                     Value::Id(id) => id.clone(),
//                                     _ => return Err(StaticError::TryId(o.clone())),
//                                 };
//
//                                 bindings.insert(bound.clone(), objs.len() == 5);
//                                 check_static(&objs.0[binder_id_index + 1], cx, bindings)?;
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
//                     for inner in objs.0.iter() {
//                         check_static(inner, cx, bindings)?;
//                     }
//                     Ok(())
//                 }
//
//                 _ => Err(StaticError::App(fst.clone())),
//             }
//         }
//     }
// }
