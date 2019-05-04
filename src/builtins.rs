use std::cmp::{min, max};

use im_rc::{OrdMap as ImOrdMap, Vector as ImVector};

use crate::context::Context;
use crate::gc_foreign::{OrdMap, Vector};
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

fn index_clamp(arr: &Vector<Value>, n: i64) -> usize {
    min(max(n, 0) as usize, arr.0.len() - 1)
}

fn index_clamp_inclusive(arr: &Vector<Value>, n: i64) -> usize {
    min(max(n, 0) as usize, arr.0.len())
}

fn int_to_u64(n: i64) -> Result<u64, Value> {
    if n > 0 {
        Ok(n as u64)
    } else {
        Err(negative_error(n))
    }
}

macro_rules! index {
    ($arr:expr, $n:expr, $fb:expr) => (
        if $n < 0 || $n as usize >= $arr.0.len() {
            match $fb {
                Some(fb) => return Ok(fb.clone()),
                None => return Err(index_error($n as usize)),
            }
        } else {
            $n as usize
        }
    )
}

macro_rules! index_incl {
    ($arr:expr, $n:expr, $fb:expr) => (
        if $n < 0 || $n as usize > $arr.0.len() {
            match $fb {
                Some(fb) => return Ok(fb.clone()),
                None => return Err(index_error($n as usize)),
            }
        } else {
            $n as usize
        }
    )
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

macro_rules! fun {
    ($v:expr) => (
        match &$v {
            Value::Fun(fun) => fun.clone(),
            _ => return Err(type_error(&$v, "function")),
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

pub fn arr_count(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 1));
    Ok(Value::int(arr.0.len() as i64))
}

pub fn arr_is_empty(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 1));
    Ok(Value::bool_(arr.0.is_empty()))
}

pub fn arr_get(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let index = index!(&arr, int!(arg!(args, 1)), arg_opt!(args, 2));

    Ok(arr.0[index].clone())
}

pub fn arr_front(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));

    match arr.0.front() {
        Some(yay) => Ok(yay.clone()),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(index_error(0))
        }
    }
}

pub fn arr_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));

    match arr.0.back() {
        Some(yay) => Ok(yay.clone()),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(index_error(0))
        }
    }
}

pub fn arr_slice(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let start = index_incl!(&arr, int!(arg!(args, 1)), arg_opt!(args, 3));
    let end = index_incl!(&arr, int!(arg!(args, 2)), arg_opt!(args, 3));

    if start > end {
        match arg_opt!(args, 3) {
            Some(fallback) => return Ok(fallback.clone()),
            None => return Err(index_error(end)),
        }
    }

    let (tmp, _) = arr.0.split_at(end);
    Ok(Value::arr(Vector(tmp.skip(start))))
}

pub fn arr_slice_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let start = int!(arg!(args, 1));
    let end = int!(arg!(args, 2));

    let start = index_clamp_inclusive(&arr, start);
    let end = index_clamp_inclusive(&arr, end);

    let (tmp, _) = arr.0.split_at(end);
    Ok(Value::arr(Vector(tmp.skip(start))))
}

pub fn arr_split(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let index = index_incl!(&arr, int!(arg!(args, 1)), arg_opt!(args, 2));

    let (fst, snd) = arr.0.split_at(index);

    Ok(Value::arr_from_vec(vec![Value::arr(Vector(fst)), Value::arr(Vector(snd))]))
}

pub fn arr_split_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let index = int!(arg!(args, 1));
    let index = index_clamp_inclusive(&arr, index);

    let (fst, snd) = arr.0.split_at(index);

    Ok(Value::arr_from_vec(vec![Value::arr(Vector(fst)), Value::arr(Vector(snd))]))
}

pub fn arr_take(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let n = index_incl!(&arr, int!(arg!(args, 1)), arg_opt!(args, 2));

    Ok(Value::arr(Vector(arr.0.take(n))))
}

pub fn arr_take_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let n = int!(arg!(args, 1));
    let n = index_clamp_inclusive(&arr, n);

    Ok(Value::arr(Vector(arr.0.take(n))))
}

pub fn arr_take_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let n = index_incl!(&arr, int!(arg!(args, 1)), arg_opt!(args, 2));
    let len = arr.0.len();

    let (_, snd) = arr.0.split_at(len - n);
    Ok(Value::arr(Vector(snd)))
}

