// Static typing for special forms to help implementing them. This doesn't actually *do*
// anything, its just a convenient/checked way of accessing special forms. The first
// attempt at implementing pavo without this layer very quickly turned into spaghetti.

use im_rc::Vector as ImVector;

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
    Lambda(Args<'a>, &'a Value),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum Args<'a> {
    All(bool, &'a Id),
    Destructured(Vec<(bool, &'a Id)>),
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

            let (args, body) = fun_def(&v.0, 1, FormType::Lambda)?;
            return Ok(Some(SpecialForm::Lambda(args, body)));
        }

        _ => return Ok(None),
    }
}

fn fun_def<'a>(v: &'a ImVector<Value>, start_at: usize, ft: FormType) -> Result<(Args<'a>, &'a Value), SpecialFormSyntaxError> {
    match v[start_at].as_arr() {
        Some(args_arr) => {
            let mut args = vec![];

            let mut i = 0;
            while i < args_arr.0.len() {
                let (mutable, id) = mut_id(&args_arr.0, i, ft)?;
                args.push((mutable, id));
                i += if mutable { 2 } else { 1 };
            }

            if v.len() != start_at + 2 {
                return Err(SpecialFormSyntaxError::Arity(ft, v.len()));
            }

            let body = &v[start_at + 1];
            return Ok((Args::Destructured(args), body));
        }

        None => {
            let (mutable, id) = mut_id(v, start_at, ft)?;

            if mutable {
                if v.len() != start_at + 3 {
                    return Err(SpecialFormSyntaxError::Arity(ft, v.len()));
                }
            } else {
                if v.len() != start_at + 2 {
                    return Err(SpecialFormSyntaxError::Arity(ft, v.len()));
                }
            }

            let body = &v[if mutable { start_at + 2 } else { start_at + 1 }];
            return Ok((Args::All(mutable, id), body));
        }
    }
}

fn mut_id<'a>(v: &'a ImVector<Value>, start_at: usize, ft: FormType) -> Result<(bool, &'a Id), SpecialFormSyntaxError> {
    // println!("{:?}", (v, start_at));
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
