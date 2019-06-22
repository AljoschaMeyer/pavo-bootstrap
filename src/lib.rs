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

use check::StaticError;
use context::Context;
use expand::ExpandError;
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

    fn assert_throw(src: &str, expected: Value) {
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
    }

    // ## Evaluation

    #[test]
    fn test_evaluation_order() {
        assert_throw("@{(sf-throw :b) (sf-throw :a)}", Value::kw_str("a"));
        assert_throw("{:b (sf-throw 1), :a (sf-throw 0)}", Value::int(0));
        assert_throw("{(sf-throw :b) 42, (sf-throw :a) 42}", Value::kw_str("a"));
    }

    // ### Special Forms

    #[test]
    fn test_sf_quote() {
        assert_ok("(sf-quote 42)", Value::int(42));
        assert_ok("(sf-quote foo)", Value::id_str("foo"));
        assert_ok("(sf-quote ())", Value::app_from_vec(vec![]));
    }

    #[test]
    fn test_sf_do() {
        assert_ok("(sf-do)", Value::nil());
        assert_ok("(sf-do 1)", Value::int(1));
        assert_ok("(sf-do 1 2 3)", Value::int(3));
    }

    #[test]
    fn test_sf_if() {
        assert_ok("(sf-if true :then :else)", Value::kw_str("then"));
        assert_ok("(sf-if 0 :then :else)", Value::kw_str("then"));
        assert_ok("(sf-if [] :then :else)", Value::kw_str("then"));
        assert_ok("(sf-if (sf-quote ()) :then :else)", Value::kw_str("then"));
        assert_ok("(sf-if nil :then :else)", Value::kw_str("else"));
        assert_ok("(sf-if false :then :else)", Value::kw_str("else"));
    }

    #[test]
    fn test_sf_set_bang() {
        assert_ok("((sf-lambda [:mut a] (sf-do (sf-set! a 42) a)) 17)", Value::int(42));
        assert_ok("((sf-lambda [:mut a] (sf-set! a 42)) 17)", Value::nil());
    }

    #[test]
    fn test_sf_throw() {
        assert_throw("(sf-throw 0)", Value::int(0));
        assert_throw("(sf-do 0 (sf-throw 1) (sf-throw 2) 3)", Value::int(1));
        assert_throw("(sf-if (sf-throw 0) (sf-throw 1) (sf-throw 2))", Value::int(0));
    }

    #[test]
    fn test_sf_try() {
        assert_ok("(sf-try 0 foo 1)", Value::int(0));
        assert_ok("(sf-try (sf-throw 0) foo 1)", Value::int(1));
        assert_ok("(sf-try (sf-throw 0) :mut foo (sf-set! foo 1))", Value::nil());
        assert_ok("(sf-try (sf-throw 0) foo foo)", Value::int(0));
        assert_throw("(sf-try (sf-throw 0) foo (sf-throw 1))", Value::int(1));
    }

    #[test]
    fn test_sf_lambda() {
        assert_ok("(typeof (sf-lambda foo nil))", Value::kw_str("function"));
        assert_ok(
            "((sf-lambda foo foo) 0 1 2)",
            Value::arr_from_vec(vec![Value::int(0), Value::int(1), Value::int(2)])
        );
        assert_ok("((sf-lambda :mut foo (sf-do (sf-set! foo 42) foo)) 0 1 2)", Value::int(42));

        assert_ok("(typeof (sf-lambda [] nil))", Value::kw_str("function"));
        assert_ok("((sf-lambda [] 42))", Value::int(42));
        assert_throw(
            "((sf-lambda [] 42) :an-argument)",
            execute("{:tag :err-num-args, :expected 0, :got 1}").unwrap()
        );
        assert_ok("((sf-lambda [a b] (int-add a b)) 1 2)", Value::int(3));
        assert_ok("((sf-lambda [a :mut b] (sf-do (sf-set! b 3) (int-add a b))) 1 2)", Value::int(4));
    }
}
