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

    fn assert_any_parse_error(src: &str) {
        match execute(src) {
            Err(ExecuteError::Parse(err)) => {},
            Err(err) => panic!("Unexpected non-parse error: {:?}", err),
            Ok(v) => panic!("Expected a syntax error, but got value: {:?}", v),
        }
    }

    // ## Syntax

    #[test]
    fn test_syntax() {
        assert_ok("nil", Value::nil());
        assert_ok(" nil", Value::nil());
        assert_ok("nil ", Value::nil());
        assert_ok(" nil ", Value::nil());
        assert_ok("# com#ment\n nil #this comment ends with eof", Value::nil());
        assert_ok("nil#", Value::nil());

        assert_ok("true", Value::bool_(true));
        assert_ok("false", Value::bool_(false));

        assert_ok("(sf-quote =P)", Value::id_str("=P"));
        assert_ok("(sf-quote !*+-_?~<>=/\\&|abcdefghijklmnopqrsstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ)", Value::id_str("!*+-_?~<>=/\\&|abcdefghijklmnopqrsstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"));
        assert_ok("(sf-quote abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefg)", Value::id_str("abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefg"));
        assert_any_parse_error("abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh");
        assert_any_parse_error("[abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh]");

        assert_ok(":!", Value::kw_str("!"));
        assert_ok(":nil", Value::kw_str("nil"));
        assert_ok(":!*+-_?~<>=/\\&|abcdefghijklmnopqrsstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ", Value::kw_str("!*+-_?~<>=/\\&|abcdefghijklmnopqrsstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"));
        assert_ok(":abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefg", Value::kw_str("abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefg"));
        assert_any_parse_error(":");
        assert_any_parse_error(":abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh");
        assert_any_parse_error("[:abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh]");

        assert_ok("0", Value::int(0));
        assert_ok("+0", Value::int(0));
        assert_ok("-0", Value::int(0));
        assert_ok("00", Value::int(0));
        assert_ok("01", Value::int(1));
        assert_ok("-9223372036854775808", Value::int(-9223372036854775808));
        assert_any_parse_error("-9223372036854775809");
        assert_ok("9223372036854775807", Value::int(9223372036854775807));
        assert_any_parse_error("9223372036854775808");
        assert_ok("0xa", Value::int(10));
        assert_ok("-0xF", Value::int(-15));

        assert_ok("0.0", Value::float(0.0));
        assert_ok("+0.0", Value::float(0.0));
        assert_ok("-0.0", Value::float(0.0));
        assert_ok("00.00", Value::float(0.0));
        assert_ok("-0.3e2", Value::float(-30.0));
        assert_ok("0.3E+2", Value::float(30.0));
        assert_ok("10.0e-2", Value::float(0.1));
        assert_ok("0.0e99999999", Value::float(0.0));
        assert_ok("9007199254740992.0", Value::float(9007199254740993.0));
        assert_ok("9007199254740993.0", Value::float(9007199254740992.0));
        assert_any_parse_error("-999E999");
        assert_any_parse_error("999E999");

        assert_ok("'a'", Value::char_('a'));
        assert_ok("'⚗'", Value::char_('⚗'));
        assert_ok("'\\{61}'", Value::char_('a'));
        assert_ok("'\\{000061}'", Value::char_('a'));
        assert_ok("'\\{2697}'", Value::char_('⚗'));
        assert_ok("'\\{10FFFF}'", Value::char_('\u{10FFFF}'));
        assert_ok("'\"'", Value::char_('"'));
        assert_ok("'\\''", Value::char_('\''));
        assert_ok("'\\\\'", Value::char_('\\'));
        assert_ok("'\\t'", Value::char_('\t'));
        assert_ok("'\\n'", Value::char_('\n'));
        assert_any_parse_error("'\\{D800}'");
        assert_any_parse_error("'\\{DBFF}'");
        assert_any_parse_error("'\\{DC00}'");
        assert_any_parse_error("'\\{DFFF}'");
        assert_any_parse_error("'\\{110000}'");
        assert_any_parse_error("'\\{}'");
        assert_any_parse_error("'\\{1234567}'");
        assert_any_parse_error("'''");
        assert_any_parse_error("'\\'");
        assert_any_parse_error("'\\r'");

        assert_ok("\"\"", Value::string_from_str(""));
        assert_ok("\"a\"", Value::string_from_str("a"));
        assert_ok("\"abc\"", Value::string_from_str("abc"));
        assert_ok("\"⚗\"", Value::string_from_str("⚗"));
        assert_ok("\"⚗\\{10FFFF}\\{10FFFF} \\\\foo\"", Value::string_from_str("⚗\u{10FFFF}\u{10FFFF} \\foo"));
        assert_ok("\"\\{61}\"", Value::string_from_str("a"));
        assert_ok("\"\\{000061}\"", Value::string_from_str("a"));
        assert_ok("\"\\{2697}\"", Value::string_from_str("⚗"));
        assert_ok("\"\\{10FFFF}\"", Value::string_from_str("\u{10FFFF}"));
        assert_ok("\"'\"", Value::string_from_str("'"));
        assert_ok("\"\\\"\"", Value::string_from_str("\""));
        assert_ok("\"\\\\\"", Value::string_from_str("\\"));
        assert_ok("\"\\t\"", Value::string_from_str("\t"));
        assert_ok("\"\\n\"", Value::string_from_str("\n"));
        assert_any_parse_error("\"\\{D800}\"");
        assert_any_parse_error("\"\\{DBFF}\"");
        assert_any_parse_error("\"\\{DC00}\"");
        assert_any_parse_error("\"\\{DFFF}\"");
        assert_any_parse_error("\"\\{110000}\"");
        assert_any_parse_error("\"\\{}\"");
        assert_any_parse_error("\"\\{1234567}\"");
        assert_any_parse_error("\"\"\"");
        assert_any_parse_error("\"\\\"");
        assert_any_parse_error("\"\\r\"");

        assert_ok("@[]", Value::bytes_from_vec(vec![]));
        assert_ok("@[0]", Value::bytes_from_vec(vec![0]));
        assert_ok("@[0,0]", Value::bytes_from_vec(vec![0, 0]));
        assert_ok("@[0xF]", Value::bytes_from_vec(vec![15]));
        assert_ok("@[   ,, 0xfE   ]", Value::bytes_from_vec(vec![254]));
        assert_ok("@[0, 001, 255]", Value::bytes_from_vec(vec![0, 1, 255]));
        assert_ok("@[1 0x1]", Value::bytes_from_vec(vec![1, 1]));
        assert_any_parse_error("@[1111]");
        assert_any_parse_error("@[256]");
        assert_any_parse_error("@[0x]");
        assert_any_parse_error("@[0xddd]");
        assert_any_parse_error("@[10x1]");

        assert_ok("[]", Value::arr_from_vec(vec![]));
        assert_ok("[0]", Value::arr_from_vec(vec![Value::int(0)]));
        assert_ok("[0,1]", Value::arr_from_vec(vec![Value::int(0), Value::int(1)]));
        assert_ok("[ 0, 1  ,,2 ]", Value::arr_from_vec(vec![Value::int(0), Value::int(1), Value::int(2)]));
        assert_ok("[[0],1,]", Value::arr_from_vec(vec![Value::arr_from_vec(vec![Value::int(0)]), Value::int(1)]));
        assert_ok("[1 :a]", Value::arr_from_vec(vec![Value::int(1), Value::kw_str("a")]));
        assert_ok("[[] []]", Value::arr_from_vec(vec![Value::arr_from_vec(vec![]), Value::arr_from_vec(vec![])]));
        assert_any_parse_error("[1a]");
        assert_any_parse_error("[1:a]");
        assert_any_parse_error("[[][]]");

        assert_ok("(sf-quote ())", Value::app_from_vec(vec![]));
        assert_ok("(sf-quote (0))", Value::app_from_vec(vec![Value::int(0)]));
        assert_ok("(sf-quote (0,1))", Value::app_from_vec(vec![Value::int(0), Value::int(1)]));
        assert_ok("(sf-quote ( 0, 1  ,,2 ))", Value::app_from_vec(vec![Value::int(0), Value::int(1), Value::int(2)]));
        assert_ok("(sf-quote ((0),1,))", Value::app_from_vec(vec![Value::app_from_vec(vec![Value::int(0)]), Value::int(1)]));
        assert_ok("(sf-quote (1 :a))", Value::app_from_vec(vec![Value::int(1), Value::kw_str("a")]));
        assert_ok("(sf-quote (() ()))", Value::app_from_vec(vec![Value::app_from_vec(vec![]), Value::app_from_vec(vec![])]));
        assert_any_parse_error("(1a)");
        assert_any_parse_error("(1:a)");
        assert_any_parse_error("(()())");

        assert_ok("@{}", Value::set_from_vec(vec![]));
        assert_ok("@{0}", Value::set_from_vec(vec![Value::int(0)]));
        assert_ok("@{0,1}", Value::set_from_vec(vec![Value::int(0), Value::int(1)]));
        assert_ok("@{1,0}", Value::set_from_vec(vec![Value::int(0), Value::int(1)]));
        assert_ok("@{ 0, 1  ,,2 }", Value::set_from_vec(vec![Value::int(0), Value::int(1), Value::int(2)]));
        assert_ok("@{@{0},1,}", Value::set_from_vec(vec![Value::set_from_vec(vec![Value::int(0)]), Value::int(1)]));
        assert_ok("@{1 :a}", Value::set_from_vec(vec![Value::int(1), Value::kw_str("a")]));
        assert_ok("@{@{} @{}}", Value::set_from_vec(vec![Value::set_from_vec(vec![]), Value::set_from_vec(vec![])]));
        assert_ok("@{0 0}", Value::set_from_vec(vec![Value::int(0)]));
        assert_ok("@{0 0x0}", Value::set_from_vec(vec![Value::int(0)]));
        assert_any_parse_error("@{1a}");
        assert_any_parse_error("@{1:a}");
        assert_any_parse_error("@{@{}@{}}");

        assert_ok("{}", Value::map_from_vec(vec![]));
        assert_ok("{0 0}", Value::map_from_vec(vec![(Value::int(0), Value::int(0))]));
        assert_ok("{ 0,1 ,2 3 }", Value::map_from_vec(vec![(Value::int(0), Value::int(1)), (Value::int(2), Value::int(3))]));
        assert_ok("{2 3 0 1}", Value::map_from_vec(vec![(Value::int(0), Value::int(1)), (Value::int(2), Value::int(3))]));
        assert_ok("{0 1 0 2 1 3 0 4}", Value::map_from_vec(vec![(Value::int(0), Value::int(4)), (Value::int(1), Value::int(3))]));
        assert_any_parse_error("{1a}");
        assert_any_parse_error("{1:a}");
        assert_any_parse_error("{{}{}}");
        assert_any_parse_error("{1}");
        assert_any_parse_error("{1 2 3}");

        assert_ok("(sf-quote $a)", Value::app_from_vec(vec![Value::id_str("quote"), Value::id_str("a")]));
        assert_ok("(sf-quote `a)", Value::app_from_vec(vec![Value::id_str("quasiquote"), Value::id_str("a")]));
        assert_ok("(sf-quote ;a)", Value::app_from_vec(vec![Value::kw_str("unquote"), Value::id_str("a")]));
        assert_ok("(sf-quote %a)", Value::app_from_vec(vec![Value::kw_str("unquote-splice"), Value::id_str("a")]));
        assert_ok("(sf-quote @a)", Value::app_from_vec(vec![Value::kw_str("fresh-name"), Value::id_str("a")]));
        assert_ok("(sf-quote $$a)", Value::app_from_vec(vec![Value::id_str("quote"), Value::app_from_vec(vec![Value::id_str("quote"), Value::id_str("a")])]));
        assert_any_parse_error("$");
        assert_any_parse_error("`");
        assert_any_parse_error(";");
        assert_any_parse_error("%");
        assert_any_parse_error("@");
        assert_any_parse_error("$ a");
        assert_any_parse_error("` a");
        assert_any_parse_error("; a");
        assert_any_parse_error("% a");
        assert_any_parse_error("@ a");
        assert_any_parse_error("@0");
        assert_any_parse_error("@:a");
        assert_any_parse_error("@nil");
        assert_any_parse_error("@true");
        assert_any_parse_error("@false");
        assert_any_parse_error("@0a");
    }

    // ## Evaluation

    #[test]
    fn test_evaluation_order() {
        assert_throw("[(sf-throw :b) (sf-throw :a)]", Value::kw_str("b"));
        assert_throw("@{(sf-throw :b) (sf-throw :a)}", Value::kw_str("a"));
        assert_throw("{:b (sf-throw 1), :a (sf-throw 0)}", Value::int(0));
        assert_throw("{(sf-throw :b) 42, (sf-throw :a) 42}", Value::kw_str("a"));
        assert_throw("((sf-throw :b) (sf-throw :a))", Value::kw_str("b"));
    }

    #[test]
    fn test_application_errors() {
        assert_throw("()", execute("{:tag :err-lookup :got 0}").unwrap());
        assert_throw("(42)", execute("{:tag :err-type, :expected :function, :got :int}").unwrap());
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
