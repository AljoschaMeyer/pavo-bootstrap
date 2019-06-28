use im_rc::{OrdMap as ImOrdMap, Vector as ImVector};
use ropey::Rope as Ropey;
use math::round;
use nom::types::CompleteStr;

use crate::context::Context;
use crate::gc_foreign::{OrdMap, OrdSet, Vector, Rope};
use crate::value::{Value, Atomic, Id};
use crate::read::{is_id_char, parse_id};

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
        Value::Cell(..) => Value::kw_str("cell"),
        Value::Opaque(_, id) => Value::Id(Id::Symbol(*id)),
    }
}

pub fn num_args_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-num-args")),
        ])))
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

pub fn type_error_(got: &Value, expected: &Value) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-type")),
            (Value::kw_str("expected"), expected.clone()),
            (Value::kw_str("got"), typeof__(got)),
        ])))
}

pub fn type_error(got: &Value, expected: &str) -> Value {
    type_error_(got, &Value::kw_str(expected))
}

pub fn byte_error(got: &Value) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-not-byte")),
            (Value::kw_str("got"), got.clone()),
        ])))
}

pub fn char_error(got: &Value) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-not-unicode-scalar")),
            (Value::kw_str("got"), got.clone()),
        ])))
}

pub fn kw_error(got: &Value) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-kw")),
            (Value::kw_str("got"), got.clone()),
        ])))
}

pub fn id_error(got: &Value) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-identifier")),
            (Value::kw_str("got"), got.clone()),
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
        (Value::kw_str("tag"), Value::kw_str("err-wrap-int")),
        ])))
}

pub fn zero_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-zero")),
        ])))
}

pub fn unwritable_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-not-writable")),
        ])))
}

fn int_to_u64(n: i64) -> Result<u64, Value> {
    if n >= 0 {
        Ok(n as u64)
    } else {
        Err(negative_error(n))
    }
}

fn int_to_u32(n: i64) -> Result<u32, Value> {
    if n >= 0 {
        if n > (std::u32::MAX as i64) {
            Ok(std::u32::MAX)
        } else {
            Ok(n as u32)
        }
    } else {
        Err(negative_error(n))
    }
}

macro_rules! index {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize >= $arr.0.len() {
            return Err(index_error($n as usize));
        } else {
            $n as usize
        }
    )
}

macro_rules! index_incl {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize > $arr.0.len() {
            return Err(index_error($n as usize));
        } else {
            $n as usize
        }
    )
}

macro_rules! index_char {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize >= $arr.0.len_chars() {
            return Err(index_error($n as usize));
        } else {
            $n as usize
        }
    )
}

macro_rules! string_index_byte {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize >= $arr.0.len_bytes() {
            return Err(index_error($n as usize));
        } else {
            $n as usize
        }
    )
}

macro_rules! index_char_incl {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize > $arr.0.len_chars() {
            return Err(index_error($n as usize));
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

macro_rules! bytes {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Bytes(b)) => b.clone(),
            _ => return Err(type_error(&$v, "bytes")),
        }
    )
}

macro_rules! char {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Char(c)) => *c,
            _ => return Err(type_error(&$v, "char")),
        }
    )
}

macro_rules! string {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::String(s)) => s.clone(),
            _ => return Err(type_error(&$v, "string")),
        }
    )
}

macro_rules! float {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Float(n)) => (*n).0.into_inner(),
            _ => return Err(type_error(&$v, "float")),
        }
    )
}

macro_rules! byte {
    ($v:expr) => (
        match int!($v) {
            b@0...255 => b as u8,
            _ => return Err(byte_error(&$v))
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

macro_rules! id {
    ($v:expr) => (
        match &$v {
            Value::Id(Id::User(id)) => id.clone(),
            _ => return Err(type_error(&$v, "identifier")),
        }
    )
}

macro_rules! kw {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Keyword(kw)) => kw.clone(),
            _ => return Err(type_error(&$v, "keyword")),
        }
    )
}

