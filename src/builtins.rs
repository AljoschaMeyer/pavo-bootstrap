use im_rc::OrdMap as ImOrdMap;

use crate::context::Context;
use crate::gc_foreign::{Vector, OrdMap};
use crate::value::{Value, Atomic, Id};

pub fn typeof__(v: &Value) -> Value {
    match v {
        Value::Fun(..) => Value::kw_str("function"),
        Value::Atomic(Atomic::Nil) => Value::kw_str("nil"),
        Value::Atomic(Atomic::Bool(..)) => Value::kw_str("bool"),
        Value::Atomic(Atomic::Int(..)) => Value::kw_str("int"),
        Value::Atomic(Atomic::Float(..)) => Value::kw_str("float"),
        Value::Atomic(Atomic::Char(..)) => Value::kw_str("char"),
        Value::Atomic(Atomic::String(..)) => Value::kw_str("string"),
        Value::Atomic(Atomic::Bytes(..)) => Value::kw_str("bytes"),
        Value::Atomic(Atomic::Keyword(..)) => Value::kw_str("keyword"),
        Value::Id(Id::User(..)) => Value::kw_str("identifier"),
        Value::Id(Id::Symbol(..)) => Value::kw_str("symbol"),
        Value::Arr(..) => Value::kw_str("array"),
        Value::App(..) => Value::kw_str("application"),
        Value::Map(..) => Value::kw_str("map"),
        Value::Set(..) => Value::kw_str("set"),
    }
}

pub fn type_error(got: &Value, expected: &str) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-type")),
            (Value::kw_str("expected"), Value::kw_str(expected)),
            (Value::kw_str("got"), typeof__(got)),
        ])))
}

pub fn index_error(got: i64) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-index")),
        (Value::kw_str("got"), Value::int(got)),
        ])))
}

macro_rules! arr {
    ($v:expr) => (
        match &$v {
            Value::Arr(arr) => arr.clone(),
            _ => return Err(type_error(&$v, "array")),
        }
    )
}

macro_rules! arg {
    ($v:expr, $i:expr) => (
        match arr!($v).0.get($i) {
            Some(the_arg) => the_arg.clone(),
            None => Value::nil(),
        }
    )
}

/////////////////////////////////////////////////////////////////////////////

pub fn is_nil(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(match arg!(args, 0) {
        Value::Atomic(Atomic::Nil) => Value::bool_(true),
        _ => Value::bool_(false),
    })
}

/////////////////////////////////////////////////////////////////////////////

pub fn is_bool(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(match arg!(args, 0) {
        Value::Atomic(Atomic::Bool(..)) => Value::bool_(true),
        _ => Value::bool_(false),
    })
}

/////////////////////////////////////////////////////////////////////////////

pub fn typeof_(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(typeof__(&arg!(args, 0)))
}
