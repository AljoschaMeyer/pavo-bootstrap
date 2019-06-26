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
mod macros;
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
    // let mut tmp = String::new();
    // builtins::write_(&expanded, &mut tmp);
    // println!("{}\n", tmp);
    let c = compile::compile(&expanded, env)?;
    c.compute(gc_foreign::Vector(im_rc::Vector::new()), cx).map_err(|nay| E::Eval(nay))
}

pub fn execute(src: &str) -> Result<Value, ExecuteError> {
    let mut default_cx = Context::default();
    let default_env = env::default();
    let default_macros = macros::default();

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

    fn assert_any_static_error(src: &str) {
        match execute(src) {
            Err(ExecuteError::E(E::Static(err))) => {},
            Err(err) => panic!("Unexpected non-static error: {:?}", err),
            Ok(v) => panic!("Expected a static error, but got value: {:?}", v),
        }
    }

    fn test_example(src: &str) {
        let mut src_in_context = "
        (macro assert-throw (sf-lambda [try exception]
            (app-insert
                (app-insert
                    (sf-quote (sf-try e))
                    1
                    (app-insert
                        (sf-quote (sf-do (sf-throw :assert-throw)))
                        1
                        try
                    )
                )
                3
                (app-insert
                    (sf-quote (sf-if nil (sf-throw :assert-eq-throw)))
                    1
                    (app-insert
                        (sf-quote (= e))
                        2
                        exception
                    )
                )
            )
        )

        ((sf-lambda [assert assert-not assert-eq] (sf-do ".to_string();
        src_in_context.push_str(src);
        src_in_context.push_str("
            ))
        (sf-lambda [v] (sf-if (= v true) nil (sf-throw :assert)))
        (sf-lambda [v] (sf-if (= v false) nil (sf-throw :assert-not)))
        (sf-lambda [v w] (sf-if (= v w) nil (sf-throw v)))
        )

        )");

        assert_ok(&src_in_context, Value::nil())
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

    // ## Static Checks

    // ### Special Form Syntax

    #[test]
    fn test_static_sf_quote() {
        assert_any_static_error("(sf-quote)");
        assert_any_static_error("(sf-quote foo bar)");
    }

    #[test]
    fn test_static_sf_do() {
        // no-op, nothing to test here
    }

    #[test]
    fn test_static_sf_if() {
        assert_any_static_error("(sf-if)");
        assert_any_static_error("(sf-if :cond)");
        assert_any_static_error("(sf-if :cond :then)");
        assert_any_static_error("(sf-if :cond :then :else :wut?)");
    }

    #[test]
    fn test_static_sf_set_bang() {
        assert_any_static_error("(sf-set! 42 43)");
        assert_any_static_error("(sf-set!)");
        assert_any_static_error("(sf-set! a)");
        assert_any_static_error("(sf-set! a 42 foo)");
    }

    #[test]
    fn test_static_sf_throw() {
        assert_any_static_error("(sf-throw)");
        assert_any_static_error("(sf-throw foo bar)");
    }

    #[test]
    fn test_static_sf_try() {
        assert_any_static_error("(sf-try 0 1 2)");
        assert_any_static_error("(sf-try 0 (:mut 1) 2)");
        assert_any_static_error("(sf-try 0 (:foo a) 2)");
        assert_any_static_error("(sf-try 0 (:mut a))");
        assert_any_static_error("(sf-try)");
        assert_any_static_error("(sf-try 0)");
        assert_any_static_error("(sf-try 0 a)");
        assert_any_static_error("(sf-try 0 a 1 2)");
    }

    #[test]
    fn test_static_sf_lambda() {
        assert_any_static_error("(sf-lambda [a a] 0)");
        assert_any_static_error("(sf-lambda 0 1)");
        assert_any_static_error("(sf-lambda (:mut 0) 1)");
        assert_any_static_error("(sf-lambda [0] 1)");
        assert_any_static_error("(sf-lambda [(:mut)] 0)");
        assert_any_static_error("(sf-lambda [(:mut a b)] 0)");
        assert_any_static_error("(sf-lambda [(a :mut)] 0)");
        assert_any_static_error("(sf-lambda)");
        assert_any_static_error("(sf-lambda a)");
        assert_any_static_error("(sf-lambda a 0 1)");
        assert_any_static_error("(sf-lambda :mut)");
        assert_any_static_error("(sf-lambda :mut a)");
        assert_any_static_error("(sf-lambda :mut a 0 1)");
        assert_any_static_error("(sf-lambda [])");
        assert_any_static_error("(sf-lambda [] 0 1)");
    }

    // ### Binding Correctness

    #[test]
    fn test_static_bindings() {
        assert_ok("(sf-quote a)", Value::id_str("a"));
        assert_ok("(sf-try 0 a a)", Value::int(0));
        assert_ok("(sf-try 0 (:mut a) (sf-set! a 42))", Value::int(0));
        assert_ok("((sf-lambda [a] a) 0)", Value::int(0));
        assert_ok("((sf-lambda (:mut a) (sf-set! a 42)) 0)", Value::nil());
        assert_ok("(((sf-lambda a (sf-lambda (:mut a) (sf-set! a 0))) 0) 0)", Value::nil());
        assert_any_static_error("some-id");
        assert_any_static_error("[some-id]");
        assert_any_static_error("(sf-set! some-id 42)");
        assert_any_static_error("(sf-set! int-max-val 42)");
        assert_any_static_error("(sf-try 0 a (sf-set! a 42))");
        assert_any_static_error("(sf-lambda a (sf-set! a 42))");
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
        assert_ok("(sf-quote (sf-if))", Value::app_from_vec(vec![Value::id_str("sf-if")]));
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
        assert_ok("((sf-lambda [(:mut a)] (sf-do (sf-set! a 42) a)) 17)", Value::int(42));
        assert_ok("((sf-lambda [(:mut a)] (sf-set! a 42)) 17)", Value::nil());
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
        assert_ok("(sf-try (sf-throw 0) (:mut foo) (sf-set! foo 1))", Value::nil());
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
        assert_ok("((sf-lambda (:mut foo) (sf-do (sf-set! foo 42) foo)) 0 1 2)", Value::int(42));

        assert_ok("(typeof (sf-lambda [] nil))", Value::kw_str("function"));
        assert_ok("((sf-lambda [] 42))", Value::int(42));
        assert_throw(
            "((sf-lambda [] 42) :an-argument)",
            execute("{:tag :err-num-args, :expected 0, :got 1}").unwrap()
        );
        assert_ok("((sf-lambda [a b] (int-add a b)) 1 2)", Value::int(3));
        assert_ok("((sf-lambda [a (:mut b)] (sf-do (sf-set! b 3) (int-add a b))) 1 2)", Value::int(4));
    }

    // ## Toplevel Values

    #[test]
    fn test_function_argument_errors() {
        test_example("
        (assert-throw (bool-not) { :tag :err-num-args, :expected 1, :got 0 })
        nil

        #(assert-throw (bool-not) { :tag :err-num-args, :expected 1, :got 0 })
        #(assert-throw (bool-not 42 43) { :tag :err-num-args, :expected 1, :got 2 })
        #(assert-throw (bool-not 42) { :tag :err-type, :expected :bool, :got :int })
        #(assert-throw (int-pow-wrap :nope \"nope\") { :tag :err-type, :expected :int, :got :keyword})
        #(assert-throw (int-pow-wrap 2 :nope) { :tag :err-type, :expected :int, :got :keyword})
        #(assert-throw (int-pow-wrap 2 -2) { :tag :err-negative, :got -2})
        ");
    }

    #[test]
    fn test_toplevel_bool() {
        test_example("
        (assert (bool-not false))
        (assert-not (bool-not true))
        (assert-throw (bool-not 0) {
            :tag :err-type,
            :expected :bool,
            :got :int,
        })
        ");

        test_example("
        (assert-not (bool-and false false))
        (assert-not (bool-and false true))
        (assert-not (bool-and true false))
        (assert (bool-and true true))

        (assert-throw (bool-and false 0) {
            :tag :err-type,
            :expected :bool,
            :got :int,
        })
        ");

        test_example("
        (assert-not (bool-or false false))
        (assert (bool-or false true))
        (assert (bool-or true false))
        (assert (bool-or true true))

        (assert-throw (bool-or true 1) {
            :tag :err-type,
            :expected :bool,
            :got :int,
        })
        ");

        test_example("
        (assert (bool-if false false))
        (assert (bool-if false true))
        (assert-not (bool-if true false))
        (assert (bool-if true true))

        (assert-throw (bool-if false 1) {
            :tag :err-type,
            :expected :bool,
            :got :int,
        })
        ");

        test_example("
        (assert (bool-iff false false))
        (assert-not (bool-iff false true))
        (assert-not (bool-iff true false))
        (assert (bool-iff true true))

        (assert-throw (bool-iff false 1) {
            :tag :err-type,
            :expected :bool,
            :got :int,
        })
        ");

        test_example("
        (assert-not (bool-xor false false))
        (assert (bool-xor false true))
        (assert (bool-xor true false))
        (assert-not (bool-xor true true))

        (assert-throw (bool-xor false 1) {
            :tag :err-type,
            :expected :bool,
            :got :int,
        })
        ");
    }

    #[test]
    fn test_toplevel_int() {
        test_example("
        (assert-eq int-max-val 9223372036854775807)
        (assert-throw (int-add int-max-val 1) { :tag :err-wrap-int })
        ");

        test_example("
        (assert-eq int-min-val -9223372036854775808)
        (assert-throw (int-sub int-min-val 1) { :tag :err-wrap-int })
        ");

        test_example("(assert-eq (int-count-ones 126) 6)");

        test_example("(assert-eq (int-count-zeros 126) 58)");

        test_example("(assert-eq (int-leading-ones -4611686018427387904) 2)");

        test_example("(assert-eq (int-leading-zeros 13) 60)");

        test_example("(assert-eq (int-trailing-ones 3) 2)");

        test_example("(assert-eq (int-trailing-zeros 4) 2)");

        test_example("(assert-eq (int-rotate-left 0xaa00000000006e1 12) 0x6e10aa)");

        test_example("(assert-eq (int-rotate-right 0x6e10aa 12) 0xaa00000000006e1)");

        test_example("(assert-eq (int-reverse-bytes 0x1234567890123456) 0x5634129078563412)");

        test_example("(assert-eq (int-reverse-bits 0x1234567890123456) 0x6a2c48091e6a2c48)");

        test_example("
        (assert-eq (int-add 1 2) 3)
        (assert-eq (int-add 1 -2) -1)
        (assert-throw (int-add int-max-val 1) { :tag :err-wrap-int })
        ");

        test_example("
        (assert-eq (int-sub 1 2) -1)
        (assert-eq (int-sub 1 -2) 3)
        (assert-throw (int-sub int-min-val 1) { :tag :err-wrap-int })
        ");

        test_example("
        (assert-eq (int-mul 2 3) 6)
        (assert-eq (int-mul 2 -3) -6)
        (assert-throw (int-mul int-max-val 2) { :tag :err-wrap-int })
        ");

        test_example("
        (assert-eq (int-div 8 3) 2)
        (assert-eq (int-div -8 3) -3)
        (assert-throw (int-div int-min-val -1) { :tag :err-wrap-int })
        (assert-throw (int-div 1 0) { :tag :err-zero })
        ");

        test_example("
        (assert-eq (int-div-trunc 8 3) 2)
        (assert-eq (int-div-trunc -8 3) -2)
        (assert-throw (int-div-trunc int-min-val -1) { :tag :err-wrap-int })
        (assert-throw (int-div-trunc 1 0) { :tag :err-zero })
        ");

        test_example("
        (assert-eq (int-mod 8 3) 2)
        (assert-eq (int-mod -8 3) 1)
        (assert-throw (int-mod int-min-val -1) { :tag :err-wrap-int })
        (assert-throw (int-mod 1 0) { :tag :err-zero })
        ");

        test_example("
        (assert-eq (int-mod-trunc 8 3) 2)
        (assert-eq (int-mod-trunc -8 3) -2)
        (assert-throw (int-mod-trunc int-min-val -1) { :tag :err-wrap-int })
        (assert-throw (int-mod-trunc 1 0) { :tag :err-zero })
        ");

        test_example("
        (assert-eq (int-neg 42) -42)
        (assert-eq (int-neg -42) 42)
        (assert-eq (int-neg 0) 0)
        (assert-throw (int-neg int-min-val) { :tag :err-wrap-int })
        ");

        test_example("
        (assert-eq (int-shl 5 1) 10)
        (assert-eq (int-shl 42 64) 0)
        ");

        test_example("
        (assert-eq (int-shr 5 1) 2)
        (assert-eq (int-shr 42 64) 0)
        ");

        test_example("
        (assert-eq (int-abs 42) 42)
        (assert-eq (int-abs -42) 42)
        (assert-eq (int-abs 0) 0)
        (assert-throw (int-abs int-min-val) { :tag :err-wrap-int })
        ");

        test_example("
        (assert-eq (int-pow 2 3) 8)
        (assert-eq (int-pow 2 0) 1)
        (assert-eq (int-pow 0 999) 0)
        (assert-eq (int-pow 1 999) 1)
        (assert-eq (int-pow -1 999) -1)
        (assert-throw (int-pow 99 99) { :tag :err-wrap-int })
        ");

        test_example("
        (assert-eq (int-add-sat 1 2) 3)
        (assert-eq (int-add-sat 1 -2) -1)
        (assert-eq (int-add-sat int-max-val 1) int-max-val)
        (assert-eq (int-add-sat int-min-val -1) int-min-val)
        ");

        test_example("
        (assert-eq (int-sub-sat 1 2) -1)
        (assert-eq (int-sub-sat 1 -2) 3)
        (assert-eq (int-sub-sat int-min-val 1) int-min-val)
        (assert-eq (int-sub-sat int-max-val -1) int-max-val)
        ");

        test_example("
        (assert-eq (int-mul-sat 2 3) 6)
        (assert-eq (int-mul-sat 2 -3) -6)
        (assert-eq (int-mul-sat int-max-val 2) int-max-val)
        (assert-eq (int-mul-sat int-min-val 2) int-min-val)
        ");

        test_example("
        (assert-eq (int-pow-sat 2 3) 8)
        (assert-eq (int-pow-sat 2 0) 1)
        (assert-eq (int-pow-sat 0 999) 0)
        (assert-eq (int-pow-sat 1 999) 1)
        (assert-eq (int-pow-sat -1 999) -1)
        (assert-eq (int-pow-sat 99 99) int-max-val)
        (assert-eq (int-pow-sat -99 99) int-min-val)
        ");

        test_example("
        (assert-eq (int-add-wrap 1 2) 3)
        (assert-eq (int-add-wrap int-max-val 1) int-min-val)
        (assert-eq (int-add-wrap int-min-val -1) int-max-val)
        ");

        test_example("
        (assert-eq (int-sub-wrap 1 2) -1)
        (assert-eq (int-sub-wrap int-min-val 1) int-max-val)
        (assert-eq (int-sub-wrap int-max-val -1) int-min-val)
        ");

        test_example("
        (assert-eq (int-mul-wrap 2 3) 6)
        (assert-eq (int-mul-wrap int-max-val 2) -2)
        (assert-eq (int-mul-wrap int-max-val -2) 2)
        #(assert-eq (int-mul-wrap int-min-val 2) 0)
        #(assert-eq (int-mul-wrap int-min-val -2) 0)
        ");

        test_example("
        (assert-eq (int-div-wrap 8 3) 2)
        (assert-eq (int-div-wrap -8 3) -3)
        (assert-eq (int-div-wrap int-min-val -1) int-min-val)
        (assert-throw (int-div-wrap 1 0) { :tag :err-zero })
        ");

        test_example("
        (assert-eq (int-div-trunc-wrap 8 3) 2)
        (assert-eq (int-div-trunc-wrap -8 3) -2)
        (assert-eq (int-div-trunc-wrap int-min-val -1) int-min-val)
        (assert-throw (int-div-trunc-wrap 1 0) { :tag :err-zero })
        ");

        test_example("
        (assert-eq (int-mod-wrap 8 3) 2)
        (assert-eq (int-mod-wrap -8 3) 1)
        (assert-eq (int-mod-wrap int-min-val -1) 0)
        (assert-throw (int-mod-wrap 1 0) { :tag :err-zero })
        ");

        test_example("
        (assert-eq (int-mod-trunc-wrap 8 3) 2)
        (assert-eq (int-mod-trunc-wrap -8 3) -2)
        (assert-eq (int-mod-trunc-wrap int-min-val -1) 0)
        (assert-throw (int-mod-trunc-wrap 1 0) { :tag :err-zero })
        ");

        test_example("
        (assert-eq (int-neg-wrap 42) -42)
        (assert-eq (int-neg-wrap -42) 42)
        (assert-eq (int-neg-wrap 0) 0)
        (assert-eq (int-neg-wrap int-min-val) int-min-val)
        ");

        test_example("
        (assert-eq (int-abs-wrap 42) 42)
        (assert-eq (int-abs-wrap -42) 42)
        (assert-eq (int-abs-wrap 0) 0)
        (assert-eq (int-abs-wrap int-min-val) int-min-val)
        ");

        test_example("
        (assert-eq (int-pow-wrap 2 3) 8)
        (assert-eq (int-pow-wrap 2 0) 1)
        (assert-eq (int-pow-wrap 0 999) 0)
        (assert-eq (int-pow-wrap 1 999) 1)
        (assert-eq (int-pow-wrap -1 999) -1)
        (assert-eq (int-pow-wrap 99 99) -7394533151961528133)
        (assert-throw (int-pow-wrap 2 -1) {:tag :err-negative :got -1 })
        ");

        test_example("
        (assert-eq (int-signum -42) -1)
        (assert-eq (int-signum 0) 0)
        (assert-eq (int-signum 42) 1)
        ");
    }

    #[test]
    fn test_toplevel_bytes() {
        test_example("
        (assert-eq (bytes-count @[]) 0)
        (assert-eq (bytes-count @[0]) 1)
        (assert-eq (bytes-count @[0, 1, 2]) 3)
        ");

        test_example("
        (assert-eq (bytes-get @[42] 0) 42)
        (assert-throw (bytes-get @[] 0) { :tag :err-lookup, :got 0})
        ");

        test_example("
        (assert-eq (bytes-insert @[0 1] 0 42) @[42 0 1])
        (assert-eq (bytes-insert @[0 1] 1 42) @[0 42 1])
        (assert-eq (bytes-insert @[0 1] 2 42) @[0 1 42])
        (assert-throw (bytes-insert @[0 1] 3 42) { :tag :err-lookup, :got 3})
        (assert-throw (bytes-insert @[] 0 256) { :tag :err-not-byte, :got 256})
        (assert-throw (bytes-insert @[] 0 :256) { :tag :err-type, :expected :int, :got :keyword})
        ");

        test_example("
        (assert-eq (bytes-remove @[0 1] 0) @[1])
        (assert-eq (bytes-remove @[0 1] 1) @[0])
        (assert-throw (bytes-remove @[0 1] 3) { :tag :err-lookup, :got 3})
        ");

        test_example("
        (assert-eq (bytes-update @[0 1] 0 42) @[42 1])
        (assert-eq (bytes-update @[0 1] 1 42) @[0 42])
        (assert-throw (bytes-update @[0 1] 2 42) { :tag :err-lookup, :got 2})
        (assert-throw (bytes-update @[0] 0 256) { :tag :err-not-byte, :got 256})
        ");

        test_example("
        (assert-eq (bytes-slice @[42 43] 1 1) @[])
        (assert-eq (bytes-slice @[42 43] 0 1) @[42])
        (assert-eq (bytes-slice @[42 43] 1 2) @[43])
        (assert-eq (bytes-slice @[42 43] 0 2) @[42 43])
        (assert-throw (bytes-slice @[] 0 1) { :tag :err-lookup, :got 1})
        (assert-throw (bytes-slice @[] 2 3) { :tag :err-lookup, :got 2})
        (assert-throw (bytes-slice @[0 1 2 3] 2 1) { :tag :err-lookup, :got 1})
        ");

        test_example("
        (assert-eq (bytes-splice @[0 1] 0 @[10 11]) @[10 11 0 1])
        (assert-eq (bytes-splice @[0 1] 1 @[10 11]) @[0 10 11 1])
        (assert-eq (bytes-splice @[0 1] 2 @[10 11]) @[0 1 10 11])
        (assert-throw (bytes-splice @[0 1] 3 @[10 11]) { :tag :err-lookup, :got 3})
        ");

        test_example("
        (assert-eq (bytes-concat @[0 1] @[2 3]) @[0 1 2 3])
        (assert-eq (bytes-concat @[] @[0 1]) @[0 1])
        (assert-eq (bytes-concat @[0 1] @[]) @[0 1])
        ");

        test_example("
        (let (:mut product) 1 (do
            (bytes-iter @[1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (bytes-iter @[1 2 3 4] (fn [elem] (sf-if
                    (= elem 3) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 2)
        ))
        (assert-throw (bytes-iter @[0 1] (fn [b] (throw b))) 0)
        ");

        test_example("
        (let (:mut product) 1 (do
            (bytes-iter-back @[1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (bytes-iter-back @[1 2 3 4] (fn [elem] (if
                    (= elem 3) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 4)
        ))
        (assert-throw (bytes-iter-back @[0 1] (fn [b] (throw b))) 1)
        ");
    }

    #[test]
    fn test_toplevel_char() {
        test_example("(assert-eq char-max-val '\\{10ffff}')");

        test_example("
        (assert-eq (int=>char 0x41) 'A')
        (assert-throw (int=>char 0x110000) { :tag :err-not-unicode-scalar, :got 0x110000})
        ");

        test_example("
        (assert (int=>char? 0x41))
        (assert-not (int=>char? 0x110000))
        ");

        test_example("(assert-eq (char->int 'A') 0x41)");
    }

    #[test]
    fn test_toplevel_string() {
        test_example(r#"
        (assert-eq (str-count "") 0)
        (assert-eq (str-count "a") 1)
        (assert-eq (str-count "⚗") 1)
        (assert-eq (str-count "abc") 3)
        "#);

        test_example(r#"
        (assert-eq (str-count-utf8 "") 0)
        (assert-eq (str-count-utf8 "a") 1)
        (assert-eq (str-count-utf8 "⚗") 3)
        (assert-eq (str-count-utf8 "abc") 3)
        "#);

        test_example(r#"
        (assert-eq (str-get "a" 0) 'a')
        (assert-eq (str-get "⚗b" 1) 'b')
        (assert-throw (str-get "" 0) { :tag :err-lookup, :got 0})
        "#);

        test_example(r#"
        (assert-eq (str-get-utf8 "a" 0) 97)
        (assert-eq (str-get-utf8 "⚗" 0) 226)
        (assert-eq (str-get-utf8 "⚗" 1) 154)
        (assert-eq (str-get-utf8 "⚗" 2) 151)
        (assert-throw (str-get-utf8 "" 0) { :tag :err-lookup, :got 0})
        "#);

        test_example(r#"
        (assert-eq (str-index-char->utf8 "a" 0) 0)
        (assert-eq (str-index-char->utf8 "ab" 1) 1)
        (assert-eq (str-index-char->utf8 "⚗b" 1) 3)
        (assert-throw (str-index-char->utf8 "" 0) { :tag :err-lookup, :got 0})
        "#);

        test_example(r#"
        (assert-eq (str-index-utf8->char "a" 0) 0)
        (assert-eq (str-index-utf8->char "ab" 1) 1)
        (assert-eq (str-index-utf8->char "⚗b" 1) 0)
        (assert-eq (str-index-utf8->char "⚗b" 2) 0)
        (assert-eq (str-index-utf8->char "⚗b" 3) 1)
        (assert-throw (str-index-char->utf8 "" 0) { :tag :err-lookup, :got 0})
        "#);

        test_example(r#"
        (assert-eq (str-insert "ab" 0 'z') "zab")
        (assert-eq (str-insert "ab" 1 'z') "azb")
        (assert-eq (str-insert "ab" 2 'z') "abz")
        (assert-throw (str-insert "ab" 3 'z') { :tag :err-lookup, :got 3})
        "#);

        test_example(r#"
        (assert-eq (str-remove "ab" 0) "b")
        (assert-eq (str-remove "ab" 1) "a")
        (assert-throw (str-remove "ab" 2) { :tag :err-lookup, :got 2})
        "#);

        test_example(r#"
        (assert-eq (str-update "ab" 0 'z') "zb")
        (assert-eq (str-update "ab" 1 'z') "az")
        (assert-throw (str-update "ab" 2 'z') { :tag :err-lookup, :got 2})
        "#);

        test_example(r#"
        (assert-eq (str-slice "ab" 1 1) "")
        (assert-eq (str-slice "ab" 0 1) "a")
        (assert-eq (str-slice "ab" 1 2) "b")
        (assert-eq (str-slice "ab" 0 2) "ab")
        (assert-throw (str-slice "" 0 1) { :tag :err-lookup, :got 1})
        (assert-throw (str-slice "" 2 3) { :tag :err-lookup, :got 2})
        (assert-throw (str-slice "abcd" 2 1) { :tag :err-lookup, :got 1})
        "#);

        test_example(r#"
        (assert-eq (str-splice "ab" 0 "cd") "cdab")
        (assert-eq (str-splice "ab" 1 "cd") "acdb")
        (assert-eq (str-splice "ab" 2 "cd") "abcd")
        (assert-throw (str-splice "ab" 3 "cd") { :tag :err-lookup, :got 3})
        "#);

        test_example(r#"
        (assert-eq (str-concat "ab" "cd") "abcd")
        (assert-eq (str-concat "" "cd") "cd")
        (assert-eq (str-concat "ab" "") "ab")
        "#);

        test_example(r#"
        (let (:mut out) "z" (do
            (str-iter "abcd" (fn [elem] (set! out (str-insert out 0 elem))))
            (assert-eq out "dcbaz")
        ))
        (let (:mut out) "z" (do
            (str-iter "abcd" (fn [elem] (if
                    (= elem 'c') true
                    (set! out (str-insert out 0 elem))
                )))
            (assert-eq out "baz")
        ))
        (assert-throw (str-iter "ab" (fn [c] (throw c))) 'a')
        "#);

        test_example(r#"
        (let (:mut out) "z" (do
            (str-iter-back "abcd" (fn [elem] (set! out (str-insert out 0 elem))))
            (assert-eq out "abcdz")
        ))
        (let (:mut out) "z" (do
            (str-iter-back "abcd" (fn [elem] (if
                    (= elem 'c') true
                    (set! out (str-insert out 0 elem))
                )))
            (assert-eq out "dz")
        ))
        (assert-throw (str-iter-back "ab" (fn [c] (throw c))) 'b')
        "#);

        test_example(r#"
        (let (:mut product) 1 (do
            (str-iter-utf8 "abc" (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 941094)
        ))
        (let (:mut product) 1 (do
            (str-iter-utf8 "abc" (fn [elem] (sf-if
                    (= elem 98) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 97)
        ))
        (assert-throw (str-iter-utf8 "abc" (fn [b] (throw b))) 97)
        "#);

        test_example(r#"
        (let (:mut product) 1 (do
            (str-iter-utf8-back "abc" (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 941094)
        ))
        (let (:mut product) 1 (do
            (str-iter-utf8-back "abc" (fn [elem] (sf-if
                    (= elem 98) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 99)
        ))
        (assert-throw (str-iter-utf8-back "abc" (fn [b] (throw b))) 99)
        "#);
    }
}