macro_rules! app {
    ($v:expr) => (
        match &$v {
            Value::App(app) => app.clone(),
            _ => return Err(type_error(&$v, "application")),
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

fn num_args(args: &Vector<Value>, expected: usize) -> Result<(), Value> {
    if args.0.len() == expected {
        Ok(())
    } else {
        Err(num_args_error())
    }
}

fn ret_float(x: f64) -> Result<Value, Value> {
    match x.classify() {
        std::num::FpCategory::Nan => Err(Value::kw_str("nan")),
        std::num::FpCategory::Infinite => {
            if x.is_sign_positive() {
                Err(Value::kw_str("inf"))
            } else {
                Err(Value::kw_str("-inf"))
            }
        }
        _ => Ok(Value::float(x))
    }
}

/////////////////////////////////////////////////////////////////////////////

pub fn bool_not(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let b = bool_!(args.0[0]);

    Ok(Value::bool_(!b))
}

pub fn bool_and(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b0 = bool_!(args.0[0]);
    let b1 = bool_!(args.0[1]);

    Ok(Value::bool_(b0 && b1))
}

pub fn bool_or(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b0 = bool_!(args.0[0]);
    let b1 = bool_!(args.0[1]);

    Ok(Value::bool_(b0 || b1))
}

pub fn bool_if(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b0 = bool_!(args.0[0]);
    let b1 = bool_!(args.0[1]);

    Ok(Value::bool_(if b0 { b1 } else { true }))
}

pub fn bool_xor(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b0 = bool_!(args.0[0]);
    let b1 = bool_!(args.0[1]);

    Ok(Value::bool_(b0 != b1))
}

pub fn bool_iff(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b0 = bool_!(args.0[0]);
    let b1 = bool_!(args.0[1]);

    Ok(Value::bool_(b0 == b1))
}

/////////////////////////////////////////////////////////////////////////////

pub fn int_count_ones(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(Value::int(int!(args.0[0]).count_ones() as i64))
}

pub fn int_count_zeros(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(Value::int(int!(args.0[0]).count_zeros() as i64))
}

pub fn int_leading_ones(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);
    Ok(Value::int((!n as u64).leading_zeros() as i64))
}

pub fn int_leading_zeros(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(Value::int(int!(args.0[0]).leading_zeros() as i64))
}

pub fn int_trailing_ones(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);
    Ok(Value::int((!n as u64).trailing_zeros() as i64))
}

pub fn int_trailing_zeros(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(Value::int(int!(args.0[0]).trailing_zeros() as i64))
}

pub fn int_rotate_left(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let by = int_to_u64(int!(args.0[1]))?;
    Ok(Value::int(n.rotate_left(by as u32)))
}

pub fn int_rotate_right(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let by = int_to_u64(int!(args.0[1]))?;
    Ok(Value::int(n.rotate_right(by as u32)))
}

pub fn int_reverse_bytes(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(Value::int(int!(args.0[0]).swap_bytes() as i64))
}

pub fn int_reverse_bits(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(Value::int(int!(args.0[0]).reverse_bits() as i64))
}

pub fn int_add(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    match n.checked_add(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(wrap_error()),
    }
}

pub fn int_sub(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    match n.checked_sub(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(wrap_error()),
    }
}

pub fn int_mul(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    match n.checked_mul(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(wrap_error()),
    }
}

pub fn int_div(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    match n.checked_div_euclid(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(if m == 0 { zero_error() } else { wrap_error() }),
    }
}

pub fn int_div_trunc(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    match n.checked_div(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(if m == 0 { zero_error() } else { wrap_error() }),
    }
}

pub fn int_mod(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    match n.checked_rem_euclid(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(if m == 0 { zero_error() } else { wrap_error() }),
    }
}

pub fn int_mod_trunc(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    match n.checked_rem(m) {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(if m == 0 { zero_error() } else { wrap_error() }),
    }
}

pub fn int_neg(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    match n.checked_neg() {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(wrap_error()),
    }
}

pub fn int_shl(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int_to_u64(int!(args.0[1]))?;

    if m >= 64 {
        Ok(Value::int(0))
    } else {
        Ok(Value::int(n << m))
    }
}

pub fn int_shr(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int_to_u64(int!(args.0[1]))?;

    if m >= 64 {
        Ok(Value::int(0))
    } else {
        Ok(Value::int(n >> m))
    }
}

pub fn int_abs(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    match n.checked_abs() {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(wrap_error()),
    }
}

pub fn int_pow(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int_to_u64(int!(args.0[1]))?;

    match n.checked_pow(m as u32) {
        Some(yay) => Ok(Value::int(yay)),
        None => Err(wrap_error()),
    }
}

pub fn int_add_sat(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    Ok(Value::int(n.saturating_add(m)))
}

pub fn int_sub_sat(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    Ok(Value::int(n.saturating_sub(m)))
}

pub fn int_mul_sat(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    Ok(Value::int(n.saturating_mul(m)))
}

pub fn int_pow_sat(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    Ok(Value::int(n.saturating_pow(m as u32)))
}

pub fn int_add_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    Ok(Value::int(n.wrapping_add(m)))
}

pub fn int_sub_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    Ok(Value::int(n.wrapping_sub(m)))
}

pub fn int_mul_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    Ok(Value::int(n.wrapping_mul(m)))
}

pub fn int_div_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    if m == 0 {
        Err(zero_error())
    } else {
        Ok(Value::int(n.wrapping_div_euclid(m)))
    }
}

pub fn int_div_trunc_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    if m == 0 {
        Err(zero_error())
    } else {
        Ok(Value::int(n.wrapping_div(m)))
    }
}

pub fn int_mod_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    if m == 0 {
        Err(zero_error())
    } else {
        Ok(Value::int(n.wrapping_rem_euclid(m)))
    }
}

pub fn int_mod_trunc_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int!(args.0[1]);

    if m == 0 {
        Err(zero_error())
    } else {
        Ok(Value::int(n.wrapping_rem(m)))
    }
}

pub fn int_neg_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    Ok(Value::int(n.wrapping_neg()))
}

pub fn int_abs_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    Ok(Value::int(n.wrapping_abs()))
}

pub fn int_pow_wrap(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = int!(args.0[0]);
    let m = int_to_u32(int!(args.0[1]))?;

    Ok(Value::int(n.wrapping_pow(m as u32)))
}

