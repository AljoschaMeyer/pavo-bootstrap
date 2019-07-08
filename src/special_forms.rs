// Static typing for special forms to help implementing them. This doesn't actually *do*
// anything, its just a convenient/checked way of accessing special forms. The first
// attempt at implementing pavo without this layer very quickly turned into spaghetti.
use crate::gc_foreign::Vector;
use crate::value::{Value, Id};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum SpecialForm<'a> {
    Quote(&'a Value),
    Do(Vec<&'a Value>),
    SetBang(&'a Id, &'a Value),
    If(&'a Value, &'a Value, &'a Value),
    Throw(&'a Value),
    Try(&'a Value, bool, &'a Id, &'a Value),
    Lambda(Vec<(bool, &'a Id)>, &'a Value),
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
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum SpecialFormSyntaxError {
    Arity(FormType, usize),
    Id(FormType, Value),
    SetBangId(Value),
    DoNotArray(Value),
    ArgsNotArray(Value),
    Binder(FormType, Value),
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
            if v.0.len() != 2 {
                return Err(SpecialFormSyntaxError::Arity(FormType::Do, v.0.len()));
            }

            match v.0[1].as_arr() {
                None => return Err(SpecialFormSyntaxError::DoNotArray(v.0[1].clone())),
                Some(arr) => {
                    let mut do_stmts = Vec::with_capacity(v.0.len() - 1);

                    for stmt in arr.0.iter() {
                        do_stmts.push(stmt);
                    }

                    return Ok(Some(SpecialForm::Do(do_stmts)));
                }
            }
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
            if v.0.len() != 4 {
                return Err(SpecialFormSyntaxError::Arity(FormType::Try, v.0.len()));
            }

            let to_try = &v.0[1];
            let (mutable, id) = mut_id(&v.0[2], FormType::Try)?;
            let cont = &v.0[3];

            return Ok(Some(SpecialForm::Try(to_try, mutable, id, cont)));
        }

        Some("sf-lambda") => {
            if v.0.len() != 3  {
                return Err(SpecialFormSyntaxError::Arity(FormType::Lambda, v.0.len()));
            }

            match v.0[1].as_arr() {
                None => return Err(SpecialFormSyntaxError::ArgsNotArray(v.0[1].clone())),
                Some(args_arr) => {
                    let mut args = vec![];

                    for arg in args_arr.0.iter() {
                        args.push(mut_id(&arg, FormType::Lambda)?);
                    }

                    return Ok(Some(SpecialForm::Lambda(args, &v.0[2])));
                }
            }
        }

        _ => return Ok(None),
    }
}

fn mut_id<'a>(v: &'a Value, ft: FormType) -> Result<(bool, &'a Id), SpecialFormSyntaxError> {
    match v.as_id() {
        Some(id) => Ok((false, id)),
        None => match v.as_app() {
            Some(app) => {
                if app.0.len() != 2 || !app.0[0].is_kw("mut") {
                    return Err(SpecialFormSyntaxError::Binder(ft, v.clone()));
                }
                match app.0[1].as_id() {
                    Some(id) => return Ok((true, id)),
                    None => return Err(SpecialFormSyntaxError::Binder(ft, v.clone())),
                }
            }
            None => Err(SpecialFormSyntaxError::Binder(ft, v.clone())),
        }
    }
}
