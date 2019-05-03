use im_rc::OrdMap as ImOrdMap;

use crate::context::Context;
use crate::gc_foreign::OrdMap;
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

pub fn assert_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-assert")),
        ])))
}

pub fn type_error(got: &Value, expected: &str) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-type")),
            (Value::kw_str("expected"), Value::kw_str(expected)),
            (Value::kw_str("got"), typeof__(got)),
        ])))
}

pub fn type_error_(got: &Value, expected: &Value) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-type")),
            (Value::kw_str("expected"), expected.clone()),
            (Value::kw_str("got"), typeof__(got)),
        ])))
}

pub fn index_error(got: usize) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-index")),
        (Value::kw_str("got"), Value::int(got as i64)),
        ])))
}

pub fn negative_error(got: i64) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-negative")),
        (Value::kw_str("got"), Value::int(got)),
        ])))
}

pub fn wrap_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-wrap")),
        ])))
}

pub fn zero_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-zero")),
        ])))
}

fn int_to_usize(n: i64) -> Result<usize, Value> {
    if n >= 0 {
        Ok(n as usize)
    } else {
        Err(negative_error(n))
    }
}

fn int_to_u64(n: i64) -> Result<u64, Value> {
    if n > 0 {
        Ok(n as u64)
    } else {
        Err(negative_error(n))
    }
}

macro_rules! bool_ {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Bool(b)) => *b,
            _ => return Err(type_error(&$v, "bool")),
        }
    )
}

macro_rules! int {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Int(n)) => *n,
            _ => return Err(type_error(&$v, "int")),
        }
    )
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

macro_rules! arg_opt {
    ($v:expr, $i:expr) => (
        arr!($v).0.get($i).clone()
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

pub fn bool_not(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let b = bool_!(arg!(args, 0));

    Ok(Value::bool_(!b))
}

pub fn bool_and(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let b0 = bool_!(arg!(args, 0));
    let b1 = bool_!(arg!(args, 1));

    Ok(Value::bool_(b0 && b1))
}

pub fn bool_or(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let b0 = bool_!(arg!(args, 0));
    let b1 = bool_!(arg!(args, 1));

    Ok(Value::bool_(b0 || b1))
}

pub fn bool_if(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let b0 = bool_!(arg!(args, 0));
    let b1 = bool_!(arg!(args, 1));

    Ok(Value::bool_(if b0 { b1 } else { true }))
}

pub fn bool_xor(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let b0 = bool_!(arg!(args, 0));
    let b1 = bool_!(arg!(args, 1));

    Ok(Value::bool_(b0 != b1))
}

pub fn bool_iff(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let b0 = bool_!(arg!(args, 0));
    let b1 = bool_!(arg!(args, 1));

    Ok(Value::bool_(b0 == b1))
}

/////////////////////////////////////////////////////////////////////////////

pub fn is_int(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(match arg!(args, 0) {
        Value::Atomic(Atomic::Int(..)) => Value::bool_(true),
        _ => Value::bool_(false),
    })
}

pub fn int_count_ones(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::int(int!(arg!(args, 0)).count_ones() as i64))
}

pub fn int_count_zeros(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::int(int!(arg!(args, 0)).count_zeros() as i64))
}

pub fn int_leading_ones(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    Ok(Value::int((!n as u64).leading_zeros() as i64))
}

pub fn int_leading_zeros(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::int(int!(arg!(args, 0)).leading_zeros() as i64))
}

pub fn int_trailing_ones(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    Ok(Value::int((!n as u64).trailing_zeros() as i64))
}

pub fn int_trailing_zeros(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::int(int!(arg!(args, 0)).trailing_zeros() as i64))
}

pub fn int_rotate_left(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let by = int_to_u64(int!(arg!(args, 1)))?;
    Ok(Value::int(n.rotate_left(by as u32)))
}

pub fn int_rotate_right(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let by = int_to_u64(int!(arg!(args, 1)))?;
    Ok(Value::int(n.rotate_right(by as u32)))
}

pub fn int_reverse_bytes(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::int(int!(arg!(args, 0)).swap_bytes() as i64))
}

pub fn int_reverse_bits(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::int(int!(arg!(args, 0)).reverse_bits() as i64))
}

pub fn int_add(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    match n.checked_add(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(wrap_error())
        }
    }
}

pub fn int_sub(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    match n.checked_sub(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(wrap_error())
        }
    }
}

pub fn int_mul(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    match n.checked_mul(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(wrap_error())
        }
    }
}

pub fn int_div(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));

    match n.checked_div_euclid(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(if m == 0 { zero_error() } else { wrap_error() })
        }
    }
}

pub fn int_div_trunc(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));

    match n.checked_div(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(if m == 0 { zero_error() } else { wrap_error() })
        }
    }
}

pub fn int_mod(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));

    match n.checked_rem_euclid(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(if m == 0 { zero_error() } else { wrap_error() })
        }
    }
}