pub fn int_signum(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    Ok(Value::int(n.signum()))
}

/////////////////////////////////////////////////////////////////////////////

pub fn bytes_count(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let b = bytes!(args.0[0]);
    Ok(Value::int(b.0.len() as i64))
}

pub fn bytes_get(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b = bytes!(args.0[0]);
    let index = index!(&b, int!(args.0[1]));

    Ok(Value::int(b.0[index] as i64))
}

pub fn bytes_insert(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let b = bytes!(args.0[0]);
    let index = index_incl!(&b, int!(args.0[1]));
    let elem = byte!(args.0[2]);

    if b.0.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut new = b.0.clone();
    new.insert(index, elem.clone());
    Ok(Value::bytes(Vector(new)))
}

pub fn bytes_remove(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b = bytes!(args.0[0]);
    let index = index!(&b, int!(args.0[1]));

    let mut new = b.0.clone();
    let _ = new.remove(index);
    Ok(Value::bytes(Vector(new)))
}

pub fn bytes_update(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let b = bytes!(args.0[0]);
    let index = index!(&b, int!(args.0[1]));
    let elem = byte!(args.0[2]);

    Ok(Value::bytes(Vector(b.0.update(index, elem))))
}

pub fn bytes_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let b = bytes!(args.0[0]);
    let start = index_incl!(&b, int!(args.0[1]));
    let end = index_incl!(&b, int!(args.0[2]));

    if start > end {
        return Err(index_error(end));
    }

    let mut tmp = b.0.clone();
    Ok(Value::bytes(Vector(tmp.slice(start..end))))
}

pub fn bytes_splice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let b = bytes!(args.0[0]);
    let index = index_incl!(&b, int!(args.0[1]));
    let new = bytes!(args.0[2]);

    let (mut left, right) = b.0.split_at(index);
    left.append(new.0);
    left.append(right);

    if left.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::bytes(Vector(left)))
}

pub fn bytes_concat(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let left = bytes!(args.0[0]);
    let right = bytes!(args.0[1]);

    let mut ret = left.0.clone();
    ret.append(right.0);

    if ret.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::bytes(Vector(ret)))
}

