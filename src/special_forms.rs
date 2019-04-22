// Static typing for special forms to help implementing them. This doesn't actually *do*
// anything, its just a convenient/checked way of accessing special forms. The first
// attempt at implementing pavo without this layer very quickly turned into spaghetti.

use crate::gc_foreign::{Vector, OrdSet};
use crate::value::{Value, Atomic, Id};

pub enum SpecialForm<'a> {
    Quote(&'a Value),
    Do(Vec<&'a Value>),
    If(Vec<(&'a Value, &'a Value)>, Option<&'a Value>),
}

pub enum SpecialFormSyntaxError {
    QuoteArity { arity: usize},
    Pattern(PatternError),
}

pub enum Pattern<'a> {
    Id(&'a Id),
    Atomic(&'a Atomic),
    Arr(Vec<Pattern<'a>>),
}

pub enum PatternError {
    NotAPattern(Value),
}

pub fn special<'a>(
    v: &'a Vector<Value>,
    forms: OrdSet<Value>, // Only recognize a form if its keyword is in this set.
) -> Result<Option<SpecialForm<'a>>, SpecialFormSyntaxError> {
    if v.0.len() == 0 {
        return Ok(None);
    }

    match v.0[0].as_id() {
        None => return Ok(None),
        Some(id) => match id.get_chars() {
            "quote" if forms.0.contains(&Value::kw_str("quote")) => {
                if v.0.len() == 2 {
                    return Ok(Some(SpecialForm::Quote(&v.0[1])));
                } else {
                    return Err(SpecialFormSyntaxError::QuoteArity { arity: v.0.len() });
                }
            }

            "do" if forms.0.contains(&Value::kw_str("do")) => {
                let mut do_stmts = Vec::with_capacity(v.0.len() - 1);

                for stmt in v.0.iter().skip(1) {
                    do_stmts.push(stmt);
                }

                return Ok(Some(SpecialForm::Do(do_stmts)));
            }

            "if" if forms.0.contains(&Value::kw_str("if")) => {
                let total = v.0.len();
                let mut pairs = Vec::with_capacity(total / 2);
                let mut i = 1;

                while i + 1 < total {
                    pairs.push((&v.0[i], &v.0[i + 1]));
                    i += 2;
                }

                let else_ = if total % 2 == 0 {
                    Some(&v.0[total - 1])
                } else {
                    None
                };
                return Ok(Some(SpecialForm::If(pairs, else_)));
            }

            _ => return Ok(None),
        }
    }
}

pub fn pattern<'a>(v: &'a Value) -> Result<Pattern<'a>, PatternError> {
    match v {
        Value::Id(id) => return Ok(Pattern::Id(id)),
        Value::Atomic(atomic) => return Ok(Pattern::Atomic(atomic)),
        Value::Arr(Vector(arr)) => {
            let mut inners = Vec::with_capacity(arr.len());

            for inner in arr.iter() {
                inners.push(pattern(inner)?);
            }

            return Ok(Pattern::Arr(inners));
        }
        _ => return Err(PatternError::NotAPattern(v.clone())),
    }
}