pub fn int_mod_trunc(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));

    match n.checked_rem(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(if m == 0 { zero_error() } else { wrap_error() })
        }
    }
}

pub fn int_neg(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));

    match n.checked_neg() {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(wrap_error())
        }
    }
}

pub fn int_shl(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int_to_u64(int!(arg!(args, 1)))?;

    if m >= 64 {
        Ok(Value::int(0))
    } else {
        Ok(Value::int(n << m))
    }
}

pub fn int_shr(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int_to_u64(int!(arg!(args, 1)))?;

    if m >= 64 {
        Ok(Value::int(0))
    } else {
        Ok(Value::int(n >> m))
    }
}

pub fn int_abs(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));

    match n.checked_abs() {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(wrap_error())
        }
    }
}

pub fn int_pow(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int_to_u64(int!(arg!(args, 1)))?;

    match n.checked_pow(m as u32) {
        Some(yay) => Ok(Value::int(yay)),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(wrap_error())
        }
    }
}

pub fn int_add_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    Ok(Value::int(n.saturating_add(m)))
}

pub fn int_sub_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    Ok(Value::int(n.saturating_sub(m)))
}

pub fn int_mul_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    Ok(Value::int(n.saturating_mul(m)))
}

pub fn int_pow_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    Ok(Value::int(n.saturating_pow(m as u32)))
}

pub fn int_add_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    Ok(Value::int(n.wrapping_add(m)))
}

pub fn int_sub_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    Ok(Value::int(n.wrapping_sub(m)))
}

pub fn int_mul_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    Ok(Value::int(n.wrapping_mul(m)))
}

pub fn int_div_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));

    if m == 0 {
        match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(zero_error()),
        }
    } else {
        Ok(Value::int(n.wrapping_div_euclid(m)))
    }
}

pub fn int_div_trunc_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));

    if m == 0 {
        match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(zero_error()),
        }
    } else {
        Ok(Value::int(n.wrapping_div(m)))
    }
}

pub fn int_mod_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));

    if m == 0 {
        match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(zero_error()),
        }
    } else {
        Ok(Value::int(n.wrapping_rem_euclid(m)))
    }
}

pub fn int_mod_trunc_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));

    if m == 0 {
        match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(zero_error()),
        }
    } else {
        Ok(Value::int(n.wrapping_rem(m)))
    }
}

pub fn int_neg_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    Ok(Value::int(n.wrapping_neg()))
}

pub fn int_abs_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    Ok(Value::int(n.wrapping_abs()))
}

pub fn int_pow_wrap(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    let m = int!(arg!(args, 1));
    Ok(Value::int(n.wrapping_pow(m as u32)))
}

pub fn int_signum(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let n = int!(arg!(args, 0));
    Ok(Value::int(n.signum()))
}

/////////////////////////////////////////////////////////////////////////////

pub fn arr_get(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let index = int_to_usize(int!(arg!(args, 0)))?;
    let arr = arr!(arg!(args, 1));

    match arr.0.get(index) {
        Some(yay) => Ok(yay.clone()),
        None => Err(index_error(index)),
    }
}

/////////////////////////////////////////////////////////////////////////////

pub fn pavo_assert(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    match arg!(args, 0) {
        Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(false)) => Err(assert_error()),
        _ => Ok(Value::nil()),
    }
}

pub fn pavo_assert_not(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    match arg!(args, 0) {
        Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(false)) => Ok(Value::nil()),
        _ => Err(assert_error()),
    }
}

pub fn pavo_assert_eq(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    if arg!(args, 0) == arg!(args, 1) {
        Ok(Value::nil())
    } else {
        Err(assert_error())
    }
}

pub fn pavo_assert_type(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let type_ = arg!(args, 0);
    let x = arg!(args, 1);
    let typeof_x = typeof__(&x);
    if typeof_x == type_ {
        Ok(Value::nil())
    } else {
        Err(type_error_(&typeof_x, &type_))
    }
}

/////////////////////////////////////////////////////////////////////////////

pub fn pavo_eq(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::bool_(arg!(args, 0) == arg!(args, 1)))
}

pub fn pavo_lt(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::bool_(arg!(args, 0) < arg!(args, 1)))
}

pub fn pavo_lte(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::bool_(arg!(args, 0) <= arg!(args, 1)))
}

pub fn pavo_gt(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::bool_(arg!(args, 0) > arg!(args, 1)))
}

pub fn pavo_gte(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(Value::bool_(arg!(args, 0) >= arg!(args, 1)))
}

/////////////////////////////////////////////////////////////////////////////

pub fn typeof_(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(typeof__(&arg!(args, 0)))
}

pub fn is_truthy(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    Ok(match arg!(args, 0) {
        Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(false)) => Value::bool_(false),
        _ => Value::bool_(true),
    })
}