pub fn bytes_iter(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b = bytes!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in b.0.iter() {
        match fun.compute(Vector(ImVector::from(vec![Value::int(*elem as i64)])), cx) {
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

pub fn bytes_iter_back(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b = bytes!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in b.0.iter().rev() {
        match fun.compute(Vector(ImVector::from(vec![Value::int(*elem as i64)])), cx) {
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

pub fn int_to_char(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    match std::char::from_u32(n as u32) {
        Some(c) => {
            Ok(Value::char_(c))
        }
        None => Err(char_error(&Value::int(n))),
    }
}

pub fn is_int_to_char(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    match std::char::from_u32(n as u32) {
        Some(c) => {
            Ok(Value::bool_(true))
        }
        None => Ok(Value::bool_(false)),
    }
}

pub fn char_to_int(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let c = char!(args.0[0]);

    Ok(Value::int(c as i64))
}

/////////////////////////////////////////////////////////////////////////////

pub fn str_count(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let s = string!(args.0[0]);

    Ok(Value::int(s.0.len_chars() as i64))
}

pub fn str_count_utf8(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let s = string!(args.0[0]);

    Ok(Value::int(s.0.len_bytes() as i64))
}

pub fn str_get(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let s = string!(args.0[0]);
    let index = index_char!(&s, int!(args.0[1]));

    Ok(Value::char_(s.0.char(index)))
}

pub fn str_get_utf8(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let s = string!(args.0[0]);
    let index = string_index_byte!(&s, int!(args.0[1]));

    Ok(Value::int(s.0.byte(index) as i64))
}

pub fn str_index_char_to_utf8(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let s = string!(args.0[0]);
    let index = index_char!(&s, int!(args.0[1]));

    Ok(Value::int(s.0.char_to_byte(index) as i64))
}

pub fn str_index_utf8_to_char(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let s = string!(args.0[0]);
    let index = string_index_byte!(&s, int!(args.0[1]));

    Ok(Value::int(s.0.byte_to_char(index) as i64))
}

pub fn str_insert(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let s = string!(args.0[0]);
    let index = index_char_incl!(&s, int!(args.0[1]));
    let elem = char!(args.0[2]);

    if s.0.len_bytes() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut new = s.0.clone();
    new.insert_char(index, elem.clone());
    Ok(Value::string(Rope(new)))
}

pub fn str_remove(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let s = string!(args.0[0]);
    let index = index_char!(&s, int!(args.0[1]));

    let mut new = s.0.clone();
    let _ = new.remove(index..index + 1);
    Ok(Value::string(Rope(new)))
}

pub fn str_update(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let s = string!(args.0[0]);
    let index = index_char!(&s, int!(args.0[1]));
    let elem = char!(args.0[2]);

    let mut new = s.0.clone();
    new.remove(index..index + 1);
    new.insert_char(index, elem);

    Ok(Value::string(Rope(new)))
}

pub fn str_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let s = string!(args.0[0]);
    let start = index_char_incl!(&s, int!(args.0[1]));
    let end = index_char_incl!(&s, int!(args.0[2]));

    if start > end {
        return Err(index_error(end));
    }

    let mut tmp = s.0.clone();
    tmp.remove(end..);
    tmp.remove(..start);
    Ok(Value::string(Rope(tmp)))
}

pub fn str_splice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let mut s = string!(args.0[0]);
    let index = index_char_incl!(&s, int!(args.0[1]));
    let new = string!(args.0[2]);

    let right = s.0.split_off(index);
    s.0.append(new.0.clone());
    s.0.append(right);

    if s.0.len_bytes() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::string(s))
}

pub fn str_concat(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let left = string!(args.0[0]);
    let right = string!(args.0[1]);

    let mut ret = left.0.clone();
    ret.append(right.0.clone());

    if ret.len_bytes() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::string(Rope(ret)))
}

pub fn str_iter(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let s = string!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in s.0.chars() {
        match fun.compute(Vector(ImVector::from(vec![Value::char_(elem)])), cx) {
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

pub fn str_iter_back(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    let s = string!(args.0[0]);
    let fun = fun!(args.0[1]);

    // https://github.com/cessen/ropey/issues/18
    let tmp: Vec<char> = s.0.chars().collect();

    for elem in tmp.iter().rev() {
        match fun.compute(Vector(ImVector::from(vec![Value::char_(*elem)])), cx) {
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

pub fn str_iter_utf8(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let s = string!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in s.0.bytes() {
        match fun.compute(Vector(ImVector::from(vec![Value::int(elem as i64)])), cx) {
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

pub fn str_iter_utf8_back(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    let s = string!(args.0[0]);
    let fun = fun!(args.0[1]);

    // https://github.com/cessen/ropey/issues/18
    let tmp: Vec<u8> = s.0.bytes().collect();

    for elem in tmp.iter().rev() {
        match fun.compute(Vector(ImVector::from(vec![Value::int(*elem as i64)])), cx) {
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

pub fn float_add(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = float!(args.0[0]);
    let m = float!(args.0[1]);

    ret_float(n + m)
}

pub fn float_sub(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = float!(args.0[0]);
    let m = float!(args.0[1]);

    ret_float(n - m)
}

pub fn float_mul(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = float!(args.0[0]);
    let m = float!(args.0[1]);

    ret_float(n * m)
}

pub fn float_div(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = float!(args.0[0]);
    let m = float!(args.0[1]);

    ret_float(n / m)
}

pub fn float_mul_add(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let n = float!(args.0[0]);
    let m = float!(args.0[1]);
    let o = float!(args.0[2]);

    ret_float(n.mul_add(m, o))
}

pub fn float_neg(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(-n)
}

pub fn float_floor(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.floor())
}

pub fn float_ceil(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.ceil())
}

pub fn float_round(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let x = float!(args.0[0]);

    ret_float(round::half_to_even(x, 0))
}

pub fn float_trunc(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.trunc())
}

pub fn float_fract(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.fract())
}

pub fn float_abs(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.abs())
}

pub fn float_signum(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    if n == 0.0f64 {
        return Ok(Value::float(0.0));
    } else {
        ret_float(n.signum())
    }
}

pub fn float_pow(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = float!(args.0[0]);
    let m = float!(args.0[1]);

    ret_float(n.powf(m))
}

pub fn float_sqrt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.sqrt())
}

pub fn float_exp(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.exp())
}

pub fn float_exp2(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.exp2())
}

pub fn float_ln(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.ln())
}

pub fn float_log2(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.log2())
}

pub fn float_log10(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.log10())
}

pub fn float_hypot(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = float!(args.0[0]);
    let m = float!(args.0[1]);

    ret_float(n.hypot(m))
}

pub fn float_sin(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.sin())
}

pub fn float_cos(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.cos())
}

pub fn float_tan(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.tan())
}

pub fn float_asin(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.asin())
}

pub fn float_acos(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.acos())
}

pub fn float_atan(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.atan())
}

pub fn float_atan2(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let n = float!(args.0[0]);
    let m = float!(args.0[1]);

    ret_float(n.atan2(m))
}

pub fn float_exp_m1(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.exp_m1())
}

pub fn float_ln_1p(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.ln_1p())
}

pub fn float_sinh(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.sinh())
}

pub fn float_cosh(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.cosh())
}

pub fn float_tanh(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.tanh())
}

pub fn float_asinh(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.asinh())
}

pub fn float_acosh(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.acosh())
}

pub fn float_atanh(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.atanh())
}

pub fn float_is_normal(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    Ok(Value::bool_(n.is_normal()))
}

pub fn float_to_degrees(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.to_degrees())
}

pub fn float_to_radians(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    ret_float(n.to_radians())
}

pub fn float_to_int(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    if n >= (std::i64::MAX as f64) {
        return Ok(Value::int(std::i64::MAX));
    } else if n <= (std::i64::MIN as f64) {
        return Ok(Value::int(std::i64::MIN));
    } else {
        return Ok(Value::int(n as i64));
    }
}

pub fn int_to_float(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    Ok(Value::float(n as f64))
}

