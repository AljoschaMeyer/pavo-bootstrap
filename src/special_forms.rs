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
    Throw(Box<Code>),
    Try(Box<Code>, bool, Id, Box<Code>),
    Lambda(Vector<(bool, Id)>, Box<Code>),
    Case(Box<Code>, Vector<(Pattern, Code)>),
    LetFn(OrdMap<Id, (Vector<(bool, Id)>, Code)>, Box<Code>)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pattern {
    Atomic(Atomic),
    Name(bool, Id), // true if mutable
    Arr(Vector<Pattern>),
    App(Vector<Pattern>),
    Set(OrdSet<Value>),
    Map(OrdMap<Value, Pattern>),
    Named(bool, Id, Box<Pattern>), // true if mutable
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
    Case,
    LetFn,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum SpecialFormSyntaxError {
    Arity(FormType, usize),
    Id(FormType, Value),
    SetBangId(Value),
    DoNotArray(Value),
    ArgsNotArray(Value),
    CaseNotArray(Value),
    LetFnNotMap(Value),
    FnName(Value),
    OddCases(Value),
    Binder(FormType, Value),
    Pattern(Value),
    Foo,
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

                Some("sf-letfn") => {
                    if app.0.len() != 3  {
                        return Err(SpecialFormSyntaxError::Arity(FormType::Lambda, app.0.len()));
                    }

                    match app.0[1].as_map() {
                        None => return Err(SpecialFormSyntaxError::LetFnNotMap(app.0[1].clone())),
                        Some(map) => {
                            let mut code_map = ImOrdMap::new();
                            for (key, val) in map.0.iter() {
                                match key.as_id() {
                                    None => return Err(SpecialFormSyntaxError::FnName(key.clone())),
                                    Some(name) => {
                                        match val.as_app() {
                                            Some(fun) if fun.0.len() == 2 => {
                                                match fun.0[0].as_arr() {
                                                    None => return Err(SpecialFormSyntaxError::ArgsNotArray(fun.0[1].clone())),
                                                    Some(args_arr) => {
                                                        let mut args = ImVector::new();

                                                        for arg in args_arr.0.iter() {
                                                            args.push_back(mut_id(&arg, FormType::LetFn)?);
                                                        }

                                                        code_map.insert(name.clone(), (Vector(args), to_code(&fun.0[1])?));
                                                    }
                                                }
                                            }
                                            _ => return Err(SpecialFormSyntaxError::Foo),
                                        }
                                    }
                                }
                            }
                            return Ok(Code::LetFn(OrdMap(code_map), Box::new(to_code(&app.0[2])?)));
                        }
                    }
                }

                Some("sf-case") => {
                    if app.0.len() != 3 {
                        return Err(SpecialFormSyntaxError::Arity(FormType::Case, app.0.len()));
                    }

                    let c = to_code(&app.0[1])?;

                    match app.0[2].as_arr() {
                        None => return Err(SpecialFormSyntaxError::CaseNotArray(app.0[2].clone())),
                        Some(arr) => {
                            if arr.0.len() % 2 != 0 {
                                return Err(SpecialFormSyntaxError::OddCases(app.0[2].clone()));
                            }

                            let mut cases = ImVector::new();
                            let mut case = Pattern::Atomic(Atomic::Nil); // never used

                            for (i, inner) in arr.0.iter().enumerate() {
                                if i % 2 == 0 {
                                    case = pattern(inner)?;
                                } else {
                                    cases.push_back((case.clone(), to_code(inner)?));
                                }
                            }
                            return Ok(Code::Case(Box::new(c), Vector(cases)));
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

pub fn pattern(v: &Value) -> Result<Pattern, SpecialFormSyntaxError> {
    match v {
        Value::Atomic(a) => return Ok(Pattern::Atomic(a.clone())),
        Value::Id(id) => return Ok(Pattern::Name(false, id.clone())),
        Value::Arr(arr) => {
            let mut pattern_arr = ImVector::new();
            for v_ in arr.0.iter() {
                pattern_arr.push_back(pattern(v_)?);
            }
            return Ok(Pattern::Arr(Vector(pattern_arr)));
        }
        Value::Set(set) => return Ok(Pattern::Set(OrdSet(set.0.clone()))),
        Value::Map(map) => {
            let mut pattern_map = ImOrdMap::new();
            for (k_, v_) in map.0.iter() {
                pattern_map.insert(k_.clone(), pattern(v_)?);
            }
            return Ok(Pattern::Map(OrdMap(pattern_map)));
        }
        Value::App(app) if app.0.len() >= 1 => {
            match app.0[0].as_kw() {
                Some("mut") if app.0.len() == 2 => {
                    match app.0[1].as_id() {
                        Some(id) => return Ok(Pattern::Name(true, id.clone())),
                        None => return Err(SpecialFormSyntaxError::Pattern(v.clone())),
                    }
                }
                Some("app") => {
                    let mut pattern_app = ImVector::new();
                    for v_ in app.0.iter().skip(1) {
                        pattern_app.push_back(pattern(v_)?);
                    }
                    return Ok(Pattern::App(Vector(pattern_app)));
                }
                Some("named") if app.0.len() == 3 => {
                    match mut_id(&app.0[1], FormType::Quote /*ignored */) {
                        Err(_) => return Err(SpecialFormSyntaxError::Pattern(v.clone())),
                        Ok((mutable, name)) => return Ok(Pattern::Named(mutable, name, Box::new(pattern(&app.0[2])?))),
                    }
                }
                _ => return Err(SpecialFormSyntaxError::Pattern(v.clone())),
            }
        }
        _ => return Err(SpecialFormSyntaxError::Pattern(v.clone())),
    }
}
