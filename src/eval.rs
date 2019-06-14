use std::collections::BTreeMap;

use gc::Gc;

use crate::builtins::{index_error, type_error};
use crate::context::Context;
use crate::env::Env;
use crate::gc_foreign::Vector;
use crate::special_forms::{SpecialForm, Args, special};
use crate::value::{Value, Id, Fun, _Fun, Builtin};
use crate::vm::Closure;

// Takes an already syntactically checked value and reduces it.
pub fn eval(v: Value, env: Env, cx: &mut Context) -> Result<Value, Value> {
    do_eval(v, env, cx, true)
}

pub fn do_eval(mut v: Value, mut env: Env, cx: &mut Context, tail: bool) -> Result<Value, Value> {
    unimplemented!();
    // loop {
    //     match v {
    //         Value::Atomic(..) | Value::Fun(..) => return Ok(v.clone()),
    //
    //         Value::Id(ref id) => return Ok(env.get(id).expect("static checks should ensure that id is bound")),
    //
    //         Value::Arr(ref vals) => {
    //             let mut evaluated = Vec::with_capacity(vals.0.len());
    //             for item in vals.0.iter() {
    //                 evaluated.push(do_eval(item.clone(), env.clone(), cx, false)?);
    //             }
    //             return Ok(Value::arr_from_vec(evaluated));
    //         }
    //
    //         Value::Set(ref vals) => {
    //             let mut evaluated = Vec::with_capacity(vals.0.len());
    //             for item in vals.0.iter() {
    //                 evaluated.push(do_eval(item.clone(), env.clone(), cx, false)?);
    //             }
    //             return Ok(Value::set_from_vec(evaluated));
    //         }
    //
    //         Value::Map(ref vals) => {
    //             let mut evaluated = Vec::with_capacity(vals.0.len());
    //             for entry in vals.0.iter() {
    //                 let key = do_eval(entry.0.clone(), env.clone(), cx, false)?;
    //                 let val = do_eval(entry.1.clone(), env.clone(), cx, false)?;
    //                 evaluated.push((key, val));
    //             }
    //             return Ok(Value::map_from_vec(evaluated));
    //         }
    //
    //         Value::App(ref params) => {
    //             if params.0.len() == 0 {
    //                 return Err(index_error(0));
    //             }
    //
    //             let fst = &params.0[0];
    //
    //             match fst {
    //                 Value::App(_) => {
    //                     let first_evaluated = do_eval(fst.clone(), env.clone(), cx, tail)?;
    //                     v = Value::app(Vector(params.0.update(0, first_evaluated)));
    //                 }
    //
    //                 Value::Id(id) => {
    //                     match special(&params).expect("static analysis should have caught malformed special forms") {
    //                         Some(SpecialForm::Do(stmts)) => {
    //                             let len = stmts.len();
    //                             if len == 0 {
    //                                 return Ok(Value::nil());
    //                             }
    //
    //                             for (i, stmt) in stmts.iter().enumerate() {
    //                                 if i + 1 < len {
    //                                     let _ = do_eval((*stmt).clone(), env.clone(), cx, false)?;
    //                                 }
    //                             }
    //                             v = stmts[len - 1].clone();
    //                         }
    //                         Some(SpecialForm::Quote(quoted)) => return Ok((*quoted).clone()),
    //                         Some(SpecialForm::SetBang(id, val)) => {
    //                             env.set(id, val.clone());
    //                             return Ok(Value::nil());
    //                         }
    //                         Some(SpecialForm::If(cond, then, else_)) => {
    //                             if do_eval((*cond).clone(), env.clone(), cx, false)?.truthy() {
    //                                 v = (*then).clone();
    //                             } else {
    //                                 v = (*else_).clone();
    //                             }
    //                         }
    //                         Some(SpecialForm::Throw(thrown)) => {
    //                             return Err(do_eval((*thrown).clone(), env.clone(), cx, false)?);
    //                         }
    //                         Some(SpecialForm::Try(try_, _, bound, catch)) => {
    //                             match do_eval((*try_).clone(), env.clone(), cx, false) {
    //                                 Ok(yay) => return Ok(yay),
    //                                 Err(thrown) => {
    //                                     env = env.update(bound.clone(), thrown.clone());
    //                                     v = (*catch).clone();
    //                                 }
    //                             }
    //                         }
    //                         Some(SpecialForm::LetFn(funs, cont)) => {
    //                             let mut funs_map = BTreeMap::new();
    //                             let mut new_env = env.clone();
    //                             for (name, args, body) in funs.iter() {
    //                                 match args {
    //                                     Args::All(mutable, bound) => {
    //                                         funs_map.insert(
    //                                             (*name).clone(),
    //                                             (ClosureArgs::All(*mutable, (*bound).clone()), (*body).clone())
    //                                         );
    //                                         new_env = new_env.update((*name).clone(), Value::nil());
    //                                     }
    //                                     Args::Destructured(the_args) => {
    //                                         funs_map.insert(
    //                                             (*name).clone(),
    //                                             (ClosureArgs::Destructured(the_args.iter().map(|arg|
    //                                                 (arg.0, arg.1.clone())
    //                                             ).collect()), (*body).clone())
    //                                         );
    //                                         new_env = new_env.update((*name).clone(), Value::nil());
    //                                     }
    //                                 }
    //                             }
    //
    //                             let funs_gc = Gc::new(funs_map);
    //
    //                             for (name, ..) in funs.iter() {
    //                                 new_env.set(name.clone(), Value::closure(Closure {
    //                                     funs: funs_gc.clone(),
    //                                     entry: (*name).clone(),
    //                                     env: new_env.clone(),
    //                                 }, cx));
    //                             }
    //
    //                             env = new_env;
    //                             v = (*cont).clone();
    //                         }
    //                         None => {
    //                             match env.get(id) {
    //                                 Some(resolved) => {
    //                                     v = Value::app(Vector(params.0.update(0, resolved)));
    //                                 }
    //
    //                                 None => panic!("static analysis should have caught free ids"),
    //                             }
    //                         }
    //                     }
    //                 }
    //
    //                 Value::Fun(Fun {fun: _Fun::Builtin(Builtin(b)), ..}) => {
    //                     let mut arg_vec = Vec::with_capacity(params.0.len() - 1);
    //                     for param in params.0.iter().skip(1) {
    //                         arg_vec.push(do_eval(param.clone(), env.clone(), cx, false)?);
    //                     }
    //                     let arg = Value::arr_from_vec(arg_vec);
    //
    //                     return b(arg, cx);
    //                 }
    //
    //                 Value::Fun(Fun {fun: _Fun::Closure(Closure {
    //                     funs,
    //                     entry,
    //                     env: c_env,
    //                 }), ..}) => {
    //                     let (args, body) = funs.get(entry).unwrap();
    //
    //                     match args {
    //                         ClosureArgs::All(_, arg_id) => {
    //                             let mut arg_vec = Vec::with_capacity(params.0.len() - 1);
    //                             for param in params.0.iter().skip(1) {
    //                                 arg_vec.push(do_eval(param.clone(), env.clone(), cx, false)?);
    //                             }
    //                             let arg = Value::arr_from_vec(arg_vec);
    //
    //                             env = c_env.update(arg_id.clone(), arg);
    //                             v = body.clone();
    //                         }
    //
    //                         ClosureArgs::Destructured(cls_args) => {
    //                             unimplemented!();
    //                         }
    //                     }
    //
    //                     // let (arg_id, _, body) = funs.get(entry).unwrap();
    //                     //
    //                     // let mut arg_vec = Vec::with_capacity(params.0.len() - 1);
    //                     // for param in params.0.iter().skip(1) {
    //                     //     arg_vec.push(do_eval(param.clone(), env.clone(), cx, false)?);
    //                     // }
    //                     // let arg = Value::arr_from_vec(arg_vec);
    //                     //
    //                     // env = c_env.update(arg_id.clone(), arg);
    //                     // v = body.clone();
    //                 }
    //
    //                 Value::Fun(Fun {fun: _Fun::Apply, ..}) => {
    //                     if params.0.len() == 1 {
    //                         return Err(type_error(&Value::nil(), "function"));
    //                     }
    //                     if params.0.len() == 2 {
    //                         return Err(type_error(&Value::nil(), "array"));
    //                     }
    //
    //                     let to_apply = &params.0[1];
    //                     let args = &params.0[2];
    //
    //                     match args {
    //                         Value::Arr(arr) => {
    //                             let mut tmp = arr.clone();
    //                             tmp.0.push_front(to_apply.clone());
    //                             v = Value::app(tmp);
    //                         }
    //                         _ => return Err(type_error(&args, "array")),
    //                     }
    //                 }
    //
    //                 _ => return Err(type_error(fst, "function")),
    //             }
    //         }
    //     }
    // }
}