pub fn float_to_bits(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    if n == 0.0 { // also catches negative zero, which is why we need a special case
        Ok(Value::int(0))
    } else {
        Ok(Value::int(n.to_bits() as i64))
    }
}

pub fn bits_to_float(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    ret_float(f64::from_bits(n as u64))
}

pub fn is_bits_to_float(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    Ok(Value::bool_(f64::from_bits(n as u64).is_finite()))
}

/////////////////////////////////////////////////////////////////////////////

pub fn str_to_id(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let a = &args.0[0];
    let s = string!(a);

    if s.0.len_bytes() == 0 || s.0.len_bytes() > 255 {
        return Err(id_error(a));
    }

    match parse_id(CompleteStr(&s.0.to_string())) {
        Ok(v) => match v.as_user_id() {
            Some(_) => return Ok(v),
            None => return Err(id_error(a)),
        },
        Err(_) => {
            return Err(id_error(a));
        }
    }
}

pub fn is_str_to_id(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    match str_to_id(args, _cx) {
        Ok(_) => Ok(Value::bool_(true)),
        Err(_) => Ok(Value::bool_(false)),
    }
}

pub fn id_to_str(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let id = id!(args.0[0]);
    Ok(Value::string_from_str(&id))
}

/////////////////////////////////////////////////////////////////////////////

pub fn str_to_kw(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let a = &args.0[0];
    let s = string!(a);

    if s.0.len_bytes() == 0 || s.0.len_bytes() > 255 {
        return Err(kw_error(a));
    }

    for c in s.0.chars() {
        if !is_id_char(c) {
            return Err(kw_error(a));
        }
    }

    Ok(Value::kw(s.0.to_string()))
}

pub fn is_str_to_kw(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let s = string!(args.0[0]);

    if s.0.len_bytes() == 0 || s.0.len_bytes() > 255 {
        return Ok(Value::bool_(false));
    }

    for c in s.0.chars() {
        if !is_id_char(c) {
            return Ok(Value::bool_(false));
        }
    }

    return Ok(Value::bool_(true));
}

pub fn kw_to_str(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let kw = kw!(args.0[0]);
    Ok(Value::string_from_str(&kw))
}

/////////////////////////////////////////////////////////////////////////////

pub fn arr_count(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let arr = arr!(args.0[0]);
    Ok(Value::int(arr.0.len() as i64))
}

pub fn arr_get(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[0]);
    let index = index!(&arr, int!(args.0[1]));

    Ok(arr.0[index].clone())
}

pub fn arr_insert(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let arr = arr!(args.0[0]);
    let index = index_incl!(&arr, int!(args.0[1]));
    let elem = &args.0[2];

    if arr.0.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut new = arr.0.clone();
    new.insert(index, elem.clone());
    Ok(Value::arr(Vector(new)))
}

pub fn arr_remove(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[0]);
    let index = index!(&arr, int!(args.0[1]));

    let mut new = arr.0.clone();
    let _ = new.remove(index);
    Ok(Value::arr(Vector(new)))
}

pub fn arr_update(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let arr = arr!(args.0[0]);
    let index = index!(&arr, int!(args.0[1]));
    let elem = &args.0[2];

    Ok(Value::arr(Vector(arr.0.update(index, elem.clone()))))
}

pub fn arr_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let arr = arr!(args.0[0]);
    let start = index_incl!(&arr, int!(args.0[1]));
    let end = index_incl!(&arr, int!(args.0[2]));

    if start > end {
        return Err(index_error(end));
    }

    let mut tmp = arr.0.clone();
    Ok(Value::arr(Vector(tmp.slice(start..end))))
}

pub fn arr_splice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let arr = arr!(args.0[0]);
    let index = index_incl!(&arr, int!(args.0[1]));
    let new = arr!(args.0[2]);

    let (mut left, right) = arr.0.split_at(index);
    left.append(new.0);
    left.append(right);

    if left.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::arr(Vector(left)))
}

pub fn arr_concat(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let left = arr!(args.0[0]);
    let right = arr!(args.0[1]);

    let mut ret = left.0.clone();
    ret.append(right.0);

    if ret.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::arr(Vector(ret)))
}

