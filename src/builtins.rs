use im_rc::{OrdMap as ImOrdMap};

use crate::context::Context;
use crate::gc_foreign::{OrdMap, OrdSet, Vector};
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

pub fn coll_full_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-collection-full")),
        ])))
}

pub fn coll_empty_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-collection-empty")),
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

pub fn lookup_error(got: Value) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-lookup")),
        (Value::kw_str("got"), got),
        ])))
}

pub fn index_error(got: usize) -> Value {
    lookup_error(Value::int(got as i64))
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

macro_rules! set {
    ($v:expr) => (
        match &$v {
            Value::Set(set) => set.clone(),
            _ => return Err(type_error(&$v, "set")),
        }
    )
}

macro_rules! map {
    ($v:expr) => (
        match &$v {
            Value::Map(map) => map.clone(),
            _ => return Err(type_error(&$v, "map")),
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

pub fn arr_get(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let index = index!(&arr, int!(arg!(args, 1)), arg_opt!(args, 2));

    Ok(arr.0[index].clone())
}

pub fn arr_insert(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let index = index_incl!(&arr, int!(arg!(args, 1)), arg_opt!(args, 3));
    let elem = arg!(args, 2);

    if arr.0.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut new = arr.0.clone();
    new.insert(index, elem.clone());
    Ok(Value::arr(Vector(new)))
}

pub fn arr_remove(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let index = index!(&arr, int!(arg!(args, 1)), arg_opt!(args, 2));

    let mut new = arr.0.clone();
    let _ = new.remove(index);
    Ok(Value::arr(Vector(new)))
}

pub fn arr_update(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let index = index!(&arr, int!(arg!(args, 1)), arg_opt!(args, 3));
    let elem = arg!(args, 2);

    Ok(Value::arr(Vector(arr.0.update(index, elem))))
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

    let mut tmp = arr.0.clone();
    Ok(Value::arr(Vector(tmp.slice(start..end))))
}

pub fn arr_splice(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let index = index_incl!(&arr, int!(arg!(args, 1)), arg_opt!(args, 3));
    let new = arr!(arg!(args, 2));

    let (mut left, right) = arr.0.split_at(index);
    left.append(new.0);
    left.append(right);

    if left.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::arr(Vector(left)))
}

pub fn arr_concat(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let left = arr!(arg!(args, 0));
    let right = arr!(arg!(args, 1));

    let mut ret = left.0.clone();
    ret.append(right.0);

    if ret.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::arr(Vector(ret)))
}

pub fn arr_iter(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let fun = fun!(arg!(args, 1));

    for elem in arr.0.iter() {
        match fun.apply(&Value::arr_from_vec(vec![elem.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    return Ok(Value::nil());
                }
            }
            Err(thrown) => return Err(thrown),
        }
    }

    Ok(Value::nil())
}

pub fn arr_iter_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let arr = arr!(arg!(args, 0));
    let fun = fun!(arg!(args, 1));

    for elem in arr.0.iter().rev() {
        match fun.apply(&Value::arr_from_vec(vec![elem.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    return Ok(Value::nil());
                }
            }
            Err(thrown) => return Err(thrown),
        }
    }

    Ok(Value::nil())
}

/////////////////////////////////////////////////////////////////////////////

pub fn set_count(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let set = set!(arg!(args, 1));
    Ok(Value::int(set.0.len() as i64))
}

pub fn set_contains(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let set = set!(arg!(args, 0));
    let needle = arg!(args, 1);

    Ok(Value::bool_(set.0.contains(&needle)))
}

pub fn set_min(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let set = set!(arg!(args, 0));

    match set.0.get_min() {
        Some(min) => Ok(min.clone()),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(coll_empty_error())
        }
    }
}

pub fn set_max(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let set = set!(arg!(args, 0));

    match set.0.get_max() {
        Some(min) => Ok(min.clone()),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(coll_empty_error())
        }
    }
}

pub fn set_insert(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let set = set!(arg!(args, 0));
    let new = arg!(args, 1);

    if set.0.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut ret = set.0.clone();
    ret.insert(new.clone());
    Ok(Value::set(OrdSet(ret)))
}

pub fn set_remove(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let set = set!(arg!(args, 0));
    let elem = arg!(args, 1);

    let mut new = set.0.clone();
    let _ = new.remove(&elem);
    Ok(Value::set(OrdSet(new)))
}

pub fn set_union(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let lhs = set!(arg!(args, 0));
    let rhs = set!(arg!(args, 1));

    let ret = lhs.0.union(rhs.0);
    if ret.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }
    Ok(Value::set(OrdSet(ret)))
}

pub fn set_intersection(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let lhs = set!(arg!(args, 0));
    let rhs = set!(arg!(args, 1));

    let ret = lhs.0.intersection(rhs.0);
    Ok(Value::set(OrdSet(ret)))
}

pub fn set_difference(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let lhs = set!(arg!(args, 0));
    let rhs = set!(arg!(args, 1));

    let mut ret = lhs.0.clone();

    for elem in rhs.0.iter() {
        let _ = ret.remove(elem);
    }

    Ok(Value::set(OrdSet(ret)))
}

pub fn set_symmetric_difference(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let lhs = set!(arg!(args, 0));
    let rhs = set!(arg!(args, 1));

    let ret = lhs.0.difference(rhs.0);
    Ok(Value::set(OrdSet(ret)))
}