pub fn arr_take_back_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let n = int!(arg!(args, 1));
    let n = index_clamp_inclusive(&arr, n);
    let len = arr.0.len();

    let (_, snd) = arr.0.split_at(len - n);
    Ok(Value::arr(Vector(snd)))
}

pub fn arr_take_while(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let pred = fun!(arg!(args, 1));

    match arr.0.iter().try_fold(ImVector::new(), |acc, elem| {
        match pred.apply(&Value::arr_from_vec(vec![elem.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    let mut new_acc = acc.clone();
                    new_acc.push_back(yay);
                    Ok(new_acc)
                } else {
                    Err(Ok(acc))
                }
            }
            Err(thrown) => Err(Err(thrown)),
        }
    }) {
        Ok(yay) => Ok(Value::arr(Vector(yay))),
        Err(Ok(yay)) => Ok(Value::arr(Vector(yay))),
        Err(Err(thrown)) => Err(thrown),
    }
}

pub fn arr_take_while_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let pred = fun!(arg!(args, 1));

    match arr.0.iter().rev().try_fold(ImVector::new(), |acc, elem| {
        match pred.apply(&Value::arr_from_vec(vec![elem.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    let mut new_acc = acc.clone();
                    new_acc.push_back(yay);
                    Ok(new_acc)
                } else {
                    Err(Ok(acc))
                }
            }
            Err(thrown) => Err(Err(thrown)),
        }
    }) {
        Ok(yay) => Ok(Value::arr(Vector(yay))),
        Err(Ok(yay)) => Ok(Value::arr(Vector(yay))),
        Err(Err(thrown)) => Err(thrown),
    }
}

pub fn arr_skip(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let n = index_incl!(&arr, int!(arg!(args, 1)), arg_opt!(args, 2));

    Ok(Value::arr(Vector(arr.0.skip(n))))
}

pub fn arr_skip_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let n = int!(arg!(args, 1));
    let n = index_clamp_inclusive(&arr, n);

    Ok(Value::arr(Vector(arr.0.skip(n))))
}

pub fn arr_skip_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let n = index_incl!(&arr, int!(arg!(args, 1)), arg_opt!(args, 2));
    let len = arr.0.len();

    let (fst, _) = arr.0.split_at(len - n);
    Ok(Value::arr(Vector(fst)))
}

pub fn arr_skip_back_sat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let n = int!(arg!(args, 1));
    let n = index_clamp_inclusive(&arr, n);
    let len = arr.0.len();

    let (fst, _) = arr.0.split_at(len - n);
    Ok(Value::arr(Vector(fst)))
}

pub fn arr_take_while(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let pred = fun!(arg!(args, 1));

    match arr.0.iter().try_fold(ImVector::new(), |acc, elem| {
        match pred.apply(&Value::arr_from_vec(vec![elem.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    let mut new_acc = acc.clone();
                    new_acc.push_back(yay);
                    Ok(new_acc)
                } else {
                    Err(Ok(acc))
                }
            }
            Err(thrown) => Err(Err(thrown)),
        }
    }) {
        Ok(yay) => Ok(Value::arr(Vector(yay))),
        Err(Ok(yay)) => Ok(Value::arr(Vector(yay))),
        Err(Err(thrown)) => Err(thrown),
    }
}

pub fn arr_take_while_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let pred = fun!(arg!(args, 1));

    match arr.0.iter().rev().try_fold(ImVector::new(), |acc, elem| {
        match pred.apply(&Value::arr_from_vec(vec![elem.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    let mut new_acc = acc.clone();
                    new_acc.push_back(yay);
                    Ok(new_acc)
                } else {
                    Err(Ok(acc))
                }
            }
            Err(thrown) => Err(Err(thrown)),
        }
    }) {
        Ok(yay) => Ok(Value::arr(Vector(yay))),
        Err(Ok(yay)) => Ok(Value::arr(Vector(yay))),
        Err(Err(thrown)) => Err(thrown),
    }
}


// pub fn arr_init(args: Value, _cx: &mut Context) -> Result<Value, Value> {
//     let arr = arr!(arg!(args, 0));
//
//     if arr.0.len() == 0 {
//         match arg_opt!(args, 1) {
//             Some(fallback) => return Ok(fallback.clone()),
//             None => return Err(index_error(0)),
//         }
//     }
//
//     Ok(Value::arr(Vector(arr.0.take(arr.0.len() - 1))))
// }

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
