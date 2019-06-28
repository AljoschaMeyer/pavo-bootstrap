#![feature(reverse_bits)]
#![feature(euclidean_division)]
#![feature(copysign)]

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
        assert_any_static_error("(sf-lambda (:mut a))");
        assert_any_static_error("(sf-lambda (:mut a) 0 1)");
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
        assert_ok("((sf-lambda [a (:mut a)] (sf-set! a 42)) 0 1)", Value::nil());
        assert_any_static_error("some-id");
        assert_any_static_error("[some-id]");
        assert_any_static_error("(sf-set! some-id 42)");
        assert_any_static_error("(sf-set! int-max-val 42)");
        assert_any_static_error("(sf-try 0 a (sf-set! a 42))");
        assert_any_static_error("(sf-lambda a (sf-set! a 42))");
        assert_any_static_error("(sf-lambda [(:mut a) a] (sf-set! a 42))");
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
        assert_throw("(int-add 1)", execute("{:tag :err-num-args}").unwrap());
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
            execute("{:tag :err-num-args}").unwrap()
        );
        assert_ok("((sf-lambda [a b] (int-add a b)) 1 2)", Value::int(3));
        assert_ok("((sf-lambda [a (:mut b)] (sf-do (sf-set! b 3) (int-add a b))) 1 2)", Value::int(4));
        assert_ok("((sf-lambda [a a] a) 0 1)", Value::int(1));
    }

    // ## Toplevel Values

    #[test]
    fn test_function_argument_errors() {
        test_example("
        (assert-throw (bool-not) { :tag :err-num-args})
        #(assert-throw (bool-not 42 43) { :tag :err-num-args })
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

    #[test]
    fn test_toplevel_float() {
        test_example("
        (assert-eq float-max-val 1.7976931348623157e308)
        (assert-throw (float-mul float-max-val 2.0) :inf)
        ");

        test_example("
        (assert-eq float-min-val -1.7976931348623157e308)
        (assert-throw (float-mul float-min-val 2.0) :-inf)
        ");

        test_example("
        (assert-eq (float-add 1.0 2.0) 3.0)
        (assert-eq (float-add 1.0 -2.0) -1.0)
        (assert-eq (float-add 0.1 0.2) 0.30000000000000004)
        ");

        test_example("
        (assert-eq (float-sub 1.0 2.0) -1.0)
        (assert-eq (float-sub 1.0 -2.0) 3.0)
        ");

        test_example("
        (assert-eq (float-mul 2.0 3.0) 6.0)
        (assert-eq (float-mul 2.0 -3.0) -6.0)
        ");

        test_example("
        (assert-eq (float-div 8.0 3.0) 2.6666666666666665)
        ");

        test_example("
        (assert-eq (float-div 8.0 3.0) 2.6666666666666665)
        (assert-throw (float-div 1.0 0.0) :inf)
        (assert-throw (float-div 1.0 -0.0) :inf)
        (assert-throw (float-div 0.0 0.0) :nan)
        ");

        test_example("(assert-eq (float-mul-add 1.2 3.4 5.6) 9.68)");

        test_example("
        (assert-eq (float-neg 1.2) -1.2)
        (assert-eq (float-neg -1.2) 1.2)
        (assert-eq (float-neg 0.0) 0.0)
        ");

        test_example("
        (assert-eq (float-floor 1.9) 1.0)
        (assert-eq (float-floor 1.0) 1.0)
        (assert-eq (float-floor -1.1) -2.0)
        ");

        test_example("
        (assert-eq (float-ceil 1.1) 2.0)
        (assert-eq (float-ceil 1.0) 1.0)
        (assert-eq (float-ceil -1.9) -1.0)
        ");

        test_example("
        (assert-eq (float-round 1.0) 1.0)
        (assert-eq (float-round 1.49) 1.0)
        (assert-eq (float-round 1.51) 2.0)
        (assert-eq (float-round 1.5) 2.0)
        (assert-eq (float-round 2.5) 2.0)
        ");

        test_example("
        (assert-eq (float-trunc 1.0) 1.0)
        (assert-eq (float-trunc 1.49) 1.0)
        (assert-eq (float-trunc 1.51) 1.0)
        (assert-eq (float-trunc -1.51) -1.0)
        ");

        test_example("
        (assert-eq (float-fract 1.0) 0.0)
        (assert-eq (float-fract 1.49) 0.49)
        (assert-eq (float-fract 1.51) 0.51)
        (assert-eq (float-fract -1.51) -0.51)
        ");

        test_example("
        (assert-eq (float-abs 1.2) 1.2)
        (assert-eq (float-abs -1.2) 1.2)
        (assert-eq (float-abs 0.0) 0.0)
        ");

        test_example("
        (assert-eq (float-signum 99.2) 1.0)
        (assert-eq (float-signum -99.2) -1.0)
        (assert-eq (float-signum 0.0) 0.0)
        (assert-eq (float-signum -0.0) 0.0)
        ");

        test_example("(assert-eq (float-pow 1.2 3.4) 1.858729691979481)");

        test_example("
        (assert-eq (float-sqrt 1.2) 1.0954451150103321)
        (assert-throw (float-sqrt -1.0) :nan)
        ");

        test_example("(assert-eq (float-exp 1.2) 3.3201169227365472)");

        test_example("(assert-eq (float-exp2 1.2) 2.2973967099940698)");

        test_example("(assert-eq (float-ln 1.2) 0.1823215567939546)");

        test_example("(assert-eq (float-log2 1.2) 0.2630344058337938)");

        test_example("(assert-eq (float-log10 1.2) 0.07918124604762482)");

        test_example("
        (assert-eq (float-hypot 1.2 3.4) 3.605551275463989)
        (assert-eq (float-hypot 1.2 -3.4) 3.605551275463989)
        (assert-eq (float-hypot -1.2 3.4) 3.605551275463989)
        (assert-eq (float-hypot -1.2 -3.4) 3.605551275463989)
        ");

        test_example("(assert-eq (float-sin 1.2) 0.9320390859672263)");

        test_example("(assert-eq (float-cos 1.2) 0.3623577544766736)");

        test_example("(assert-eq (float-tan 1.2) 2.5721516221263188)");

        test_example("
        (assert-eq (float-asin 0.8) 0.9272952180016123)
        (assert-throw (float-asin 1.2) :nan)
        ");

        test_example("
        (assert-eq (float-acos 0.8) 0.6435011087932843)
        (assert-throw (float-acos 1.2) :nan)
        ");

        test_example("(assert-eq (float-atan 1.2) 0.8760580505981934)");

        test_example("(assert-eq (float-atan2 1.2 3.4) 0.3392926144540447)");

        test_example("(assert-eq (float-exp-m1 1.2) 2.3201169227365472)");

        test_example("(assert-eq (float-ln-1p 1.2) 0.7884573603642702)");

        test_example("(assert-eq (float-sinh 1.2) 1.5094613554121725)");

        test_example("(assert-eq (float-cosh 1.2) 1.8106555673243747)");

        test_example("(assert-eq (float-tanh 1.2) 0.8336546070121552)");

        test_example("(assert-eq (float-asinh 1.2) 1.015973134179692)");

        test_example("(assert-eq (float-acosh 1.2) 0.6223625037147785)");

        test_example("
        (assert-eq (float-atanh 0.8) 1.0986122886681098)
        (assert-throw (float-atanh 1.2) :nan)
        ");

        test_example("
        (assert-eq (float-normal? 1.0) true)
        (assert-eq (float-normal? 1.0e-308) false) # subnormal
        (assert-eq (float-normal? 0.0) false)
        ");

        test_example("(assert-eq (float->degrees 1.2) 68.75493541569878)");

        test_example("(assert-eq (float->radians 1.2) 0.020943951023931952)");

        test_example("
        (assert-eq (float->int 0.0) 0)
        (assert-eq (float->int 1.0) 1)
        (assert-eq (float->int -1.0) -1)
        (assert-eq (float->int 1.9) 1)
        (assert-eq (float->int -1.9) -1)
        (assert-eq (float->int float-max-val) int-max-val)
        (assert-eq (float->int float-min-val) int-min-val)
        ");

        test_example("
        (assert-eq (int->float 0) 0.0)
        (assert-eq (int->float 1) 1.0)
        (assert-eq (int->float -1) -1.0)
        (assert-eq (int->float 9007199254740993) 9007199254740992.0)
        (assert-eq (int->float -9007199254740993) -9007199254740992.0)
        ");

        test_example("
        (assert-eq (float->bits 1.2) 4608083138725491507)
        (assert-eq (float->bits -1.2) -4615288898129284301)
        (assert-eq (float->bits 0.0) 0)
        (assert-eq (float->bits -0.0) 0)
        ");

        test_example("
        (assert-eq (bits=>float 42) 2.08e-322)
        (assert-throw (bits=>float -42) :nan)
        (assert-throw (bits=>float 9218868437227405312) :inf)
        (assert-throw (bits=>float -4503599627370496) :-inf)
        ");

        test_example("
        (assert-eq (bits=>float? 42) true)
        (assert-eq (bits=>float? -42) false)
        (assert-eq (bits=>float? 9218868437227405312) false)
        (assert-eq (bits=>float? -4503599627370496) false)
        ");
    }

    #[test]
    fn test_toplevel_identifier() {
        test_example(r#"
        (assert-eq (str=>id "foo") $foo)
        (assert-throw (str=>id "nil") { :tag :err-identifier, :got "nil" })
        (assert-throw (str=>id "true") { :tag :err-identifier, :got "true" })
        (assert-throw (str=>id "false") { :tag :err-identifier, :got "false" })
        (assert-throw (str=>id "42") { :tag :err-identifier, :got "42" })
        (assert-throw (str=>id "1.2") { :tag :err-identifier, :got "1.2" })
        (assert-throw (str=>id "") { :tag :err-identifier, :got ""})
        (assert-throw (str=>id "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") { :tag :err-identifier, :got "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789"})
        (assert-throw (str=>id ":a") { :tag :err-identifier, :got ":a"})
        (assert-throw (str=>id "ß") { :tag :err-identifier, :got "ß"})
        "#);

        test_example(r#"
        (assert-eq (str=>id? "foo") true)
        (assert-eq (str=>id? "nil") false)
        (assert-eq (str=>id? "true") false)
        (assert-eq (str=>id? "false") false)
        (assert-eq (str=>id? "42") false)
        (assert-eq (str=>id? "-_") true)
        (assert-eq (str=>id? "-42") false)
        (assert-eq (str=>id? "1.2") false)
        (assert-eq (str=>id? "") false)
        (assert-eq (str=>id? "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") false)
        (assert-eq (str=>id? "ß") false)
        (assert-eq (str=>id? ":a") false)
        "#);

        test_example(r#"(assert-eq (id->str $foo) "foo")"#);
    }

    #[test]
    fn test_toplevel_keyword() {
        test_example(r#"
        (assert-eq (str=>kw "foo") :foo)
        (assert-eq (str=>kw "nil") :nil)
        (assert-eq (str=>kw "42") :42)
        (assert-throw (str=>kw "") { :tag :err-kw, :got ""})
        (assert-throw (str=>kw "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") { :tag :err-kw, :got "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789"})
        (assert-throw (str=>kw ":a") { :tag :err-kw, :got ":a"})
        (assert-throw (str=>kw "ß") { :tag :err-kw, :got "ß"})
        "#);

        test_example(r#"
        (assert-eq (str=>kw? "foo") true)
        (assert-eq (str=>kw? "nil") true)
        (assert-eq (str=>kw? "42") true)
        (assert-eq (str=>kw? "-_") true)
        (assert-eq (str=>kw? "-42") true)
        (assert-eq (str=>kw? "") false)
        (assert-eq (str=>kw? "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") false)
        (assert-eq (str=>kw? "ß") false)
        (assert-eq (str=>kw? ":a") false)
        "#);

        test_example(r#"(assert-eq (kw->str :foo) "foo")"#);
    }

    #[test]
    fn test_toplevel_array() {
        test_example("
        (assert-eq (arr-count []) 0)
        (assert-eq (arr-count [nil]) 1)
        (assert-eq (arr-count [0, 1, 2]) 3)
        ");

        test_example("
        (assert-eq (arr-get [true] 0) true)
        (assert-throw (arr-get [] 0) { :tag :err-lookup, :got 0})
        ");

        test_example("
        (assert-eq (arr-insert [0 1] 0 42) [42 0 1])
        (assert-eq (arr-insert [0 1] 1 42) [0 42 1])
        (assert-eq (arr-insert [0 1] 2 42) [0 1 42])
        (assert-throw (arr-insert [0 1] 3 42) { :tag :err-lookup, :got 3})
        ");

        test_example("
        (assert-eq (arr-remove [0 1] 0) [1])
        (assert-eq (arr-remove [0 1] 1) [0])
        (assert-throw (arr-remove [0 1] 3) { :tag :err-lookup, :got 3})
        ");

        test_example("
        (assert-eq (arr-update [0 1] 0 42) [42 1])
        (assert-eq (arr-update [0 1] 1 42) [0 42])
        (assert-throw (arr-update [0 1] 2 42) { :tag :err-lookup, :got 2})
        ");

        test_example("
        (assert-eq (arr-slice [true false] 1 1) [])
        (assert-eq (arr-slice [true false] 0 1) [true])
        (assert-eq (arr-slice [true false] 1 2) [false])
        (assert-eq (arr-slice [true false] 0 2) [true false])
        (assert-throw (arr-slice [] 0 1) { :tag :err-lookup, :got 1})
        (assert-throw (arr-slice [] 2 3) { :tag :err-lookup, :got 2})
        (assert-throw (arr-slice [0 1 2 3] 2 1) { :tag :err-lookup, :got 1})
        ");

        test_example("
        (assert-eq (arr-splice [0 1] 0 [10 11]) [10 11 0 1])
        (assert-eq (arr-splice [0 1] 1 [10 11]) [0 10 11 1])
        (assert-eq (arr-splice [0 1] 2 [10 11]) [0 1 10 11])
        (assert-throw (arr-splice [0 1] 3 [10 11]) { :tag :err-lookup, :got 3})
        ");

        test_example("
        (assert-eq (arr-concat [0 1] [2 3]) [0 1 2 3])
        (assert-eq (arr-concat [] [0 1]) [0 1])
        (assert-eq (arr-concat [0 1] []) [0 1])
        ");

        test_example("
        (let (:mut product) 1 (do
            (arr-iter [1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (arr-iter [1 2 3 4] (fn [elem] (if
                    (= elem 3) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 2)
        ))
        (assert-throw (arr-iter [0 1] (fn [n] (throw n))) 0)
        ");

        test_example("
        (let (:mut product) 1 (do
            (arr-iter-back [1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (arr-iter-back [1 2 3 4] (fn [elem] (if
                    (= elem 3) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 4)
        ))
        (assert-throw (arr-iter-back [0 1] (fn [n] (throw n))) 1)
        ");
    }

    #[test]
    fn test_toplevel_application() {
        test_example("
        (assert-eq (app-count $()) 0)
        (assert-eq (app-count $(nil)) 1)
        (assert-eq (app-count $(0, 1, 2)) 3)
        ");

        test_example("
        (assert-eq (app-get $(true) 0) true)
        (assert-throw (app-get $() 0) { :tag :err-lookup, :got 0})
        ");

        test_example("
        (assert-eq (app-insert $(0 1) 0 42) $(42 0 1))
        (assert-eq (app-insert $(0 1) 1 42) $(0 42 1))
        (assert-eq (app-insert $(0 1) 2 42) $(0 1 42))
        (assert-throw (app-insert $(0 1) 3 42) { :tag :err-lookup, :got 3})
        ");

        test_example("
        (assert-eq (app-remove $(0 1) 0) $(1))
        (assert-eq (app-remove $(0 1) 1) $(0))
        (assert-throw (app-remove $(0 1) 3) { :tag :err-lookup, :got 3})
        ");

        test_example("
        (assert-eq (app-update $(0 1) 0 42) $(42 1))
        (assert-eq (app-update $(0 1) 1 42) $(0 42))
        (assert-throw (app-update $(0 1) 2 42) { :tag :err-lookup, :got 2})
        ");

        test_example("
        (assert-eq (app-slice $(true false) 1 1) $())
        (assert-eq (app-slice $(true false) 0 1) $(true))
        (assert-eq (app-slice $(true false) 1 2) $(false))
        (assert-eq (app-slice $(true false) 0 2) $(true false))
        (assert-throw (app-slice $() 0 1) { :tag :err-lookup, :got 1})
        (assert-throw (app-slice $() 2 3) { :tag :err-lookup, :got 2})
        (assert-throw (app-slice $(0 1 2 3) 2 1) { :tag :err-lookup, :got 1})
        ");

        test_example("
        (assert-eq (app-splice $(0 1) 0 $(10 11)) $(10 11 0 1))
        (assert-eq (app-splice $(0 1) 1 $(10 11)) $(0 10 11 1))
        (assert-eq (app-splice $(0 1) 2 $(10 11)) $(0 1 10 11))
        (assert-throw (app-splice $(0 1) 3 $(10 11)) { :tag :err-lookup, :got 3})
        ");

        test_example("
        (assert-eq (app-concat $(0 1) $(2 3)) $(0 1 2 3))
        (assert-eq (app-concat $() $(0 1)) $(0 1))
        (assert-eq (app-concat $(0 1) $()) $(0 1))
        ");

        test_example("
        (let (:mut product) 1 (do
            (app-iter $(1 2 3 4) (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (app-iter $(1 2 3 4) (fn [elem] (if
                    (= elem 3) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 2)
        ))
        (assert-throw (app-iter $(0 1) (fn [n] (throw n))) 0)
        ");

        test_example("
        (let (:mut product) 1 (do
            (app-iter-back $(1 2 3 4) (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (app-iter-back $(1 2 3 4) (fn [elem] (if
                    (= elem 3) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 4)
        ))
        (assert-throw (app-iter-back $(0 1) (fn [n] (throw n))) 1)
        ");

        // TODO uncomment when quasiquote has been implemented
        test_example("
        #(assert-eq (app-apply `(;int-add 1 2)) 3)
        #(assert-throw (app-apply `(;int-add 1)) {:tag :err-num-args})
        (assert-throw (app-apply $()) {:tag :err-lookup :got 0})
        (assert-throw (app-apply $(42)) {:tag :err-type, :expected :function, :got :int})
        ");
    }

    #[test]
    fn test_toplevel_set() {
        test_example("
        (assert-eq (set-count @{}) 0)
        (assert-eq (set-count @{nil}) 1)
        (assert-eq (set-count @{0, 1, 2}) 3)
        ");

        test_example("
        (assert-eq (set-contains? @{ nil } nil) true)
        (assert-eq (set-contains? @{ 42 } 43) false)
        (assert-eq (set-contains? @{} nil) false)
        ");

        test_example("
        (assert-eq (set-min @{ 4 3 }) 3)
        (assert-throw (set-min @{}) { :tag :err-collection-empty })
        ");

        test_example("
        (assert-eq (set-max @{ 4 3 }) 4)
        (assert-throw (set-max @{}) { :tag :err-collection-empty })
        ");

        test_example("
        (assert-eq (set-insert @{} nil) @{nil})
        (assert-eq (set-insert @{nil} nil) @{nil})
        ");

        test_example("
        (assert-eq (set-remove @{nil} nil) @{})
        (assert-eq (set-remove @{} nil) @{})
        ");

        test_example("
        (assert-eq (set-union @{1 2} @{2 3}) @{1 2 3})
        (assert-eq (set-union @{1 2} @{}) @{1 2})
        (assert-eq (set-union @{} @{2 3}) @{2 3})
        (assert-eq (set-union @{} @{}) @{})
        ");

        test_example("
        (assert-eq (set-intersection @{1 2} @{2 3}) @{2})
        (assert-eq (set-intersection @{1 2} @{}) @{})
        (assert-eq (set-intersection @{} @{2 3}) @{})
        (assert-eq (set-intersection @{} @{}) @{})
        ");

        test_example("
        (assert-eq (set-difference @{1 2} @{2 3}) @{1})
        (assert-eq (set-difference @{1 2} @{}) @{1 2})
        (assert-eq (set-difference @{} @{2 3}) @{})
        (assert-eq (set-difference @{} @{}) @{})
        ");

        test_example("
        (assert-eq (set-symmetric-difference @{1 2} @{2 3}) @{1 3})
        (assert-eq (set-symmetric-difference @{1 2} @{}) @{1 2})
        (assert-eq (set-symmetric-difference @{} @{2 3}) @{2 3})
        (assert-eq (set-symmetric-difference @{} @{}) @{})
        ");

        test_example("
        (let (:mut product) 1 (do
            (set-iter @{4 2 3 1} (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (set-iter @{4 2 3 1} (fn [elem] (if
                    (= elem 3) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 2)
        ))
        (assert-throw (set-iter @{0 1} (fn [n] (throw n))) 0)
        ");

        test_example("
        (let (:mut product) 1 (do
            (set-iter-back @{4 2 3 1} (fn [elem] (set! product (int-mul product elem))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (set-iter-back @{4 2 3 1} (fn [elem] (if
                    (= elem 3) true
                    (set! product (int-mul product elem))
                )))
            (assert-eq product 4)
        ))
        (assert-throw (set-iter-back @{0 1} (fn [n] (throw n))) 1)
        ");
    }

    #[test]
    fn test_toplevel_map() {
        test_example("
        (assert-eq (map-count {}) 0)
        (assert-eq (map-count {{} nil}) 1)
        (assert-eq (map-count {0 42, 1 41, 2 40}) 3)
        ");

        test_example("
        (assert-eq (map-get {0 42} 0) 42)
        (assert-throw (map-get {} 0) { :tag :err-lookup, :got 0})
        ");

        test_example("
        (assert-eq (map-contains? { nil 0 } nil) true)
        (assert-eq (map-contains? { 42 0 } 43) false)
        (assert-eq (map-contains? {} nil) false)
        ");

        test_example("
        (assert-eq (map-min {0 42, 1 41}) 42)
        (assert-throw (map-min {}) { :tag :err-collection-empty })
        ");

        test_example("
        (assert-eq (map-min-key {0 42, 1 41}) 0)
        (assert-throw (map-min-key {}) { :tag :err-collection-empty })
        ");

        test_example("
        (assert-eq (map-min-entry {0 42, 1 41}) [0 42])
        (assert-throw (map-min-entry {}) { :tag :err-collection-empty })
        ");

        test_example("
        (assert-eq (map-max {0 42, 1 41}) 41)
        (assert-throw (map-max {}) { :tag :err-collection-empty })
        ");

        test_example("
        (assert-eq (map-max-key {0 42, 1 41}) 1)
        (assert-throw (map-max-key {}) { :tag :err-collection-empty })
        ");

        test_example("
        (assert-eq (map-max-entry {0 42, 1 41}) [1 41])
        (assert-throw (map-max-entry {}) { :tag :err-collection-empty })
        ");

        test_example("
        (assert-eq (map-insert {} 0 42) {0 42})
        (assert-eq (map-insert {0 42} 0 43) {0 43})
        ");

        test_example("
        (assert-eq (map-remove {0 42} 0) {})
        (assert-eq (map-remove {} 0) {})
        ");

        test_example("
        (assert-eq (map-union {0 42, 1 41} {1 17, 2 40}) {0 42, 1 41, 2 40})
        (assert-eq (map-union {0 42, 1 41} {}) {0 42, 1 41})
        (assert-eq (map-union {} {1 41, 2 40}) {1 41, 2 40})
        (assert-eq (map-union {} {}) {})
        ");

        test_example("
        (assert-eq (map-intersection {0 42, 1 41} {1 17, 2 40}) {1 41})
        (assert-eq (map-intersection {0 42, 1 41} {}) {})
        (assert-eq (map-intersection {} {1 41, 2 40}) {})
        (assert-eq (map-intersection {} {}) {})
        ");

        test_example("
        (assert-eq (map-difference {0 42, 1 41} {1 17, 2 40}) {0 42})
        (assert-eq (map-difference {0 42, 1 41} {}) {0 42, 1 41})
        (assert-eq (map-difference {} {1 41, 2 40}) {})
        (assert-eq (map-difference {} {}) {})
        ");

        test_example("
        (assert-eq (map-symmetric-difference {0 42, 1 41} {1 17, 2 40}) {0 42, 2 40})
        (assert-eq (map-symmetric-difference {0 42, 1 41} {}) {0 42, 1 41})
        (assert-eq (map-symmetric-difference {} {1 41, 2 40}) {1 41, 2 40})
        (assert-eq (map-symmetric-difference {} {}) {})
        ");

        test_example("
        (let (:mut product) 1 (do
            (map-iter {4 2, 3 1} (fn [key value] (set! product (int-mul product (int-mul key value)))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (map-iter {4 2, 3 1} (fn [key value] (if
                    (= key 3) true
                    (set! product (int-mul product (int-mul key value)))
                )))
            (assert-eq product 1)
        ))
        (assert-throw (map-iter {0 1, 2 3} (fn [n m] (throw (int-mul n m)))) 0)
        ");

        test_example("
        (let (:mut product) 1 (do
            (map-iter-back {4 2, 3 1} (fn [key value] (set! product (int-mul product (int-mul key value)))))
            (assert-eq product 24)
        ))
        (let (:mut product) 1 (do
            (map-iter-back {4 2, 3 1} (fn [key value] (if
                    (= key 3) true
                    (set! product (int-mul product (int-mul key value)))
                )))
            (assert-eq product 8)
        ))
        (assert-throw (map-iter-back {0 1, 2 3} (fn [n m] (throw (int-mul n m)))) 6)
        ");
    }

    #[test]
    fn test_toplevel_symbol() {
        test_example("
        (assert (let x (symbol) (= x x)))
        (assert-not (= (symbol) (symbol)))
        ");
    }

    #[test]
    fn test_toplevel_cell() {
        test_example("
        (assert (let x (cell 42) (= x x)))
        (assert-not (= (cell 42) (cell 42)))
        ");

        test_example("(assert-eq (cell-get (cell 42)) 42)");

        test_example("
        #(assert-eq (cell-set (cell 42) 43) nil)
        (assert-eq ((sf-lambda [x] (sf-do (cell-set x 43) (cell-get x))) (cell 42)) 43)
        ");
    }

    #[test]
    fn test_toplevel_opaque() {
        test_example("
        (let o (opaque) (sf-do
            (assert-eq ((map-get o :unhide) ((map-get o :hide) 42)) 42)
            (assert-eq (typeof ((map-get o :hide) 42)) (map-get o :type))
            (assert-throw ((map-get o :unhide) 42) {:tag :err-type :expected (map-get o :type) :got :int})
        ))
        (assert-eq (= (map-get (opaque) :type) (map-get (opaque) :type)) false)
        ");
    }

    #[test]
    fn test_toplevel_ordering() {
        test_example(r#"
        (assert-eq (cmp nil false) :<)
        (assert-eq (cmp false true) :<)
        (assert-eq (cmp true -1) :<)
        (assert-eq (cmp 999 -1.2) :<)
        (assert-eq (cmp 1.2 :zero) :<)
        (assert-eq (cmp :bcd $abc) :<)
        (assert-eq (cmp $abc (symbol)) :<)
        (assert-eq (cmp (symbol) 'a') :<)
        (assert-eq (cmp 'b' "a") :<)
        (assert-eq (cmp "zzz" @[0]) :<)
        (assert-eq (cmp @[1] [0]) :<)
        (assert-eq (cmp [1 2 3] $(1)) :<)
        (assert-eq (cmp $(1 2 3) @{1}) :<)
        (assert-eq (cmp @{1 2 3} {1 2}) :<)
        (assert-eq (cmp {1 2} cmp) :<)
        (assert-eq (cmp cmp (cell 42)) :<)
        (assert-eq (cmp (cell 42) ((map-get (opaque) :hide) 42)) :<)
        (assert-eq (cmp -1 0) :<)
        (assert-eq (cmp 0 1) :<)
        (assert-eq (cmp -0 0) :=)
        (assert-eq (cmp -1.0 0.0) :<)
        (assert-eq (cmp 0.0 1.0) :<)
        (assert-eq (cmp -0.0 0.0) :=)
        (assert-eq (cmp :a :b) :<)
        (assert-eq (cmp :a :bc) :<)
        (assert-eq (cmp :aa :ab) :<)
        (assert-eq (cmp :aa :b) :<)
        (assert-eq (cmp $a $b) :<)
        (assert-eq (cmp $a $bc) :<)
        (assert-eq (cmp $aa $ab) :<)
        (assert-eq (cmp $aa $b) :<)
        (assert-eq (cmp (symbol) (symbol)) :<)
        (assert-eq (cmp (map-get (opaque) :type) (symbol)) :<)
        (assert-eq (cmp (symbol) (map-get (opaque) :type)) :<)
        (assert-eq (cmp 'a' 'b') :<)
        (assert-eq (cmp 'A' 'a') :<)
        (assert-eq (cmp '#' 'ß') :<)
        (assert-eq (cmp "a" "b") :<)
        (assert-eq (cmp "a" "bc") :<)
        (assert-eq (cmp "aa" "ab") :<)
        (assert-eq (cmp "aa" "b") :<)
        (assert-eq (cmp @[0] @[1]) :<)
        (assert-eq (cmp @[0] @[1 2]) :<)
        (assert-eq (cmp @[0 0] @[0 1]) :<)
        (assert-eq (cmp @[0 0] @[1]) :<)
        (assert-eq (cmp [0] [1]) :<)
        (assert-eq (cmp [0] [1 2]) :<)
        (assert-eq (cmp [0 0] [0 1]) :<)
        (assert-eq (cmp [0 0] [1]) :<)
        (assert-eq (cmp @{0} @{1}) :<)
        (assert-eq (cmp @{0} @{1 2}) :<)
        (assert-eq (cmp @{0 1} @{0 2}) :<)
        (assert-eq (cmp @{0 1} @{2}) :<)
        (assert-eq (cmp {} {}) :=)
        (assert-eq (cmp {} {0 1}) :<)
        (assert-eq (cmp {0 99} {1 2}) :<)
        (assert-eq (cmp {0 1} {0 2}) :<)
        (assert-eq (cmp {0 1, 2 3} {0 1, 2 4}) :<)
        (assert-eq (cmp {0 1} {0 1, 2 3}) :<)
        (assert-eq (cmp cmp cmp) :=)
        (assert-eq (cmp app-apply cmp) :<)
        (assert-eq (cmp cmp (sf-lambda [] nil)) :<)
        (assert-eq (cmp (sf-lambda [] nil) (sf-lambda [] nil)) :<)
        (assert-eq (cmp (cell 42) (cell 41)) :<)

        (let o1 (opaque) (let o2 (opaque)
            (let hide1 (map-get o1 :hide) (let hide2 (map-get o2 :hide)
                (sf-do
                    (assert-eq (cmp (hide1 42) (hide2 41)) :<)
                    (assert-eq (cmp (hide1 41) (hide1 42)) :<)
                    (assert-eq (cmp (hide1 42) (hide1 42)) :=)
                    (assert-eq (cmp (hide1 42) (hide1 41)) :>)
                )
                ))
            ))
        "#);
    }
}
