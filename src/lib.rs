#![feature(reverse_bits)]
#![feature(euclidean_division)]
use std::collections::HashMap;

use nom::types::CompleteStr;
use im_rc::OrdMap as ImOrdMap;

mod builtins;
mod check;
mod compile;
mod context;
mod env;
mod expand;
mod gc_foreign;
mod special_forms;
mod value;
mod read;
mod vm;

use check::{check_toplevel, StaticError};
use context::Context;
use expand::ExpandError;
use gc_foreign::OrdMap;
use value::{Id, Value};
use read::{read, ParseError};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ExecuteError {
    Parse(ParseError),
    E(E),
}

impl From<ParseError> for ExecuteError {
    fn from(err: ParseError) -> Self {
        ExecuteError::Parse(err)
    }
}

impl From<E> for ExecuteError {
    fn from(err: E) -> Self {
        ExecuteError::E(err)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum E {
    Expand(ExpandError),
    Static(StaticError),
    Eval(Value),
}

impl From<StaticError> for E {
    fn from(err: StaticError) -> Self {
        E::Static(err)
    }
}

impl From<ExpandError> for E {
    fn from(err: ExpandError) -> Self {
        E::Expand(err)
    }
}

impl From<Value> for E {
    fn from(err: Value) -> Self {
        E::Eval(err)
    }
}

pub fn exval(
    v: &Value,
    m_env: &HashMap<Id, Value>,
    macros: &ImOrdMap<Id, Value>,
    env: &HashMap<Id, Value>,
    cx: &mut Context,
) -> Result<Value, E> {
    let expanded = expand::expand(v, m_env, macros, cx)?;
    let c = compile::compile(&expanded, env)?;
    c.compute(gc_foreign::Vector(im_rc::Vector::new()), cx).map_err(|nay| E::Eval(nay))
}

pub fn execute(src: &str) -> Result<Value, ExecuteError> {
    let mut default_cx = Context::default();
    let default_env = env::default();
    let default_macros = ImOrdMap::new(); // TODO

    let v = read(CompleteStr(src))?;
    let yay = exval(&v, &default_env, &default_macros, &default_env, &mut default_cx)?;
    return Ok(yay);
}

#[cfg(test)]
mod tests {
    use super::{Value, execute, ExecuteError, E, StaticError, value::Id};
    use super::special_forms::{SpecialFormSyntaxError, FormType};

    fn assert_ok(src: &str, expected: Value) {
        match execute(src) {
            Err(err) => panic!("Unexpected error: {:?}", err),
            Ok(v) => assert_eq!(v, expected),
        }
    }

    fn assert_static_err(src: &str, expected: StaticError) {
        match execute(src) {
            Err(ExecuteError::E(E::Static(err))) => assert_eq!(err, expected),
            Err(ExecuteError::Parse(err)) => panic!("Unexpected parse error: {:?}", err),
            Err(other) => panic!("Expected a static error, got another error instead: {:?}"),
            Ok(v) => panic!("Expected a static error, but it evaluated: {:?}", v),
        }
    }

    fn assert_runtime_error(src: &str, expected: Value) {
        match execute(src) {
            Err(ExecuteError::E(E::Eval(err))) => assert_eq!(err, expected),
            Err(err) => panic!("Unexpected non-runtime error: {:?}", err),
            Ok(v) => panic!("Expected a runtime error, but got value: {:?}", v),
        }
    }

    fn assert_any_runtime_error(src: &str) {
        match execute(src) {
            Err(ExecuteError::E(E::Eval(err))) => {}
            Err(err) => panic!("Unexpected non-runtime error: {:?}", err),
            Ok(v) => panic!("Expected a runtime error, but got value: {:?}", v),
        }
    }

    #[test]
    fn test_nil() {
        assert_ok("nil", Value::nil());
        assert_ok(" nil ", Value::nil());
        assert_ok("# com#ment\n nil #this comment ends with eof", Value::nil());
        assert_ok("nil#", Value::nil());

        assert_ok("(typeof nil)", Value::kw_str("nil"));
    }

    #[test]
    fn test_apply() {
        assert_ok("(apply typeof [nil, false])", Value::kw_str("nil"));
        assert_ok("(apply typeof [false])", Value::kw_str("bool"));

        assert_any_runtime_error("(apply true [])");
        assert_any_runtime_error("(apply typeof true)");
        assert_any_runtime_error("(apply typeof)");
        assert_any_runtime_error("(apply)");

        assert_any_runtime_error("()");
    }

    #[test]
    fn test_sf_do() {
        assert_ok("(sf-do)", Value::nil());
        assert_ok("(sf-do false)", Value::bool_(false));
        assert_ok("(sf-do true false)", Value::bool_(false));
        assert_ok("(sf-do true true false)", Value::bool_(false));
    }

    #[test]
    fn test_sf_quote() {
        assert_ok("(sf-quote true)", Value::bool_(true));
        assert_ok("(sf-quote x)", Value::id_str("x"));
        assert_ok("(sf-quote sf-quote)", Value::id_str("sf-quote"));
        assert_ok("(sf-quote typeof)", Value::id_str("typeof"));
        assert_ok("(sf-quote ())", Value::app_from_vec(vec![]));
        assert_ok("(sf-quote (typeof nil))", Value::app_from_vec(vec![
                Value::id_str("typeof"),
                Value::nil(),
            ]));

        assert_static_err("(sf-quote)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Quote, 1)
        ));
        assert_static_err("(sf-quote x y)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Quote, 3)
        ));
    }

