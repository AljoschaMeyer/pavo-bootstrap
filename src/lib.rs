#![feature(reverse_bits)]
#![feature(euclidean_division)]

use nom::types::CompleteStr;

mod builtins;
mod check;
mod context;
mod env;
mod eval;
mod expand;
mod gc_foreign;
mod special_forms;
mod value;
mod read;
mod toplevel;

use check::{check, StaticError};
use context::Context;
use env::Env;
use eval::eval;
use value::Value;
use read::{read, ParseError};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Error {
    Parse(ParseError),
    Static(StaticError),
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::Parse(err)
    }
}

impl From<StaticError> for Error {
    fn from(err: StaticError) -> Self {
        Error::Static(err)
    }
}

pub fn execute(src: &str) -> Result<Result<Value, Value>, Error> {
    let mut default_cx = Context::default();
    let default_env = Env::default(&mut default_cx);

    let v = read(CompleteStr(src))?;
    check(&v, &default_env)?;
    Ok(eval(v, default_env, &mut default_cx))
}

#[cfg(test)]
mod tests {
    use super::{Value, execute, Error, StaticError, value::Id};
    use super::special_forms::{SpecialFormSyntaxError, FormType};

    fn assert_ok(src: &str, expected: Value) {
        match execute(src) {
            Err(err) => panic!("Unexpected static error: {:?}", err),
            Ok(Err(err)) => panic!("Unexpected runtime error: {:?}", err),
            Ok(Ok(v)) => assert_eq!(v, expected),
        }
    }

    fn assert_static_err(src: &str, expected: StaticError) {
        match execute(src) {
            Err(Error::Static(err)) => assert_eq!(err, expected),
            Err(Error::Parse(err)) => panic!("Unexpected parse error: {:?}", err),
            Ok(Err(err)) => panic!("Unexpected runtime error: {:?}", err),
            Ok(Ok(v)) => panic!("Expected a static error, but it evaluated: {:?}", v),
        }
    }

    fn assert_runtime_error(src: &str, expected: Value) {
        match execute(src) {
            Err(err) => panic!("Unexpected static error: {:?}", err),
            Ok(Err(err)) => assert_eq!(err, expected),
            Ok(Ok(v)) => panic!("Expected a runtime error, but got value: {:?}", v),
        }
    }

    fn assert_any_runtime_error(src: &str) {
        match execute(src) {
            Err(err) => panic!("Unexpected static error: {:?}", err),
            Ok(Err(err)) => {}
            Ok(Ok(v)) => panic!("Expected runtime error, but got value: {:?}", v),
        }
    }

    #[test]
    fn test_nil() {
        assert_ok("nil", Value::nil());
        assert_ok(" nil ", Value::nil());
        assert_ok("# com#ment\n nil #this comment ends with eof", Value::nil());
        assert_ok("nil#", Value::nil());

        assert_ok("(nil? nil)", Value::bool_(true));
        assert_ok("(nil? false)", Value::bool_(false));
    }

    #[test]
    fn test_apply() {
        assert_ok("(apply nil? [nil, false])", Value::bool_(true));
        assert_ok("(apply nil? [false])", Value::bool_(false));

        assert_any_runtime_error("(apply true [])");
        assert_any_runtime_error("(apply nil? true)");
        assert_any_runtime_error("(apply nil?)");
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
        assert_ok("(sf-quote nil?)", Value::id_str("nil?"));
        assert_ok("(sf-quote ())", Value::app_from_vec(vec![]));
        assert_ok("(sf-quote (nil? nil))", Value::app_from_vec(vec![
                Value::id_str("nil?"),
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
    fn test_sf_let() {
        assert_ok("(sf-let x true x)", Value::bool_(true));
        assert_ok("(sf-let :mut x true x)", Value::bool_(true));
        assert_ok("(sf-let x false (sf-let x true x))", Value::bool_(true));

        assert_static_err("(sf-let x true y)", StaticError::Free(Id::user("y")));

        assert_static_err("(sf-let)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Let, 1)
        ));
        assert_static_err("(sf-let x)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Let, 2)
        ));
        assert_static_err("(sf-let x true)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Let, 3)
        ));
        assert_static_err("(sf-let x true x x)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Let, 5)
        ));
        assert_static_err("(sf-let true false false)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Id(FormType::Let, Value::bool_(true))
        ));
    }

    #[test]
    fn test_sf_set_bang() {
        assert_ok("(sf-let :mut x true (sf-set! x false))", Value::nil());
        assert_ok("(sf-let :mut x true (sf-do (sf-set! x false) x))", Value::bool_(false));

        assert_static_err("(sf-set! true true)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::SetBangId(Value::bool_(true))
        ));
        assert_static_err("(sf-set! x true)", StaticError::Free(Id::user("x")));
        assert_static_err("(sf-let x true (sf-set! x false))", StaticError::Immutable(Id::user("x")));

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

    #[test]
    fn test_sf_lambda() {
        assert_ok("((sf-lambda x (nil? (arr-get x 0))) nil)", Value::bool_(true));
        assert_ok("((sf-lambda x (nil? (arr-get x 0))) false)", Value::bool_(false));
        assert_ok("((sf-lambda :mut x (sf-do (sf-set! x nil) (nil? x))) false)", Value::bool_(true));

        assert_static_err("(sf-lambda)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Lambda, 1)
        ));
        assert_static_err("(sf-lambda x)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Lambda, 2)
        ));
        assert_static_err("(sf-lambda x x x)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Lambda, 4)
        ));
        assert_static_err("(sf-lambda :mut x x x)", StaticError::SpecialFormSyntax(
            SpecialFormSyntaxError::Arity(FormType::Lambda, 5)
        ));
    }

    #[test]
    fn test_sf_letfn() {
        assert_ok("(sf-letfn (foo x (bar (arr-get x 0))) (bar x (nil? (arr-get x 0))) (foo nil))", Value::bool_(true));
        assert_ok("(sf-letfn (foo x (bar (arr-get x 0))) (bar x (nil? (arr-get x 0))) (foo 42))", Value::bool_(false));
    }

    #[test]
    fn test_tco() {
        assert_ok("(sf-letfn
            (even? n (sf-if (= (arr-get n 0) 0) true (odd? (int-sub (arr-get n 0) 1))))
            (odd? n (sf-if (= (arr-get n 0) 0) false (even? (int-sub (arr-get n 0) 1))))
            (even? 9999)
        )", Value::bool_(false));
    }

    #[test]
    fn test_toplevel_values() {
        // TODO test all the stuff from the reference docs. For now, this is just to check that particular stuff works
        assert_ok("(sf-do
            (assert-eq (arr-get [] 0 nil) nil)
        )", Value::nil());
    }

    use im_rc::Vector;
    #[test]
    fn test_name() {
        println!("{:?}", Vector::unit(42).split_at(1));
    }
}
