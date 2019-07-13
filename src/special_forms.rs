use im_rc::{Vector as ImVector, OrdMap as ImOrdMap, OrdSet as ImOrdSet};
use gc::{Gc, GcCell};

use crate::gc_foreign::{Vector, OrdMap, OrdSet};
use crate::value::{Value, Id, Atomic, Fun, Opaque};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Code {
    Atomic(Atomic),
    Id(Id),
    Arr(Vector<Code>),
    App(Vector<Code>),
    Set(OrdSet<Code>),
    Map(OrdMap<Code, Code>),
    Fun(Fun),
    Cell(Gc<GcCell<Value>>, u64),
    Opaque(u64 /* creation id */, Opaque),
    Quote(Value),
    Do(Vector<Code>),
    SetBang(Id, Box<Code>),
    If(Box<Code>, Box<Code>, Box<Code>),
    Throw(Box<Code>),
    Try(Box<Code>, bool, Id, Box<Code>),
    Lambda(Vector<(bool, Id)>, Box<Code>),
    // Match(Box<Code>, Box<Pattern>, Box<Code>, Box<Code>),
    // LetFn(Vector<Id, Vector<(bool, Id)>, Code>, Box<Code>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pattern {
    Atomic(Atomic),
    Name(bool, Id), // true if mutable
    Arr(Vector<Pattern>),
    App(Vector<Pattern>),
    Map(OrdMap<Code, Pattern>, bool), // true if exact
    Guard(Box<Pattern>, Code),
    Named(bool, Id, Box<Pattern>), // true if mutable
    Typeof(Code),
    Eq(Code),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum FormType {
    Quote,
    Do,
    SetBang,
    If,
    Throw,
    Try,
    Lambda,
    // Match,
    // LetFn,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum SpecialFormSyntaxError {
    Arity(FormType, usize),
    Id(FormType, Value),
    SetBangId(Value),
    DoNotArray(Value),
    ArgsNotArray(Value),
    Binder(FormType, Value),
    // Pattern(Value),
}

pub fn to_code(v: &Value) -> Result<Code, SpecialFormSyntaxError> {
    match v {
        Value::Atomic(a) => Ok(Code::Atomic(a.clone())),
        Value::Id(id) => Ok(Code::Id(id.clone())),
        Value::Arr(arr) => {
            let mut code_arr = ImVector::new();
            for v_ in arr.0.iter() {
                code_arr.push_back(to_code(v_)?);
            }
            return Ok(Code::Arr(Vector(code_arr)));
        }
        Value::App(app) => {
            if app.0.len() == 0 {
                return Ok(Code::App(Vector(ImVector::new())));
            }

            match app.0[0].as_user_id() {
                None => {
                    let mut code_app = ImVector::new();
                    for v_ in app.0.iter() {
                        code_app.push_back(to_code(v_)?);
                    }
                    return Ok(Code::App(Vector(code_app)));
                }
                Some("sf-quote") => {
                    if app.0.len() != 2 {
                        return Err(SpecialFormSyntaxError::Arity(FormType::Quote, app.0.len()));
                    }

                    return Ok(Code::Quote(app.0[1].clone()));
                }

                Some("sf-do") => {
                    if app.0.len() != 2 {
                        return Err(SpecialFormSyntaxError::Arity(FormType::Do, app.0.len()));
                    }

                    match app.0[1].as_arr() {
                        None => return Err(SpecialFormSyntaxError::DoNotArray(app.0[1].clone())),
                        Some(arr) => {
                            let mut code_arr = ImVector::new();
                            for v_ in arr.0.iter() {
                                code_arr.push_back(to_code(v_)?);
                            }
                            return Ok(Code::Do(Vector(code_arr)));
                        }
                    }
                }

                Some("sf-set!") => {
                    if app.0.len() != 3 {
                        return Err(SpecialFormSyntaxError::Arity(FormType::SetBang, app.0.len()));
                    }

                    let id = match app.0[1].as_id() {
                        Some(id) => id,
                        None => return Err(SpecialFormSyntaxError::SetBangId(app.0[1].clone())),
                    };

                    return Ok(Code::SetBang(id.clone(), Box::new(to_code(&app.0[2])?)));
                }

                Some("sf-if") => {
                    if app.0.len() != 4 {
                        return Err(SpecialFormSyntaxError::Arity(FormType::If, app.0.len()));
                    }

                    return Ok(Code::If(
                        Box::new(to_code(&app.0[1])?),
                        Box::new(to_code(&app.0[2])?),
                        Box::new(to_code(&app.0[3])?),
                    ));
                }

                Some("sf-throw") => {
                    if app.0.len() != 2 {
                        return Err(SpecialFormSyntaxError::Arity(FormType::Throw, app.0.len()));
                    }

                    return Ok(Code::Throw(Box::new(to_code(&app.0[1])?)));
                }

                Some("sf-try") => {
                    if app.0.len() != 4 {
                        return Err(SpecialFormSyntaxError::Arity(FormType::Try, app.0.len()));
                    }

                    let (mutable, id) = mut_id(&app.0[2], FormType::Try)?;
                    return Ok(Code::Try(Box::new(to_code(&app.0[1])?), mutable, id, Box::new(to_code(&app.0[3])?)));
                }

                Some("sf-lambda") => {
                    if app.0.len() != 3  {
                        return Err(SpecialFormSyntaxError::Arity(FormType::Lambda, app.0.len()));
                    }

                    match app.0[1].as_arr() {
                        None => return Err(SpecialFormSyntaxError::ArgsNotArray(app.0[1].clone())),
                        Some(args_arr) => {
                            let mut args = ImVector::new();

                            for arg in args_arr.0.iter() {
                                args.push_back(mut_id(&arg, FormType::Lambda)?);
                            }

                            return Ok(Code::Lambda(Vector(args), Box::new(to_code(&app.0[2])?)));
                        }
                    }
                }

                _ => {
                    let mut code_app = ImVector::new();
                    for v_ in app.0.iter() {
                        code_app.push_back(to_code(v_)?);
                    }
                    return Ok(Code::App(Vector(code_app)));
                }
            }
        }

        Value::Set(set) => {
            let mut code_set = ImOrdSet::new();
            for v_ in set.0.iter() {
                code_set.insert(to_code(v_)?);
            }
            return Ok(Code::Set(OrdSet(code_set)));
        }

        Value::Map(map) => {
            let mut code_map = ImOrdMap::new();
            for (k_, v_) in map.0.iter() {
                code_map.insert(to_code(k_)?, to_code(v_)?);
            }
            return Ok(Code::Map(OrdMap(code_map)));
        }

        Value::Fun(fun) => return Ok(Code::Fun(fun.clone())),

        Value::Cell(cell, id) => return Ok(Code::Cell(cell.clone(), id.clone())),

        Value::Opaque(id, o) => return Ok(Code::Opaque(id.clone(), o.clone())),
    }
}

fn mut_id(v: &Value, ft: FormType) -> Result<(bool, Id), SpecialFormSyntaxError> {
    match v.as_id() {
        Some(id) => Ok((false, id.clone())),
        None => match v.as_app() {
            Some(app) => {
                if app.0.len() != 2 || !app.0[0].is_kw("mut") {
                    return Err(SpecialFormSyntaxError::Binder(ft, v.clone()));
                }
                match app.0[1].as_id() {
                    Some(id) => return Ok((true, id.clone())),
                    None => return Err(SpecialFormSyntaxError::Binder(ft, v.clone())),
                }
            }
            None => Err(SpecialFormSyntaxError::Binder(ft, v.clone())),
        }
    }
}

// pub fn pattern<'a>(v: &'a Value) -> Result<Pattern<'a>, &'a Value> {
//     match v {
//         Value::Atomic(a) => return Ok(Pattern::Atomic(a)),
//         Value::Id(id) => return Ok(Pattern::Name(false, id)),
//         Value::Arr(arr) => return Ok(Pattern::Arr(arr)),
//         Value::Map(map) => return Ok(Pattern::Map(map, false)),
//         Value::App(app) if app.0.len() >= 1 => {
//             match app.0[0].as_kw() {
//                 Some("mut") if app.0.len() == 2 => {
//                     match app.0[1].as_id() {
//                         Some(id) => return Ok(Pattern::Name(true, id)),
//                         None => return Err(v),
//                     }
//                 }
//                 Some("app") => return Ok(Pattern::App(Vector(app.0.skip(1)))),
//                 Some("guard") if app.0.len() == 3 => return Ok(Pattern::Guard(&app.0[1], &app.0[2])),
//                 Some("named") if app.0.len() == 3 => {
//                     match mut_id(&app.0[1], FormType::Quote /*ignored */) {
//                         Err(_) => return Err(v),
//                         Ok((mutable, name)) => return Ok(Pattern::Named(mutable, name, &app.0[2])),
//                     }
//                 }
//                 Some("typeof") if app.0.len() == 2 => return Ok(Pattern::Typeof(&app.0[1])),
//                 Some("=") if app.0.len() == 2 => return Ok(Pattern::Eq(&app.0[1])),
//                 _ => return Err(v),
//             }
//         }
//         _ => return Err(v),
//     }
// }