    #[test]
    fn test_sf_set_bang() {
        assert_ok("(sf-letfn (foo :mut x (sf-set! x false)) (foo true))", Value::nil());
        assert_ok("(sf-letfn
            (foo :mut x (sf-do
                    (sf-set! x false)
                    x
                ))
            (foo true)
        )", Value::bool_(false));

        assert_static_err("(sf-set! true true)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::SetBangId(Value::bool_(true))
        ));
        assert_static_err("(sf-set! x true)", StaticError::Free(Id::user("x")));

        assert_static_err("(sf-letfn (foo x (sf-set! x false)) (foo true))", StaticError::Immutable(Id::user("x")));

        assert_static_err("(sf-set!)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::SetBang, 1)
        ));
        assert_static_err("(sf-set! x)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::SetBang, 2)
        ));
        assert_static_err("(sf-set! x true nil)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::SetBang, 4)
        ));
    }

    #[test]
    fn test_sf_if() {
        assert_ok("(sf-if 42 true false)", Value::bool_(true));
        assert_ok("(sf-if false true false)", Value::bool_(false));
        assert_ok("(sf-if nil true false)", Value::bool_(false));

        assert_static_err("(sf-if)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::If, 1)
        ));
        assert_static_err("(sf-if true)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::If, 2)
        ));
        assert_static_err("(sf-if true true)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::If, 3)
        ));
        assert_static_err("(sf-if true true true true)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::If, 5)
        ));
    }

    #[test]
    fn test_sf_throw() {
        assert_runtime_error("(sf-throw true)", Value::bool_(true));

        assert_static_err("(sf-throw)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Throw, 1)
        ));
        assert_static_err("(sf-throw true true)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Throw, 3)
        ));
    }

    #[test]
    fn test_sf_try() {
        assert_ok("(sf-try true x x)", Value::bool_(true));
        assert_ok("(sf-try (sf-throw true) x x)", Value::bool_(true));
        assert_ok("(sf-try (sf-throw true) :mut x x)", Value::bool_(true));
        assert_ok("(sf-try (sf-throw true) :mut x (sf-do (sf-set! x false) x))", Value::bool_(false));

        assert_static_err("(sf-try)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Try, 1)
        ));
        assert_static_err("(sf-try true)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Try, 2)
        ));
        assert_static_err("(sf-try true x)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Try, 3)
        ));
        assert_static_err("(sf-try true x x x)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Try, 5)
        ));
        assert_static_err("(sf-try false true false)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Id(FormType::Try, Value::bool_(true))
        ));
    }

    // TODO test sf-lambda, and use it in other tests instead of sf-letfn

    #[test]
    fn test_sf_letfn() {
        assert_ok("(sf-letfn (foo x (bar (arr-get x 0))) (bar x (typeof (arr-get x 0))) (foo nil))", Value::kw_str("nil"));
        assert_ok("(sf-letfn (foo x (bar (arr-get x 0))) (bar x (typeof (arr-get x 0))) (foo 42))", Value::kw_str("int"));
    }

    #[test]
    fn test_tco() {
        assert_ok("(sf-letfn
            (even? n (sf-if (= (arr-get n 0) 0) true (odd? (int-sub (arr-get n 0) 1))))
            (odd? n (sf-if (= (arr-get n 0) 0) false (even? (int-sub (arr-get n 0) 1))))
            (even? 9999)
        )", Value::bool_(false));
    }

    // #[test]
    // fn test_toplevel_values() {
    //     // TODO test all the stuff from the reference docs. For now, this is just to check that particular stuff works
    //     assert_ok("(sf-do
    //         (assert-eq (arr-get [] 0 nil) nil)
    //     )", Value::nil());
    // }

    // use im_rc::Vector;
    // #[test]
    // fn test_name() {
    //     println!("{:?}", Vector::unit(42).split_at(1));
    // }
}