pub fn arr_iter(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in arr.0.iter() {
        match fun.compute(Vector(ImVector::from(vec![elem.clone()])), cx) {
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

pub fn arr_iter_back(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in arr.0.iter().rev() {
        match fun.compute(Vector(ImVector::from(vec![elem.clone()])), cx) {
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

pub fn app_count(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let app = app!(args.0[0]);
    Ok(Value::int(app.0.len() as i64))
}

pub fn app_get(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let app = app!(args.0[0]);
    let index = index!(&app, int!(args.0[1]));

    Ok(app.0[index].clone())
}

pub fn app_insert(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let app = app!(args.0[0]);
    let index = index_incl!(&app, int!(args.0[1]));
    let elem = args.0[2].clone();

    if app.0.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut new = app.0.clone();
    new.insert(index, elem);
    Ok(Value::app(Vector(new)))
}

pub fn app_remove(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let app = app!(args.0[0]);
    let index = index!(&app, int!(args.0[1]));

    let mut new = app.0.clone();
    let _ = new.remove(index);
    Ok(Value::app(Vector(new)))
}

pub fn app_update(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let app = app!(args.0[0]);
    let index = index!(&app, int!(args.0[1]));
    let elem = args.0[2].clone();

    Ok(Value::app(Vector(app.0.update(index, elem))))
}

pub fn app_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let app = app!(args.0[0]);
    let start = index_incl!(&app, int!(args.0[1]));
    let end = index_incl!(&app, int!(args.0[2]));

    if start > end {
        return Err(index_error(end));
    }

    let mut tmp = app.0.clone();
    Ok(Value::app(Vector(tmp.slice(start..end))))
}

pub fn app_splice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let app = app!(args.0[0]);
    let index = index_incl!(&app, int!(args.0[1]));
    let new = app!(args.0[2]);

    let (mut left, right) = app.0.split_at(index);
    left.append(new.0);
    left.append(right);

    if left.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::app(Vector(left)))
}

pub fn app_concat(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let left = app!(args.0[0]);
    let right = app!(args.0[1]);

    let mut ret = left.0.clone();
    ret.append(right.0);

    if ret.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    Ok(Value::app(Vector(ret)))
}

pub fn app_iter(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let app = app!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in app.0.iter() {
        match fun.compute(Vector(ImVector::from(vec![elem.clone()])), cx) {
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

pub fn app_iter_back(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let app = app!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in app.0.iter().rev() {
        match fun.compute(Vector(ImVector::from(vec![elem.clone()])), cx) {
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

pub fn app_apply(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let a = app!(args.0[0]);

    if a.0.len() == 0 {
        return Err(lookup_error(Value::int(0)));
    } else {
        match a.0[0].as_fun() {
            None => return Err(type_error(&a.0[0], "function")),
            Some(fun) => fun.compute(Vector(a.0.skip(1)), cx),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

pub fn set_count(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let set = set!(args.0[0]);
    Ok(Value::int(set.0.len() as i64))
}

pub fn set_contains(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let needle = args.0[1].clone();

    Ok(Value::bool_(set.0.contains(&needle)))
}

pub fn set_min(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let set = set!(args.0[0]);

    match set.0.get_min() {
        Some(min) => Ok(min.clone()),
        None => Err(coll_empty_error()),
    }
}

pub fn set_max(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let set = set!(args.0[0]);

    match set.0.get_max() {
        Some(min) => Ok(min.clone()),
        None => Err(coll_empty_error()),
    }
}

pub fn set_insert(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let new = args.0[1].clone();

    if set.0.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut ret = set.0.clone();
    ret.insert(new.clone());
    Ok(Value::set(OrdSet(ret)))
}

pub fn set_remove(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let elem = args.0[1].clone();

    let mut new = set.0.clone();
    let _ = new.remove(&elem);
    Ok(Value::set(OrdSet(new)))
}

pub fn set_union(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let lhs = set!(args.0[0]);
    let rhs = set!(args.0[1]);

    let ret = lhs.0.union(rhs.0);
    if ret.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }
    Ok(Value::set(OrdSet(ret)))
}

pub fn set_intersection(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let lhs = set!(args.0[0]);
    let rhs = set!(args.0[1]);

    let ret = lhs.0.intersection(rhs.0);
    Ok(Value::set(OrdSet(ret)))
}

pub fn set_difference(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let lhs = set!(args.0[0]);
    let rhs = set!(args.0[1]);

    let mut ret = lhs.0.clone();

    for elem in rhs.0.iter() {
        let _ = ret.remove(elem);
    }

    Ok(Value::set(OrdSet(ret)))
}

pub fn set_symmetric_difference(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let lhs = set!(args.0[0]);
    let rhs = set!(args.0[1]);

    let ret = lhs.0.difference(rhs.0);
    Ok(Value::set(OrdSet(ret)))
}

pub fn set_iter(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in set.0.iter() {
        match fun.compute(Vector(ImVector::from(vec![elem.clone()])), cx) {
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

pub fn set_iter_back(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let fun = fun!(args.0[1]);

    for elem in set.0.iter().rev() {
        match fun.compute(Vector(ImVector::from(vec![elem.clone()])), cx) {
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

pub fn map_count(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);
    Ok(Value::int(map.0.len() as i64))
}

pub fn map_get(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let key = args.0[1].clone();

    match map.0.get(&key) {
        Some(val) => Ok(val.clone()),
        None => Err(lookup_error(key)),
    }
}

pub fn map_contains(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let key = args.0[1].clone();

    Ok(Value::bool_(map.0.contains_key(&key)))
}

pub fn map_min(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);

    match map.0.get_min() {
        Some(min) => Ok(min.1.clone()),
        None => Err(coll_empty_error()),
    }
}

pub fn map_min_key(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);

    match map.0.get_min() {
        Some(min) => Ok(min.0.clone()),
        None => Err(coll_empty_error()),
    }
}

pub fn map_min_entry(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);

    match map.0.get_min() {
        Some(min) => Ok(Value::arr_from_vec(vec![min.0.clone(), min.1.clone()])),
        None => Err(coll_empty_error()),
    }
}

pub fn map_max(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);

    match map.0.get_max() {
        Some(max) => Ok(max.1.clone()),
        None => Err(coll_empty_error()),
    }
}

pub fn map_max_key(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);

    match map.0.get_max() {
        Some(max) => Ok(max.0.clone()),
        None => Err(coll_empty_error()),
    }
}

pub fn map_max_entry(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);

    match map.0.get_max() {
        Some(max) => Ok(Value::arr_from_vec(vec![max.0.clone(), max.1.clone()])),
        None => Err(coll_empty_error()),
    }
}

pub fn map_insert(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let map = map!(args.0[0]);
    let key = args.0[1].clone();
    let value = args.0[2].clone();

    if map.0.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }

    let mut ret = map.0.clone();
    ret.insert(key.clone(), value.clone());
    Ok(Value::map(OrdMap(ret)))
}

pub fn map_remove(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let key = args.0[1].clone();

    let mut new = map.0.clone();
    let _ = new.remove(&key);
    Ok(Value::map(OrdMap(new)))
}

pub fn map_union(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let lhs = map!(args.0[0]);
    let rhs = map!(args.0[1]);

    let ret = lhs.0.union(rhs.0);
    if ret.len() >= (i64::max as usize) {
        return Err(coll_full_error());
    }
    Ok(Value::map(OrdMap(ret)))
}

pub fn map_intersection(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let lhs = map!(args.0[0]);
    let rhs = map!(args.0[1]);

    let ret = lhs.0.intersection(rhs.0);
    Ok(Value::map(OrdMap(ret)))
}

pub fn map_difference(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let lhs = map!(args.0[0]);
    let rhs = map!(args.0[1]);

    let mut ret = lhs.0.clone();

    for key in rhs.0.keys() {
        let _ = ret.remove(key);
    }

    Ok(Value::map(OrdMap(ret)))
}

pub fn map_symmetric_difference(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let lhs = map!(args.0[0]);
    let rhs = map!(args.0[1]);

    let ret = lhs.0.difference(rhs.0);
    Ok(Value::map(OrdMap(ret)))
}

pub fn map_iter(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let fun = fun!(args.0[1]);

    for entry in map.0.iter() {
        match fun.compute(Vector(ImVector::from(vec![entry.0.clone(), entry.1.clone()])), cx) {
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

pub fn map_iter_back(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let fun = fun!(args.0[1]);

    for entry in map.0.iter().rev() {
        match fun.compute(Vector(ImVector::from(vec![entry.0.clone(), entry.1.clone()])), cx) {
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

pub fn symbol(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 0)?;
    Ok(Value::symbol(cx))
}

/////////////////////////////////////////////////////////////////////////////

pub fn cell(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(Value::cell(&args.0[0], cx))
}

pub fn cell_get(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    match args.0[0].as_cell() {
        None => Err(type_error(&args.0[0], "cell")),
        Some(inner) => Ok(inner.borrow().clone()),
    }
}

pub fn cell_set(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    match args.0[0].as_cell() {
        None => Err(type_error(&args.0[0], "cell")),
        Some(inner) => {
            *inner.borrow_mut() = args.0[1].clone();
            Ok(Value::nil())
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

pub fn opaque(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 0)?;

    let s = Value::symbol(cx);
    let id = s.as_symbol().unwrap();

    return Ok(Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("type"), s),
            (Value::kw_str("hide"), Value::hide(id, cx)),
            (Value::kw_str("unhide"), Value::unhide(id, cx)),
        ]))));
}

/////////////////////////////////////////////////////////////////////////////

pub fn pavo_cmp(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;

    match args.0[0].cmp(&args.0[1]) {
        std::cmp::Ordering::Less => Ok(Value::kw_str("<")),
        std::cmp::Ordering::Equal => Ok(Value::kw_str("=")),
        std::cmp::Ordering::Greater => Ok(Value::kw_str(">")),
    }
}

pub fn pavo_eq(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    Ok(Value::bool_(args.0[0] == args.0[1]))
}

// pub fn pavo_lt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
//     Ok(Value::bool_(args.0[0] < args.0[1]))
// }
//
// pub fn pavo_lte(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
//     Ok(Value::bool_(args.0[0] <= args.0[1]))
// }
//
// pub fn pavo_gt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
//     Ok(Value::bool_(args.0[0] > args.0[1]))
// }
//
// pub fn pavo_gte(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
//     Ok(Value::bool_(args.0[0] >= args.0[1]))
// }
//
// /////////////////////////////////////////////////////////////////////////////
//
// pub fn write(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
//     let mut buf = String::new();
//
//     write_(&args.0[0], &mut buf)?;
//     if buf.len() >= i64::max as usize {
//         return Err(coll_full_error());
//     }
//
//     Ok(Value::string(Rope(Ropey::from(&buf[..]))))
// }

/////////////////////////////////////////////////////////////////////////////

pub fn typeof_(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(typeof__(&args.0[0]))
}

// pub fn is_truthy(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
//     Ok(match args.0[0] {
//         Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(false)) => Value::bool_(false),
//         _ => Value::bool_(true),
//     })
// }
//
// pub fn diverge(_args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
//     panic!("Called diverge")
// }
//
// /////////////////////////////////////////////////////////////////////////////

pub fn macro_quote(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;

    Ok(Value::app_from_vec(vec![Value::id_str("sf-quote"), args.0[0].clone()]))
}

pub fn macro_do(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    let mut tmp = args.clone();
    tmp.0.push_front(Value::id_str("sf-do"));
    Ok(Value::app(tmp))
}

pub fn macro_set_bang(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    let mut tmp = args.clone();
    tmp.0.push_front(Value::id_str("sf-set!"));
    Ok(Value::app(tmp))
}

pub fn macro_throw(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    if args.0.len() == 0 {
        Ok(Value::app_from_vec(vec![Value::id_str("sf-throw"), Value::nil()]))
    } else {
        num_args(&args, 1)?;
        Ok(Value::app_from_vec(vec![Value::id_str("sf-throw"), args.0[0].clone()]))
    }
}

// TODO proper implementation (if-else like, inserting nil as final else)
pub fn macro_if(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    let mut tmp = args.clone();
    tmp.0.push_front(Value::id_str("sf-if"));
    Ok(Value::app(tmp))
}

pub fn macro_let(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;

    Ok(Value::app_from_vec(vec![
            Value::app_from_vec(vec![
                    Value::id_str("fn"),
                    Value::arr_from_vec(vec![args.0[0].clone()]),
                    args.0[2].clone(),
                ]),
            args.0[1].clone(),
        ]))
}

// TODO: proper implementation (named recursion, pattern matching)
pub fn macro_fn(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;

    Ok(Value::app_from_vec(vec![Value::id_str("sf-lambda"), args.0[0].clone(), args.0[1].clone()]))
}

//////////////////////////////////////////////////////////////////////////////

pub fn write_(v: &Value, out: &mut String) -> Result<(), Value> {
    match v {
        Value::Atomic(Atomic::Nil) => Ok(out.push_str("nil")),
        Value::Atomic(Atomic::Bool(true)) => Ok(out.push_str("true")),
        Value::Atomic(Atomic::Bool(false)) => Ok(out.push_str("false")),
        Value::Atomic(Atomic::Int(n)) => Ok(out.push_str(&n.to_string())),
        Value::Atomic(Atomic::Float(n)) => unimplemented!(),
        Value::Atomic(Atomic::Char('\\')) => Ok(out.push_str("'\\\\'")),
        Value::Atomic(Atomic::Char('\'')) => Ok(out.push_str("'\\''")),
        Value::Atomic(Atomic::Char('\t')) => Ok(out.push_str("'\\t'")),
        Value::Atomic(Atomic::Char('\n')) => Ok(out.push_str("'\\n'")),
        Value::Atomic(Atomic::Char(other)) => {
            out.push('\'');
            out.push(*other);
            out.push('\'');
            Ok(())
        }
        Value::Atomic(Atomic::String(chars)) => {
            out.push('"');
            for c in chars.0.chars() {
                match c {
                    '\\' => out.push_str("\\"),
                    '\"' => out.push_str("\""),
                    '\n' => out.push_str("\\"),
                    '\t' => out.push_str("\\"),
                    _ => out.push(c),
                }
            }
            out.push('"');
            Ok(())
        }
        Value::Atomic(Atomic::Bytes(bytes)) => {
            out.push_str("@[");
            for (i, b) in bytes.0.iter().enumerate() {
                out.push_str(&b.to_string());
                if i + 1 < bytes.0.len() {
                    out.push(' ');
                }
            }
            out.push(']');
            Ok(())
        }
        Value::Atomic(Atomic::Keyword(kw)) => {
            out.push(':');
            out.push_str(kw);
            Ok(())
        }
        Value::Id(Id::User(id)) => Ok(out.push_str(id)),
        Value::Arr(arr) => {
            out.push_str("[");
            for (i, v) in arr.0.iter().enumerate() {
                let _ = write_(v, out)?;
                if i + 1 < arr.0.len() {
                    out.push(' ');
                }
            }
            out.push_str("]");
            Ok(())
        }
        Value::App(app) => {
            out.push_str("(");
            for (i, v) in app.0.iter().enumerate() {
                let _ = write_(v, out)?;
                if i + 1 < app.0.len() {
                    out.push(' ');
                }
            }
            out.push_str(")");
            Ok(())
        }
        Value::Set(s) => {
            out.push_str("@{");
            for (i, v) in s.0.iter().enumerate() {
                let _ = write_(v, out)?;
                if i + 1 < s.0.len() {
                    out.push(' ');
                }
            }
            out.push_str("}");
            Ok(())
        }
        Value::Map(m) => {
            out.push_str("{");
            for (i, v) in m.0.iter().enumerate() {
                let _ = write_(&v.0, out)?;
                out.push(' ');
                let _ = write_(&v.1, out)?;
                if i + 1 < m.0.len() {
                    out.push(' ');
                }
            }
            out.push_str("}");
            Ok(())
        }
        Value::Id(Id::Symbol(..)) | Value::Fun(..)
        | Value::Cell(..) | Value::Opaque(..) => Err(unwritable_error()),
    }
}
