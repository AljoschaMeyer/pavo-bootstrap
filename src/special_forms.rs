// Static typing for special forms to help implementing them. This doesn't actually *do*
// anything, its just a convenient/checked way of accessing special forms. The first
// attempt at implementing pavo without this layer very quickly turned into spaghetti.

use im_rc::Vector as ImVector;

use crate::gc_foreign::Vector;
use crate::value::{Value, Atomic, Id};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum SpecialForm<'a> {
    Quote(&'a Value),
    Do(Vec<&'a Value>),
    Let(bool, &'a Id, &'a Value, &'a Value),
    SetBang(&'a Id, &'a Value),
    If(&'a Value, &'a Value, &'a Value),
    Throw(&'a Value),
    Try(&'a Value, bool, &'a Id, &'a Value),
    Lambda(bool, &'a Id, &'a Value),
    LetFn(Vec<(&'a Id, bool, &'a Id, &'a Value)>, &'a Value),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum FormType {
    Quote,
    Do,
    Let,
    SetBang,
    If,
    Throw,
    Try,
    Lambda,
    LetFn,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum SpecialFormSyntaxError {
    Arity(FormType, usize),
    Id(FormType, Value),
    SetBangId(Value),
    LetFnNoParens(Value),
    FnName(Value),
}

pub fn special<'a>(v: &'a Vector<Value>) -> Result<Option<SpecialForm<'a>>, SpecialFormSyntaxError> {
    if v.0.len() == 0 {
        return Ok(None);
    }

    match v.0[0].as_user_id() {
        None => return Ok(None),
        Some("sf-quote") => {
            if v.0.len() != 2 {
                return Err(SpecialFormSyntaxError::Arity(FormType::Quote, v.0.len()));
            }

            return Ok(Some(SpecialForm::Quote(&v.0[1])));
        }

        Some("sf-do") => {
            let mut do_stmts = Vec::with_capacity(v.0.len() - 1);

            for stmt in v.0.iter().skip(1) {
                do_stmts.push(stmt);
            }

            return Ok(Some(SpecialForm::Do(do_stmts)));
        }

        Some("sf-let") => {
            if v.0.len() != 4 && v.0.len() != 5  {
                return Err(SpecialFormSyntaxError::Arity(FormType::Let, v.0.len()));
            }

            let (mutable, id) = mut_id(&v.0, 1, FormType::Let)?;
            if !mutable && v.0.len() == 5 {
                return Err(SpecialFormSyntaxError::Arity(FormType::Let, v.0.len()));
            }
            let val = &v.0[if mutable { 3 } else { 2 }];
            let cont = &v.0[if mutable { 4 } else { 3 }];

            return Ok(Some(SpecialForm::Let(mutable, id, val, cont)));
        }

        Some("sf-set!") => {
            if v.0.len() != 3 {
                return Err(SpecialFormSyntaxError::Arity(FormType::SetBang, v.0.len()));
            }

            let id = match v.0[1].as_id() {
                Some(id) => id,
                None => return Err(SpecialFormSyntaxError::SetBangId(v.0[1].clone())),
            };

            return Ok(Some(SpecialForm::SetBang(id, &v.0[2])));
        }

        Some("sf-if") => {
            if v.0.len() != 4 {
                return Err(SpecialFormSyntaxError::Arity(FormType::If, v.0.len()));
            }

            return Ok(Some(SpecialForm::If(&v.0[1], &v.0[2], &v.0[3])));
        }

        Some("sf-throw") => {
            if v.0.len() != 2 {
                return Err(SpecialFormSyntaxError::Arity(FormType::Throw, v.0.len()));
            }

            return Ok(Some(SpecialForm::Throw(&v.0[1])));
        }

        Some("sf-try") => {
            if v.0.len() != 4 && v.0.len() != 5  {
                return Err(SpecialFormSyntaxError::Arity(FormType::Try, v.0.len()));
            }

            let to_try = &v.0[1];
            let (mutable, id) = mut_id(&v.0, 2, FormType::Try)?;
            if !mutable && v.0.len() == 5 {
                return Err(SpecialFormSyntaxError::Arity(FormType::Try, v.0.len()));
            }
            let cont = &v.0[if mutable { 4 } else { 3 }];

            return Ok(Some(SpecialForm::Try(to_try, mutable, id, cont)));
        }

        Some("sf-lambda") => {
            if v.0.len() != 3 && v.0.len() != 4  {
                return Err(SpecialFormSyntaxError::Arity(FormType::Lambda, v.0.len()));
            }

            let (mutable, id) = mut_id(&v.0, 1, FormType::Let)?;
            if !mutable && v.0.len() == 4 {
                return Err(SpecialFormSyntaxError::Arity(FormType::Lambda, v.0.len()));
            }
            let cont = &v.0[if mutable { 3 } else { 2 }];

            return Ok(Some(SpecialForm::Lambda(mutable, id, cont)));
        }

        Some("sf-letfn") => {
            let total = v.0.len();
            if total < 2 {
                return Err(SpecialFormSyntaxError::Arity(FormType::LetFn, v.0.len()));
            }

            let mut funs = Vec::with_capacity(total - 2);
            for exp in v.0.iter().skip(1).take(total - 2) {
                match exp.as_app() {
                    Some(fun_def) => {
                        if fun_def.0.len() != 3 && fun_def.0.len() != 4  {
                            return Err(SpecialFormSyntaxError::Arity(FormType::LetFn, fun_def.0.len()));
                        }

                        let name = match fun_def.0[0].as_id() {
                            Some(name) => name,
                            None => return Err(SpecialFormSyntaxError::FnName(fun_def.0[0].clone())),
                        };
                        let (mutable, id) = mut_id(&fun_def.0, 1, FormType::LetFn)?;
                        if !mutable && fun_def.0.len() == 4 {
                            return Err(SpecialFormSyntaxError::Arity(FormType::LetFn, fun_def.0.len()));
                        }
                        let cont = &fun_def.0[if mutable { 3 } else { 2 }];

                        funs.push((name, mutable, id, cont));
                    }
                    None => return Err(SpecialFormSyntaxError::LetFnNoParens(exp.clone())),
                }
            }

            return Ok(Some(SpecialForm::LetFn(funs, &v.0[total - 1])));
        }

        _ => return Ok(None),
    }
}

fn mut_id<'a>(v: &'a ImVector<Value>, start_at: usize, ft: FormType) -> Result<(bool, &'a Id), SpecialFormSyntaxError> {
    if v[start_at].is_kw("mut") {
        match v[start_at + 1].as_id() {
            Some(id) => Ok((true, id)),
            None => Err(SpecialFormSyntaxError::Id(ft, v[start_at + 1].clone()))
        }
    } else {
        match v[start_at].as_id() {
            Some(id) => Ok((false, id)),
            None => Err(SpecialFormSyntaxError::Id(ft, v[start_at].clone()))
        }
    }
}

// TODO the stuff below is for the builtin macros - the special forms became more lightweight since.

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub enum SpecialForm<'a> {
//     Quote(&'a Value),
//     Do(Vec<&'a Value>),
//     Let(Pattern<'a>, &'a Value, &'a Value),
//     SetBang(&'a Id, &'a Value),
//     If(Vec<(&'a Value, &'a Value)>, Option<&'a Value>),
//     Case(&'a Value, Vec<(Pattern<'a>, &'a Value)>),
//     Throw(Option<&'a Value>),
//     Try(&'a Value, Vec<(Pattern<'a>, &'a Value)>),
//     Fun(Option<&'a Id>, Vec<(Pattern<'a>, &'a Value)>),
//     LetFn(Vec<(&'a Id, Vec<(Pattern<'a>, &'a Value)>)>, &'a Value),
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
// pub enum FormType {
//     Quote,
//     Do,
//     Let,
//     SetBang,
//     If,
//     Case,
//     Throw,
//     Try,
//     LetFn,
// }
//
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub enum SpecialFormSyntaxError {
//     Arity(FormType, usize),
//     Pattern(PatternError),
//     SetBangId(Value),
//     LetFnNoParens(Value),
// }
//
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub enum Pattern<'a> {
//     Id(bool /* mutability */, &'a Id),
//     Atomic(&'a Atomic),
//     Arr(Vec<Pattern<'a>>),
// }
//
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
// pub enum PatternError {
//     NotPattern(Value),
//     AppNotKw(Value),
//     Arity(Value, usize),
//     MutableNotId(Value),
// }
//
// impl From<PatternError> for SpecialFormSyntaxError {
//     fn from(err: PatternError) -> Self {
//         SpecialFormSyntaxError::Pattern(err)
//     }
// }
//
// pub fn special<'a>(v: &'a Vector<Value>) -> Result<Option<SpecialForm<'a>>, SpecialFormSyntaxError> {
//     if v.0.len() == 0 {
//         return Ok(None);
//     }
//
//     match v.0[0].as_id() {
//         None => return Ok(None),
//         Some(id) => match id.get_chars() {
//             "quote" => {
//                 if v.0.len() != 2 {
//                     return Err(SpecialFormSyntaxError::Arity(FormType::Quote, v.0.len()));
//                 }
//
//                 return Ok(Some(SpecialForm::Quote(&v.0[1])));
//             }
//
//             "do" => {
//                 let mut do_stmts = Vec::with_capacity(v.0.len() - 1);
//
//                 for stmt in v.0.iter().skip(1) {
//                     do_stmts.push(stmt);
//                 }
//
//                 return Ok(Some(SpecialForm::Do(do_stmts)));
//             }
//
//             "let" => {
//                 if v.0.len() != 4 {
//                     return Err(SpecialFormSyntaxError::Arity(FormType::Let, v.0.len()));
//                 }
//
//                 let pat = pattern(&v.0[1])?;
//                 return Ok(Some(SpecialForm::Let(pat, &v.0[2], &v.0[3])));
//             }
//
//             "set!" => {
//                 if v.0.len() != 3 {
//                     return Err(SpecialFormSyntaxError::Arity(FormType::SetBang, v.0.len()));
//                 }
//
//                 let id = match v.0[1].as_id() {
//                     Some(id) => id,
//                     None => return Err(SpecialFormSyntaxError::SetBangId(v.0[1].clone())),
//                 };
//
//                 return Ok(Some(SpecialForm::SetBang(id, &v.0[2])));
//             }
//
//             "if" => {
//                 let total = v.0.len();
//                 let mut pairs = Vec::with_capacity(total / 2);
//                 let mut i = 1;
//
//                 while i + 1 < total {
//                     pairs.push((&v.0[i], &v.0[i + 1]));
//                     i += 2;
//                 }
//
//                 let else_ = if total % 2 == 0 {
//                     Some(&v.0[total - 1])
//                 } else {
//                     None
//                 };
//                 return Ok(Some(SpecialForm::If(pairs, else_)));
//             }
//
//             "case" => {
//                 let total = v.0.len();
//                 if total < 2 || total % 2 == 1 {
//                     return Err(SpecialFormSyntaxError::Arity(FormType::Case, v.0.len()));
//                 }
//
//                 let mut pairs = Vec::with_capacity(total / 2);
//                 let mut i = 2;
//
//                 while i + 1 < total {
//                     pairs.push((pattern(&v.0[i])?, &v.0[i + 1]));
//                     i += 2;
//                 }
//
//                 return Ok(Some(SpecialForm::Case(&v.0[1], pairs)));
//             }
//
//             "throw" => {
//                 if v.0.len() > 2 {
//                     return Err(SpecialFormSyntaxError::Arity(FormType::Throw, v.0.len()));
//                 }
//
//                 return Ok(Some(SpecialForm::Throw(v.0.get(1))));
//             }
//
//             "try" => {
//                 let total = v.0.len();
//                 if total < 2 || total % 2 == 1 {
//                     return Err(SpecialFormSyntaxError::Arity(FormType::Try, v.0.len()));
//                 }
//
//                 let mut pairs = Vec::with_capacity(total / 2);
//                 let mut i = 2;
//
//                 while i + 1 < total {
//                     pairs.push((pattern(&v.0[i])?, &v.0[i + 1]));
//                     i += 2;
//                 }
//
//                 return Ok(Some(SpecialForm::Try(&v.0[1], pairs)));
//             }
//
//             "fn" => {
//                 let total = v.0.len();
//
//                 let name = if total % 2 == 0 {
//                     v.0[1].as_id()
//                 } else {
//                     None
//                 };
//
//                 let mut pairs = Vec::with_capacity(total / 2);
//                 let mut i = 2;
//
//                 while i + 1 < total {
//                     pairs.push((pattern(&v.0[i])?, &v.0[i + 1]));
//                     i += 2;
//                 }
//
//                 return Ok(Some(SpecialForm::Fun(name, pairs)));
//             }
//
//             "letfn" => {
//                 let total = v.0.len();
//                 if total < 2 {
//                     return Err(SpecialFormSyntaxError::Arity(FormType::LetFn, v.0.len()));
//                 }
//
//                 let mut funs = Vec::with_capacity(total - 2);
//                 for exp in v.0.iter().skip(1).take(total - 2) {
//                     match exp.as_app() {
//                         Some(fun_def) => {
//                             let total = fun_def.0.len();
//
//                             let name = match fun_def.0[0].as_id() {
//                                 Some(name) => name,
//                                 None => return Err(unimplemented!()),
//                             };
//
//                             let mut pairs = Vec::with_capacity(total / 2);
//                             let mut i = 1;
//
//                             while i + 1 < total {
//                                 pairs.push((pattern(&v.0[i])?, &v.0[i + 1]));
//                                 i += 2;
//                             }
//
//                             funs.push((name, pairs));
//                         }
//                         None => return Err(SpecialFormSyntaxError::LetFnNoParens(exp.clone())),
//                     }
//                 }
//
//                 return Ok(Some(SpecialForm::LetFn(funs, &v.0[total - 1])));
//             }
//
//             _ => return Ok(None),
//         }
//     }
// }
//
// pub fn pattern<'a>(v: &'a Value) -> Result<Pattern<'a>, PatternError> {
//     match v {
//         Value::Id(id) => return Ok(Pattern::Id(false, id)),
//         Value::Atomic(atomic) => return Ok(Pattern::Atomic(atomic)),
//         Value::Arr(Vector(arr)) => {
//             let mut inners = Vec::with_capacity(arr.len());
//
//             for inner in arr.iter() {
//                 inners.push(pattern(inner)?);
//             }
//
//             return Ok(Pattern::Arr(inners));
//         }
//         Value::App(Vector(app)) => {
//             if app.len() < 2 {
//                 return Err(PatternError::Arity(v.clone(), app.len()));
//             }
//
//             let kw = match app[0].as_kw() {
//                 None => return Err(PatternError::AppNotKw(v.clone())),
//                 Some(kw) => kw,
//             };
//
//             match kw {
//                 "mut" => {
//                     if app.len() > 2 {
//                         return Err(PatternError::Arity(v.clone(), app.len()));
//                     }
//
//                     match app[1].as_id() {
//                         Some(id) => return Ok(Pattern::Id(true, id)),
//                         None => return Err(PatternError::MutableNotId(v.clone())),
//                     }
//                 }
//                 _ => return Err(PatternError::AppNotKw(v.clone())),
//             }
//         }
//         _ => return Err(PatternError::NotPattern(v.clone())),
//     }
// }
