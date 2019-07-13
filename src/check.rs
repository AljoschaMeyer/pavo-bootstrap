use std::collections::HashMap;

use im_rc::OrdMap;

use crate::special_forms::Code;
use crate::value::{Value, Id};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BindingError {
    Free(Id),
    Immutable(Id),
}

pub fn check_toplevel(c: Code, bindings: &HashMap<Id, (Value, bool)>) -> Result<(), BindingError> {
    let mut env = OrdMap::new();

    for (key, (_, mutability)) in bindings.iter() {
        env.insert(key.clone(), *mutability);
    }

    return check(c, &env);
}

pub fn check(
    c: Code,
    bindings: &OrdMap<Id, bool /*mutability*/>
) -> Result<(), BindingError> {
    match c {
        Code::Atomic(..) | Code::Fun(..) | Code::Cell(..) | Code::Opaque(..) => Ok(()),

        Code::Id(id) => match bindings.get(&id) {
            Some(_) => Ok(()),
            None => Err(BindingError::Free(id.clone())),
        }

        Code::Arr(vals) => {
            for val in vals.0.iter() {
                check(val.clone(), bindings)?
            }
            Ok(())
        }

        Code::App(vals) => {
            for val in vals.0.iter() {
                check(val.clone(), bindings)?
            }
            Ok(())
        }

        Code::Map(m) => {
            for entry in m.0.iter() {
                check(entry.0.clone(), bindings)?;
                check(entry.1.clone(), bindings)?;
            }
            Ok(())
        }

        Code::Set(vals) => {
            for val in vals.0.iter() {
                check(val.clone(), bindings)?
            }
            Ok(())
        }

        Code::Quote(_) => return Ok(()),

        Code::Do(stmts) => {
            for stmt in stmts.0.iter() {
                check(stmt.clone(), bindings)?;
            }
            Ok(())
        }

        Code::SetBang(id, body) => {
            match bindings.get(&id) {
                Some(true) => check(*body, bindings),
                Some(false) => Err(BindingError::Immutable(id.clone())),
                None => Err(BindingError::Free(id.clone())),
            }
        }

        Code::If(cond, then, else_) => {
            check(*cond, bindings)?;
            check(*then, bindings)?;
            check(*else_, bindings)
        }

        Code::Throw(thrown) => check(*thrown, bindings),

        Code::Try(try_, mutable, bound, catch) => {
            let _ = check(*try_, bindings)?;
            check(*catch, &bindings.update(bound.clone(), mutable))
        }

        Code::Lambda(args, body) => {
            let mut fn_bindings = bindings.clone();
            for (mutable, bound) in args.0.iter() {
                fn_bindings = fn_bindings.update((*bound).clone(), *mutable);
            }
            check(*body, &fn_bindings)
        }
    }
}