pub fn set_iter(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let set = set!(arg!(args, 0));
    let fun = fun!(arg!(args, 1));

    for elem in set.0.iter() {
        match fun.apply(&Value::arr_from_vec(vec![elem.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    return Ok(Value::nil());
                }
            }
            Err(thrown) => return Err(thrown),
        }
    }

    Ok(Value::nil())
}

pub fn set_iter_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let set = set!(arg!(args, 0));
    let fun = fun!(arg!(args, 1));

    for elem in set.0.iter().rev() {
        match fun.apply(&Value::arr_from_vec(vec![elem.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    return Ok(Value::nil());
                }
            }
            Err(thrown) => return Err(thrown),
        }
    }

    Ok(Value::nil())
}

/////////////////////////////////////////////////////////////////////////////

pub fn map_count(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 1));
    Ok(Value::int(map.0.len() as i64))
}

pub fn map_get(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));
    let key = arg!(args, 1);

    match map.0.get(&key) {
        Some(val) => Ok(val.clone()),
        None => match arg_opt!(args, 2) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(lookup_error(key))
        }
    }
}

pub fn map_contains(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));
    let key = arg!(args, 1);

    Ok(Value::bool_(map.0.contains_key(&key)))
}

pub fn map_min(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));

    match map.0.get_min() {
        Some(min) => Ok(min.1.clone()),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(coll_empty_error())
        }
    }
}

pub fn map_min_key(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));

    match map.0.get_min() {
        Some(min) => Ok(min.0.clone()),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(coll_empty_error())
        }
    }
}

pub fn map_min_entry(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));

    match map.0.get_min() {
        Some(min) => Ok(Value::arr_from_vec(vec![min.0.clone(), min.1.clone()])),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(coll_empty_error())
        }
    }
}

pub fn map_max(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));

    match map.0.get_max() {
        Some(max) => Ok(max.1.clone()),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(coll_empty_error())
        }
    }
}

pub fn map_max_key(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));

    match map.0.get_max() {
        Some(max) => Ok(max.0.clone()),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(coll_empty_error())
        }
    }
}

pub fn map_max_entry(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));

    match map.0.get_max() {
        Some(max) => Ok(Value::arr_from_vec(vec![max.0.clone(), max.1.clone()])),
        None => match arg_opt!(args, 1) {
            Some(fallback) => Ok(fallback.clone()),
            None => Err(coll_empty_error())
        }
    }
}

pub fn map_insert(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));
    let key = arg!(args, 1);
    let value = arg!(args, 2);

    if map.0.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut ret = map.0.clone();
    ret.insert(key.clone(), value.clone());
    Ok(Value::map(OrdMap(ret)))
}

pub fn map_remove(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));
    let key = arg!(args, 1);

    let mut new = map.0.clone();
    let _ = new.remove(&key);
    Ok(Value::map(OrdMap(new)))
}

pub fn map_union(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let lhs = map!(arg!(args, 0));
    let rhs = map!(arg!(args, 1));

    let ret = lhs.0.union(rhs.0);
    if ret.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }
    Ok(Value::map(OrdMap(ret)))
}

pub fn map_intersection(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let lhs = map!(arg!(args, 0));
    let rhs = map!(arg!(args, 1));

    let ret = lhs.0.intersection(rhs.0);
    Ok(Value::map(OrdMap(ret)))
}

pub fn map_difference(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let lhs = map!(arg!(args, 0));
    let rhs = map!(arg!(args, 1));

    let mut ret = lhs.0.clone();

    for key in rhs.0.keys() {
        let _ = ret.remove(key);
    }

    Ok(Value::map(OrdMap(ret)))
}

pub fn map_symmetric_difference(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let lhs = map!(arg!(args, 0));
    let rhs = map!(arg!(args, 1));

    let ret = lhs.0.difference(rhs.0);
    Ok(Value::map(OrdMap(ret)))
}

pub fn map_iter(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));
    let fun = fun!(arg!(args, 1));

    for entry in map.0.iter() {
        match fun.apply(&Value::arr_from_vec(vec![entry.0.clone(), entry.1.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    return Ok(Value::nil());
                }
            }
            Err(thrown) => return Err(thrown),
        }
    }

    Ok(Value::nil())
}

pub fn map_iter_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
    let map = map!(arg!(args, 0));
    let fun = fun!(arg!(args, 1));

    for entry in map.0.iter().rev() {
        match fun.apply(&Value::arr_from_vec(vec![entry.0.clone(), entry.1.clone()])) {
            Ok(yay) => {
                if yay.truthy() {
                    return Ok(Value::nil());
                }
            }
            Err(thrown) => return Err(thrown),
        }
    }

    Ok(Value::nil())
}

// pub fn set_iter_back(args: Value, _cx: &mut Context) -> Result<Value, Value> {
//     let set = set!(arg!(args, 0));
//     let fun = fun!(arg!(args, 1));
//
//     for elem in set.0.iter().rev() {
//         match fun.apply(&Value::arr_from_vec(vec![elem.clone()])) {
//             Ok(yay) => {
//                 if yay.truthy() {
//                     return Ok(Value::nil());
//                 }
//             }
//             Err(thrown) => return Err(thrown),
//         }
//     }
//
//     Ok(Value::nil())
// }

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
