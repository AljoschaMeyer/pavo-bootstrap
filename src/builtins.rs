use std::collections::HashMap;

use im_rc::{OrdMap as ImOrdMap, Vector as ImVector, OrdSet as ImOrdSet};
use ropey::Rope as Ropey;
use math::round;
use nom::types::CompleteStr;
use ryu_ecmascript::Buffer;

use crate::context::Context;
use crate::gc_foreign::{OrdMap, OrdSet, Vector, Rope};
use crate::value::{Value, Atomic, Id, Opaque, BuiltinOpaque, Builtin, self, Fun};
use crate::read::{is_id_char, parse_id, read as read_};
use crate::expand::expand as expand_;
use crate::check::check as check_;
use crate::env;
use crate::macros;
use crate::special_forms::to_code;
use crate::compile::compile as compile_;
use crate::opaques::{set_cursor, map_cursor};

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
        Value::Opaque(_, Opaque::User(_, id)) => Value::Id(Id::Symbol(*id)),
        Value::Opaque(_, Opaque::Builtin(o)) => Value::Id(Id::Symbol(o.type_id())),
    }
}

pub fn utf8_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-utf8")),
        ])))
}

pub fn static_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-static")),
        ])))
}

pub fn eval_error(err: Value) -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-eval")),
            (Value::kw_str("cause"), err),
        ])))
}

pub fn expand_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-expand")),
        ])))
}

pub fn not_expression_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-not-expression")),
        ])))
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

pub fn type_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-type")),
        ])))
}

pub fn byte_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-not-byte")),
        ])))
}

pub fn char_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-not-unicode-scalar")),
        ])))
}

pub fn kw_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-kw")),
        ])))
}

pub fn id_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
            (Value::kw_str("tag"), Value::kw_str("err-identifier")),
        ])))
}

pub fn lookup_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-lookup")),
        ])))
}

pub fn index_error() -> Value {
    lookup_error()
}

pub fn negative_error() -> Value {
    Value::map(OrdMap(ImOrdMap::from(vec![
        (Value::kw_str("tag"), Value::kw_str("err-negative")),
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
        Err(negative_error())
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
        Err(negative_error())
    }
}

macro_rules! index {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize >= $arr.0.len() {
            return Err(index_error());
        } else {
            $n as usize
        }
    )
}

macro_rules! index_incl {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize > $arr.0.len() {
            return Err(index_error());
        } else {
            $n as usize
        }
    )
}

macro_rules! index_char {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize >= $arr.0.len_chars() {
            return Err(index_error());
        } else {
            $n as usize
        }
    )
}

macro_rules! string_index_byte {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize >= $arr.0.len_bytes() {
            return Err(index_error());
        } else {
            $n as usize
        }
    )
}

macro_rules! index_char_incl {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize > $arr.0.len_chars() {
            return Err(index_error());
        } else {
            $n as usize
        }
    )
}

macro_rules! string_index_byte_incl {
    ($arr:expr, $n:expr) => (
        if $n < 0 || $n as usize > $arr.0.len_bytes() {
            return Err(index_error());
        } else {
            $n as usize
        }
    )
}

macro_rules! bool_ {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Bool(b)) => *b,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! int {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Int(n)) => *n,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! bytes {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Bytes(b)) => b.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! char {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Char(c)) => *c,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! string {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::String(s)) => s.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! float {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Float(n)) => (*n).0.into_inner(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! byte {
    ($v:expr) => (
        match int!($v) {
            b@0...255 => b as u8,
            _ => return Err(byte_error())
        }
    )
}

macro_rules! arr {
    ($v:expr) => (
        match &$v {
            Value::Arr(arr) => arr.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! fun {
    ($v:expr) => (
        match &$v {
            Value::Fun(fun) => fun.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! id {
    ($v:expr) => (
        match &$v {
            Value::Id(id) => id.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! user_id {
    ($v:expr) => (
        match &$v {
            Value::Id(Id::User(id)) => id.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! kw {
    ($v:expr) => (
        match &$v {
            Value::Atomic(Atomic::Keyword(kw)) => kw.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! app {
    ($v:expr) => (
        match &$v {
            Value::App(app) => app.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! set {
    ($v:expr) => (
        match &$v {
            Value::Set(set) => set.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! map {
    ($v:expr) => (
        match &$v {
            Value::Map(map) => map.clone(),
            _ => return Err(type_error()),
        }
    )
}

macro_rules! cursor_arr {
    ($v:expr) => (
        match &$v {
            Value::Opaque(_, Opaque::Builtin(BuiltinOpaque::CursorArr(cell))) => cell,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! cursor_app {
    ($v:expr) => (
        match &$v {
            Value::Opaque(_, Opaque::Builtin(BuiltinOpaque::CursorApp(cell))) => cell,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! cursor_bytes {
    ($v:expr) => (
        match &$v {
            Value::Opaque(_, Opaque::Builtin(BuiltinOpaque::CursorBytes(cell))) => cell,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! cursor_str {
    ($v:expr) => (
        match &$v {
            Value::Opaque(_, Opaque::Builtin(BuiltinOpaque::CursorStringChars(cell))) => cell,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! cursor_str_utf8 {
    ($v:expr) => (
        match &$v {
            Value::Opaque(_, Opaque::Builtin(BuiltinOpaque::CursorStringUtf8(cell))) => cell,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! cursor_set {
    ($v:expr) => (
        match &$v {
            Value::Opaque(_, Opaque::Builtin(BuiltinOpaque::CursorSet(cell))) => cell,
            _ => return Err(type_error()),
        }
    )
}

macro_rules! cursor_map {
    ($v:expr) => (
        match &$v {
            Value::Opaque(_, Opaque::Builtin(BuiltinOpaque::CursorMap(cell))) => cell,
            _ => return Err(type_error()),
        }
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

pub fn bytes_split(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let b = bytes!(args.0[0]);
    let i = index_incl!(&b, int!(args.0[1]));

    let (left, right) = b.0.split_at(i);

    return Ok(Value::arr_from_vec(vec![
        Value::bytes(Vector(left)),
        Value::bytes(Vector(right)),
        ]));
}

pub fn bytes_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let b = bytes!(args.0[0]);
    let start = index_incl!(&b, int!(args.0[1]));
    let end = index_incl!(&b, int!(args.0[2]));

    if start > end {
        return Err(index_error());
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

pub fn bytes_cursor(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let bytes = bytes!(args.0[0]);
    let index = index_incl!(&bytes, int!(args.0[1]));

    return Ok(Value::cursor_bytes(bytes, index, cx));
}

pub fn cursor_bytes_next(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_bytes = cursor_bytes!(args.0[0]);

    match (*cursor_bytes.borrow_mut()).next() {
        Some(b) => return Ok(Value::int(b as i64)),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

pub fn cursor_bytes_prev(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_bytes = cursor_bytes!(args.0[0]);

    match (*cursor_bytes.borrow_mut()).prev() {
        Some(b) => return Ok(Value::int(b as i64)),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

/////////////////////////////////////////////////////////////////////////////

pub fn int_to_char(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    match std::char::from_u32(n as u32) {
        Some(c) => {
            Ok(Value::char_(c))
        }
        None => Err(char_error()),
    }
}

pub fn is_int_to_char(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = int!(args.0[0]);

    match std::char::from_u32(n as u32) {
        Some(_) => {
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

pub fn str_to_bytes(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let s = string!(args.0[0]);

    return Ok(Value::bytes(Vector(s.0.bytes().collect())));
}

pub fn bytes_to_str(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let b = bytes!(args.0[0]);

    let tmp: Vec<u8> = b.0.iter().map(|byte| *byte).collect();
    match std::str::from_utf8(&tmp) {
        Ok(yay) => return Ok(Value::string(Rope(Ropey::from_str(yay)))),
        Err(_) => return Err(utf8_error()),
    }
}

pub fn is_bytes_to_str(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let b = bytes!(args.0[0]);

    let tmp: Vec<u8> = b.0.iter().map(|byte| *byte).collect();
    match std::str::from_utf8(&tmp) {
        Ok(_) => return Ok(Value::bool_(true)),
        Err(_) => return Ok(Value::bool_(false)),
    }
}

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

pub fn str_split(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let s = string!(args.0[0]);
    let i = index_char_incl!(&s, int!(args.0[1]));

    let mut left = s.0.clone();
    let right = left.split_off(i);

    return Ok(Value::arr_from_vec(vec![
        Value::string(Rope(left)),
        Value::string(Rope(right)),
        ]));
}

pub fn str_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let s = string!(args.0[0]);
    let start = index_char_incl!(&s, int!(args.0[1]));
    let end = index_char_incl!(&s, int!(args.0[2]));

    if start > end {
        return Err(index_error());
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

pub fn str_cursor(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let string = string!(args.0[0]);
    let index = index_char_incl!(&string, int!(args.0[1]));

    return Ok(Value::cursor_str(string, index, cx));
}

pub fn cursor_str_next(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_str = cursor_str!(args.0[0]);

    match (*cursor_str.borrow_mut()).next_char() {
        Some(c) => return Ok(Value::char_(c)),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

pub fn cursor_str_prev(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_str = cursor_str!(args.0[0]);

    match (*cursor_str.borrow_mut()).prev_char() {
        Some(c) => return Ok(Value::char_(c)),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

pub fn str_cursor_utf8(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let string = string!(args.0[0]);
    let index = string_index_byte_incl!(&string, int!(args.0[1]));

    return Ok(Value::cursor_str_utf8(string, index, cx));
}

pub fn cursor_str_utf8_next(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_str = cursor_str_utf8!(args.0[0]);

    match (*cursor_str.borrow_mut()).next_byte() {
        Some(b) => return Ok(Value::int(b as i64)),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

pub fn cursor_str_utf8_prev(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_str = cursor_str_utf8!(args.0[0]);

    match (*cursor_str.borrow_mut()).prev_byte() {
        Some(b) => return Ok(Value::int(b as i64)),
        None => return Err(Value::kw_str("cursor-end"))
    }
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

pub fn float_is_integral(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let n = float!(args.0[0]);

    Ok(Value::bool_(n.round() == n))
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
        return Err(id_error());
    }

    match parse_id(CompleteStr(&s.0.to_string())) {
        Ok(v) => match v.as_user_id() {
            Some(_) => return Ok(v),
            None => return Err(id_error()),
        },
        Err(_) => {
            return Err(id_error());
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
    let id = user_id!(args.0[0]);
    Ok(Value::string_from_str(&id))
}

/////////////////////////////////////////////////////////////////////////////

pub fn str_to_kw(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let a = &args.0[0];
    let s = string!(a);

    if s.0.len_bytes() == 0 || s.0.len_bytes() > 255 {
        return Err(kw_error());
    }

    for c in s.0.chars() {
        if !is_id_char(c) {
            return Err(kw_error());
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

pub fn arr_to_app(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let arr = arr!(args.0[0]);
    return Ok(Value::app(arr));
}

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

pub fn arr_split(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[0]);
    let i = index_incl!(&arr, int!(args.0[1]));

    let (left, right) = arr.0.split_at(i);

    return Ok(Value::arr_from_vec(vec![
        Value::arr(Vector(left)),
        Value::arr(Vector(right)),
        ]));
}

pub fn arr_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let arr = arr!(args.0[0]);
    let start = index_incl!(&arr, int!(args.0[1]));
    let end = index_incl!(&arr, int!(args.0[2]));

    if start > end {
        return Err(index_error());
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

pub fn arr_cursor(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[0]);
    let index = index_incl!(&arr, int!(args.0[1]));

    return Ok(Value::cursor_arr(arr, index, cx));
}

pub fn cursor_arr_next(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_arr = cursor_arr!(args.0[0]);

    match (*cursor_arr.borrow_mut()).next() {
        Some(v) => return Ok(v),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

pub fn cursor_arr_prev(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_arr = cursor_arr!(args.0[0]);

    match (*cursor_arr.borrow_mut()).prev() {
        Some(v) => return Ok(v),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

/////////////////////////////////////////////////////////////////////////////

pub fn app_to_arr(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let app = app!(args.0[0]);
    return Ok(Value::arr(app));
}

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

pub fn app_split(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let app = app!(args.0[0]);
    let i = index_incl!(&app, int!(args.0[1]));

    let (left, right) = app.0.split_at(i);

    return Ok(Value::arr_from_vec(vec![
        Value::app(Vector(left)),
        Value::app(Vector(right)),
        ]));
}

pub fn app_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let app = app!(args.0[0]);
    let start = index_incl!(&app, int!(args.0[1]));
    let end = index_incl!(&app, int!(args.0[2]));

    if start > end {
        return Err(index_error());
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

pub fn app_cursor(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let app = app!(args.0[0]);
    let index = index_incl!(&app, int!(args.0[1]));

    return Ok(Value::cursor_app(app, index, cx));
}

pub fn cursor_app_next(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_app = cursor_app!(args.0[0]);

    match (*cursor_app.borrow_mut()).next() {
        Some(v) => return Ok(v),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

pub fn cursor_app_prev(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_app = cursor_app!(args.0[0]);

    match (*cursor_app.borrow_mut()).prev() {
        Some(v) => return Ok(v),
        None => return Err(Value::kw_str("cursor-end"))
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

pub fn set_find_lt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let needle = args.0[1].clone();

    match set_cursor::set_find_strict_lesser(&set, &needle) {
        Some(yay) => return Ok(yay),
        None => return Err(lookup_error())
    }
}

pub fn set_find_gt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let needle = args.0[1].clone();

    match set_cursor::set_find_strict_greater(&set, &needle) {
        Some(yay) => return Ok(yay),
        None => return Err(lookup_error())
    }
}

pub fn set_find_lte(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let needle = args.0[1].clone();

    match set_cursor::set_find_lesser(&set, &needle) {
        Some(yay) => return Ok(yay),
        None => return Err(lookup_error())
    }
}

pub fn set_find_gte(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let needle = args.0[1].clone();

    match set_cursor::set_find_greater(&set, &needle) {
        Some(yay) => return Ok(yay),
        None => return Err(lookup_error())
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

// FIXME: This is O(n), pavo requires O(log n)
pub fn set_split(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);
    let (left, member, mut right) = set.0.split_member(&args.0[1]);
    if member {
        right.insert(args.0[1].clone());
    }

    return Ok(Value::arr_from_vec(vec![
        Value::set(OrdSet(left)),
        Value::set(OrdSet(right)),
        ]));
}

// FIXME: This is O(n), pavo requires O(log n)
pub fn set_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let set = set!(args.0[0]);
    let (tmp, _) = set.0.split(&args.0[2]);
    let (_, member, mut ret) = tmp.split_member(&args.0[1]);
    if member {
        ret.insert(args.0[1].clone());
    }

    return Ok(Value::set(OrdSet(ret)));
}

pub fn set_cursor_min(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let set = set!(args.0[0]);

    return Ok(Value::cursor_set_min(set, cx));
}

pub fn set_cursor_max(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let set = set!(args.0[0]);

    return Ok(Value::cursor_set_max(set, cx));
}

pub fn set_cursor_less_strict(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);

    return Ok(Value::cursor_set_less_strict(set, &args.0[1], cx));
}

pub fn set_cursor_greater_strict(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);

    return Ok(Value::cursor_set_greater_strict(set, &args.0[1], cx));
}

pub fn set_cursor_less(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);

    return Ok(Value::cursor_set_less(set, &args.0[1], cx));
}

pub fn set_cursor_greater(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let set = set!(args.0[0]);

    return Ok(Value::cursor_set_greater(set, &args.0[1], cx));
}

pub fn cursor_set_next(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_set = cursor_set!(args.0[0]);

    match (*cursor_set.borrow_mut()).next() {
        Some(v) => return Ok(v.clone()),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

pub fn cursor_set_prev(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_set = cursor_set!(args.0[0]);

    match (*cursor_set.borrow_mut()).prev() {
        Some(v) => return Ok(v.clone()),
        None => return Err(Value::kw_str("cursor-end"))
    }
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
        None => Err(lookup_error()),
    }
}

pub fn map_contains(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let key = args.0[1].clone();

    Ok(Value::bool_(map.0.contains_key(&key)))
}

pub fn map_find_lt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let needle = args.0[1].clone();

    match map_cursor::map_find_strict_lesser(&map, &needle) {
        Some(yay) => return Ok(yay),
        None => return Err(lookup_error())
    }
}

pub fn map_find_gt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let needle = args.0[1].clone();

    match map_cursor::map_find_strict_greater(&map, &needle) {
        Some(yay) => return Ok(yay),
        None => return Err(lookup_error())
    }
}

pub fn map_find_lte(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let needle = args.0[1].clone();

    match map_cursor::map_find_lesser(&map, &needle) {
        Some(yay) => return Ok(yay),
        None => return Err(lookup_error())
    }
}

pub fn map_find_gte(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let needle = args.0[1].clone();

    match map_cursor::map_find_greater(&map, &needle) {
        Some(yay) => return Ok(yay),
        None => return Err(lookup_error())
    }
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

// FIXME: This is O(n), pavo requires O(log n)
pub fn map_split(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let (left, member, mut right) = map.0.split_lookup(&args.0[1]);
    if let Some(v) = member {
        right.insert(args.0[1].clone(), v);
    }

    return Ok(Value::arr_from_vec(vec![
        Value::map(OrdMap(left)),
        Value::map(OrdMap(right)),
        ]));
}

// FIXME: This is O(n), pavo requires O(log n)
pub fn map_slice(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let map = map!(args.0[0]);
    let (tmp, _) = map.0.split(&args.0[2]);
    let (_, member, mut ret) = tmp.split_lookup(&args.0[1]);
    if let Some(v) = member {
        ret.insert(args.0[1].clone(), v);
    }

    return Ok(Value::map(OrdMap(ret)));
}

pub fn map_cursor_min(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);

    return Ok(Value::cursor_map_min(map, cx));
}

pub fn map_cursor_max(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let map = map!(args.0[0]);

    return Ok(Value::cursor_map_max(map, cx));
}

pub fn map_cursor_less_strict(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);

    return Ok(Value::cursor_map_less_strict(map, &args.0[1], cx));
}

pub fn map_cursor_greater_strict(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);

    return Ok(Value::cursor_map_greater_strict(map, &args.0[1], cx));
}

pub fn map_cursor_less(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);

    return Ok(Value::cursor_map_less(map, &args.0[1], cx));
}

pub fn map_cursor_greater(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);

    return Ok(Value::cursor_map_greater(map, &args.0[1], cx));
}

pub fn cursor_map_next(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_map = cursor_map!(args.0[0]);

    match (*cursor_map.borrow_mut()).next() {
        Some((k, v)) => return Ok(Value::arr_from_vec(vec![k.clone(), v.clone()])),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

pub fn cursor_map_prev(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let cursor_map = cursor_map!(args.0[0]);

    match (*cursor_map.borrow_mut()).prev() {
        Some((k, v)) => return Ok(Value::arr_from_vec(vec![k.clone(), v.clone()])),
        None => return Err(Value::kw_str("cursor-end"))
    }
}

/////////////////////////////////////////////////////////////////////////////

pub fn fun_arity(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let f = fun!(args.0[0]);

    let arity = match f {
        Fun::Opaque{..} => 1,
        Fun::Closure(ref c, _) => c.args.clone(),
        Fun::Builtin(Builtin::BoolNot) => 1,
        Fun::Builtin(Builtin::BoolAnd) => 2,
        Fun::Builtin(Builtin::BoolOr) => 2,
        Fun::Builtin(Builtin::BoolIf) => 2,
        Fun::Builtin(Builtin::BoolIff) => 2,
        Fun::Builtin(Builtin::BoolXor) => 2,

        Fun::Builtin(Builtin::IntCountOnes) => 1,
        Fun::Builtin(Builtin::IntCountZeros) => 1,
        Fun::Builtin(Builtin::IntLeadingOnes) => 1,
        Fun::Builtin(Builtin::IntLeadingZeros) => 1,
        Fun::Builtin(Builtin::IntTrailingOnes) => 1,
        Fun::Builtin(Builtin::IntTrailingZeros) => 1,
        Fun::Builtin(Builtin::IntRotateLeft) => 1,
        Fun::Builtin(Builtin::IntRotateRight) => 1,
        Fun::Builtin(Builtin::IntReverseBytes) => 1,
        Fun::Builtin(Builtin::IntReverseBits) => 1,
        Fun::Builtin(Builtin::IntAdd) => 2,
        Fun::Builtin(Builtin::IntSub) => 2,
        Fun::Builtin(Builtin::IntMul) => 2,
        Fun::Builtin(Builtin::IntDiv) => 2,
        Fun::Builtin(Builtin::IntDivTrunc) => 2,
        Fun::Builtin(Builtin::IntMod) => 2,
        Fun::Builtin(Builtin::IntModTrunc) => 2,
        Fun::Builtin(Builtin::IntNeg) => 1,
        Fun::Builtin(Builtin::IntShl) => 2,
        Fun::Builtin(Builtin::IntShr) => 2,
        Fun::Builtin(Builtin::IntAbs) => 1,
        Fun::Builtin(Builtin::IntPow) => 2,
        Fun::Builtin(Builtin::IntAddSat) => 2,
        Fun::Builtin(Builtin::IntSubSat) => 2,
        Fun::Builtin(Builtin::IntMulSat) => 2,
        Fun::Builtin(Builtin::IntPowSat) => 2,
        Fun::Builtin(Builtin::IntAddWrap) => 2,
        Fun::Builtin(Builtin::IntSubWrap) => 2,
        Fun::Builtin(Builtin::IntMulWrap) => 2,
        Fun::Builtin(Builtin::IntDivWrap) => 2,
        Fun::Builtin(Builtin::IntDivTruncWrap) => 2,
        Fun::Builtin(Builtin::IntModWrap) => 2,
        Fun::Builtin(Builtin::IntModTruncWrap) => 2,
        Fun::Builtin(Builtin::IntNegWrap) => 1,
        Fun::Builtin(Builtin::IntAbsWrap) => 1,
        Fun::Builtin(Builtin::IntPowWrap) => 2,
        Fun::Builtin(Builtin::IntSignum) => 1,

        Fun::Builtin(Builtin::BytesCount) => 1,
        Fun::Builtin(Builtin::BytesGet) => 2,
        Fun::Builtin(Builtin::BytesInsert) => 3,
        Fun::Builtin(Builtin::BytesRemove) => 2,
        Fun::Builtin(Builtin::BytesUpdate) => 3,
        Fun::Builtin(Builtin::BytesSplit) => 2,
        Fun::Builtin(Builtin::BytesSlice) => 3,
        Fun::Builtin(Builtin::BytesSplice) => 3,
        Fun::Builtin(Builtin::BytesConcat) => 2,
        Fun::Builtin(Builtin::BytesCursor) => 2,

        Fun::Builtin(Builtin::IntToChar) => 1,
        Fun::Builtin(Builtin::IsIntToChar) => 1,
        Fun::Builtin(Builtin::CharToInt) => 1,

        Fun::Builtin(Builtin::StrToBytes) => 1,
        Fun::Builtin(Builtin::BytesToStr) => 1,
        Fun::Builtin(Builtin::IsBytesToStr) => 1,
        Fun::Builtin(Builtin::StrCount) => 1,
        Fun::Builtin(Builtin::StrCountUtf8) => 1,
        Fun::Builtin(Builtin::StrGet) => 2,
        Fun::Builtin(Builtin::StrGetUtf8) => 2,
        Fun::Builtin(Builtin::StrIndexCharToUtf8) => 2,
        Fun::Builtin(Builtin::StrIndexUtf8ToChar) => 2,
        Fun::Builtin(Builtin::StrInsert) => 3,
        Fun::Builtin(Builtin::StrRemove) => 2,
        Fun::Builtin(Builtin::StrUpdate) => 3,
        Fun::Builtin(Builtin::StrSplit) => 2,
        Fun::Builtin(Builtin::StrSlice) => 3,
        Fun::Builtin(Builtin::StrSplice) => 3,
        Fun::Builtin(Builtin::StrConcat) => 2,
        Fun::Builtin(Builtin::StrCursor) => 2,
        Fun::Builtin(Builtin::StrCursorUtf8) => 2,

        Fun::Builtin(Builtin::FloatAdd) => 2,
        Fun::Builtin(Builtin::FloatSub) => 2,
        Fun::Builtin(Builtin::FloatMul) => 2,
        Fun::Builtin(Builtin::FloatDiv) => 2,
        Fun::Builtin(Builtin::FloatMulAdd) => 3,
        Fun::Builtin(Builtin::FloatNeg) => 1,
        Fun::Builtin(Builtin::FloatFloor) => 1,
        Fun::Builtin(Builtin::FloatCeil) => 1,
        Fun::Builtin(Builtin::FloatRound) => 1,
        Fun::Builtin(Builtin::FloatTrunc) => 1,
        Fun::Builtin(Builtin::FloatFract) => 1,
        Fun::Builtin(Builtin::FloatAbs) => 1,
        Fun::Builtin(Builtin::FloatSignum) => 1,
        Fun::Builtin(Builtin::FloatPow) => 2,
        Fun::Builtin(Builtin::FloatSqrt) => 1,
        Fun::Builtin(Builtin::FloatExp) => 1,
        Fun::Builtin(Builtin::FloatExp2) => 1,
        Fun::Builtin(Builtin::FloatLn) => 1,
        Fun::Builtin(Builtin::FloatLog2) => 1,
        Fun::Builtin(Builtin::FloatLog10) => 1,
        Fun::Builtin(Builtin::FloatHypot) => 2,
        Fun::Builtin(Builtin::FloatSin) => 1,
        Fun::Builtin(Builtin::FloatCos) => 1,
        Fun::Builtin(Builtin::FloatTan) => 1,
        Fun::Builtin(Builtin::FloatAsin) => 1,
        Fun::Builtin(Builtin::FloatAcos) => 1,
        Fun::Builtin(Builtin::FloatAtan) => 1,
        Fun::Builtin(Builtin::FloatAtan2) => 2,
        Fun::Builtin(Builtin::FloatExpM1) => 1,
        Fun::Builtin(Builtin::FloatLn1P) => 1,
        Fun::Builtin(Builtin::FloatSinH) => 1,
        Fun::Builtin(Builtin::FloatCosH) => 1,
        Fun::Builtin(Builtin::FloatTanH) => 1,
        Fun::Builtin(Builtin::FloatAsinH) => 1,
        Fun::Builtin(Builtin::FloatAcosH) => 1,
        Fun::Builtin(Builtin::FloatAtanH) => 1,
        Fun::Builtin(Builtin::FloatIsNormal) => 1,
        Fun::Builtin(Builtin::FloatIsIntegral) => 1,
        Fun::Builtin(Builtin::FloatToDegrees) => 1,
        Fun::Builtin(Builtin::FloatToRadians) => 1,
        Fun::Builtin(Builtin::FloatToInt) => 1,
        Fun::Builtin(Builtin::IntToFloat) => 1,
        Fun::Builtin(Builtin::FloatToBits) => 1,
        Fun::Builtin(Builtin::BitsToFloat) => 1,
        Fun::Builtin(Builtin::IsBitsToFloat) => 1,

        Fun::Builtin(Builtin::StrToId) => 1,
        Fun::Builtin(Builtin::IsStrToId) => 1,
        Fun::Builtin(Builtin::IdToStr) => 1,

        Fun::Builtin(Builtin::StrToKw) => 1,
        Fun::Builtin(Builtin::IsStrToKw) => 1,
        Fun::Builtin(Builtin::KwToStr) => 1,

        Fun::Builtin(Builtin::ArrToApp) => 1,
        Fun::Builtin(Builtin::ArrCount) => 1,
        Fun::Builtin(Builtin::ArrGet) => 2,
        Fun::Builtin(Builtin::ArrInsert) => 3,
        Fun::Builtin(Builtin::ArrRemove) => 2,
        Fun::Builtin(Builtin::ArrUpdate) => 3,
        Fun::Builtin(Builtin::ArrSplit) => 2,
        Fun::Builtin(Builtin::ArrSlice) => 3,
        Fun::Builtin(Builtin::ArrSplice) => 3,
        Fun::Builtin(Builtin::ArrConcat) => 2,
        Fun::Builtin(Builtin::ArrCursor) => 2,

        Fun::Builtin(Builtin::AppToArr) => 1,
        Fun::Builtin(Builtin::AppCount) => 1,
        Fun::Builtin(Builtin::AppGet) => 2,
        Fun::Builtin(Builtin::AppInsert) => 3,
        Fun::Builtin(Builtin::AppRemove) => 2,
        Fun::Builtin(Builtin::AppUpdate) => 3,
        Fun::Builtin(Builtin::AppSplit) => 2,
        Fun::Builtin(Builtin::AppSlice) => 3,
        Fun::Builtin(Builtin::AppSplice) => 3,
        Fun::Builtin(Builtin::AppConcat) => 2,
        Fun::Builtin(Builtin::AppCursor) => 2,

        Fun::Builtin(Builtin::SetCount) => 1,
        Fun::Builtin(Builtin::SetContains) => 2,
        Fun::Builtin(Builtin::SetMin) => 1,
        Fun::Builtin(Builtin::SetMax) => 1,
        Fun::Builtin(Builtin::SetFindLT) => 2,
        Fun::Builtin(Builtin::SetFindGT) => 2,
        Fun::Builtin(Builtin::SetFindLTE) => 2,
        Fun::Builtin(Builtin::SetFindGTE) => 2,
        Fun::Builtin(Builtin::SetInsert) => 2,
        Fun::Builtin(Builtin::SetRemove) => 2,
        Fun::Builtin(Builtin::SetUnion) => 2,
        Fun::Builtin(Builtin::SetIntersection) => 2,
        Fun::Builtin(Builtin::SetDifference) => 2,
        Fun::Builtin(Builtin::SetSymmetricDifference) => 2,
        Fun::Builtin(Builtin::SetSplit) => 2,
        Fun::Builtin(Builtin::SetSlice) => 2,
        Fun::Builtin(Builtin::SetCursorMin) => 1,
        Fun::Builtin(Builtin::SetCursorMax) => 1,
        Fun::Builtin(Builtin::SetCursorLessStrict) => 2,
        Fun::Builtin(Builtin::SetCursorGreaterStrict) => 2,
        Fun::Builtin(Builtin::SetCursorLess) => 2,
        Fun::Builtin(Builtin::SetCursorGreater) => 2,

        Fun::Builtin(Builtin::MapCount) => 1,
        Fun::Builtin(Builtin::MapGet) => 2,
        Fun::Builtin(Builtin::MapFindLT) => 2,
        Fun::Builtin(Builtin::MapFindGT) => 2,
        Fun::Builtin(Builtin::MapFindLTE) => 2,
        Fun::Builtin(Builtin::MapFindGTE) => 2,
        Fun::Builtin(Builtin::MapContains) => 2,
        Fun::Builtin(Builtin::MapMin) => 1,
        Fun::Builtin(Builtin::MapMinKey) => 1,
        Fun::Builtin(Builtin::MapMinEntry) => 1,
        Fun::Builtin(Builtin::MapMax) => 1,
        Fun::Builtin(Builtin::MapMaxKey) => 1,
        Fun::Builtin(Builtin::MapMaxEntry) => 1,
        Fun::Builtin(Builtin::MapInsert) => 3,
        Fun::Builtin(Builtin::MapRemove) => 2,
        Fun::Builtin(Builtin::MapUnion) => 2,
        Fun::Builtin(Builtin::MapIntersection) => 2,
        Fun::Builtin(Builtin::MapDifference) => 2,
        Fun::Builtin(Builtin::MapSymmetricDifference) => 2,
        Fun::Builtin(Builtin::MapSplit) => 2,
        Fun::Builtin(Builtin::MapSlice) => 3,
        Fun::Builtin(Builtin::MapCursorMin) => 1,
        Fun::Builtin(Builtin::MapCursorMax) => 1,
        Fun::Builtin(Builtin::MapCursorLessStrict) => 2,
        Fun::Builtin(Builtin::MapCursorGreaterStrict) => 2,
        Fun::Builtin(Builtin::MapCursorLess) => 2,
        Fun::Builtin(Builtin::MapCursorGreater) => 2,

        Fun::Builtin(Builtin::FunArity) => 1,
        Fun::Builtin(Builtin::FunApply) => 2,

        Fun::Builtin(Builtin::Symbol) => 0,

        Fun::Builtin(Builtin::Cell) => 1,
        Fun::Builtin(Builtin::CellGet) => 1,
        Fun::Builtin(Builtin::CellSet) => 2,

        Fun::Builtin(Builtin::Opaque) => 0,

        Fun::Builtin(Builtin::Cmp) => 2,
        Fun::Builtin(Builtin::Eq) => 2,
        Fun::Builtin(Builtin::Lt) => 2,
        Fun::Builtin(Builtin::Lte) => 2,
        Fun::Builtin(Builtin::Gt) => 2,
        Fun::Builtin(Builtin::Gte) => 2,

        Fun::Builtin(Builtin::Read) => 1,
        Fun::Builtin(Builtin::Write) => 1,
        Fun::Builtin(Builtin::Check) => 2,
        Fun::Builtin(Builtin::Eval) => 2,
        Fun::Builtin(Builtin::Expand) => 2,
        Fun::Builtin(Builtin::Exval) => 2,

        Fun::Builtin(Builtin::Typeof) => 1,
        Fun::Builtin(Builtin::IsTruthy) => 1,
        Fun::Builtin(Builtin::Not) => 1,
        Fun::Builtin(Builtin::Diverge) => 1,
        Fun::Builtin(Builtin::Trace) => 1,

        Fun::Builtin(Builtin::CursorArrNext) => 1,
        Fun::Builtin(Builtin::CursorArrPrev) => 1,
        Fun::Builtin(Builtin::CursorAppNext) => 1,
        Fun::Builtin(Builtin::CursorAppPrev) => 1,
        Fun::Builtin(Builtin::CursorBytesNext) => 1,
        Fun::Builtin(Builtin::CursorBytesPrev) => 1,
        Fun::Builtin(Builtin::CursorStrNext) => 1,
        Fun::Builtin(Builtin::CursorStrPrev) => 1,
        Fun::Builtin(Builtin::CursorStrUtf8Next) => 1,
        Fun::Builtin(Builtin::CursorStrUtf8Prev) => 1,
        Fun::Builtin(Builtin::CursorSetNext) => 1,
        Fun::Builtin(Builtin::CursorSetPrev) => 1,
        Fun::Builtin(Builtin::CursorMapNext) => 1,
        Fun::Builtin(Builtin::CursorMapPrev) => 1,

        Fun::Builtin(Builtin::MacroIf) => 3,
        Fun::Builtin(Builtin::MacroSetBang) => 2,
        Fun::Builtin(Builtin::MacroThrow) => 1,
        Fun::Builtin(Builtin::MacroDo) => 1,
        Fun::Builtin(Builtin::MacroCond) => 1,
        Fun::Builtin(Builtin::MacroLet) => 3,
        Fun::Builtin(Builtin::MacroLetFn) => 2,
        Fun::Builtin(Builtin::MacroFn) => 3,
        Fun::Builtin(Builtin::MacroLambda) => 2,
        Fun::Builtin(Builtin::MacroThreadFirst) => 2,
        Fun::Builtin(Builtin::MacroThreadLast) => 2,
        Fun::Builtin(Builtin::MacroThreadAs) => 3,
        Fun::Builtin(Builtin::MacroOr) => 1,
        Fun::Builtin(Builtin::MacroOr2) => 2,
        Fun::Builtin(Builtin::MacroAnd) => 1,
        Fun::Builtin(Builtin::MacroAnd2) => 2,
        Fun::Builtin(Builtin::MacroQuasiquote) => 1,
        Fun::Builtin(Builtin::MacroWhile) => 2,
        Fun::Builtin(Builtin::MacroMatch) => 4,
        Fun::Builtin(Builtin::MacroCase) => 2,
        Fun::Builtin(Builtin::MacroLoop) => 2,
        Fun::Builtin(Builtin::MacroTry) => 3,

        Fun::Builtin(Builtin::Require) => 2,
    };

    return Ok(Value::int(arity as i64));
}

pub fn fun_apply(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let f = fun!(args.0[0]);
    let f_args = arr!(args.0[1]);

    return f.compute(f_args, cx);
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
        None => Err(type_error()),
        Some(inner) => Ok(inner.borrow().clone()),
    }
}

pub fn cell_set(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    match args.0[0].as_cell() {
        None => Err(type_error()),
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

pub fn pavo_lt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    Ok(Value::bool_(args.0[0] < args.0[1]))
}

pub fn pavo_lte(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    Ok(Value::bool_(args.0[0] <= args.0[1]))
}

pub fn pavo_gt(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    Ok(Value::bool_(args.0[0] > args.0[1]))
}

pub fn pavo_gte(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    Ok(Value::bool_(args.0[0] >= args.0[1]))
}

/////////////////////////////////////////////////////////////////////////////

pub fn read(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let s = string!(args.0[0]);

    match read_(CompleteStr(&s.0.to_string())) {
        Ok(v) => return Ok(v),
        Err(_) => return Err(not_expression_error()),
    }
}

pub fn write(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let mut buf = String::new();

    write_(&args.0[0], &mut buf)?;
    if buf.len() >= i64::max as usize {
        return Err(coll_full_error());
    }

    Ok(Value::string(Rope(Ropey::from(&buf[..]))))
}

pub fn check(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let v = &args.0[0];
    let map = map!(args.0[1]);

    let mut check_env = ImOrdMap::new();
    for key in env::default().keys() {
        check_env.insert(key.clone(), false);
    }

    let remove = match map.0.get(&Value::kw_str("remove")) {
        Some(tmp) => set!(tmp).0,
        None => ImOrdSet::new(),
    };
    for val in remove.iter() {
        check_env.remove(&id!(val));
    }

    let mutable = match map.0.get(&Value::kw_str("mutable")) {
        Some(tmp) => set!(tmp).0,
        None => ImOrdSet::new(),
    };
    for val in mutable.iter() {
        check_env.insert(id!(val), true);
    }

    let immutable = match map.0.get(&Value::kw_str("immutable")) {
        Some(tmp) => set!(tmp).0,
        None => ImOrdSet::new(),
    };
    for val in immutable.iter() {
        check_env.insert(id!(val), false);
    }

    match to_code(v) {
        Err(_) => return  Ok(Value::bool_(false)),
        Ok(code) => match check_(code, &check_env) {
            Ok(()) => return Ok(Value::bool_(true)),
            Err(_) => return Ok(Value::bool_(false)),
        }
    }
}

pub fn eval(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let v = &args.0[0];
    let map = map!(args.0[1]);

    let mut env = env::default();

    let remove = match map.0.get(&Value::kw_str("remove")) {
        Some(tmp) => set!(tmp).0,
        None => ImOrdSet::new(),
    };
    for val in remove.iter() {
        env.remove(&id!(val));
    }

    let mutable = match map.0.get(&Value::kw_str("mutable")) {
        Some(tmp) => map!(tmp).0,
        None => ImOrdMap::new(),
    };
    for (key, val) in mutable.iter() {
        env.insert(id!(key), (val.clone(), true));
    }

    let immutable = match map.0.get(&Value::kw_str("immutable")) {
        Some(tmp) => map!(tmp).0,
        None => ImOrdMap::new(),
    };
    for (key, val) in immutable.iter() {
        env.insert(id!(key), (val.clone(), false));
    }

    match compile_(v, &env) {
        Err(_) => return Err(static_error()),
        Ok(c) => match c.compute(Vector(ImVector::new()), cx) {
            Ok(yay) => return Ok(yay),
            Err(err) => return Err(eval_error(err)),
        }
    }
}

pub fn expand(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let v = &args.0[0];
    let map = map!(args.0[1]);

    let mut def_env = env::default();
    let mut macro_env = macros::default();

    let def_remove = match map.0.get(&Value::kw_str("def-remove")) {
        Some(tmp) => set!(tmp).0,
        None => ImOrdSet::new(),
    };
    for val in def_remove.iter() {
        def_env.remove(&id!(val));
    }

    let def_mutable = match map.0.get(&Value::kw_str("def-mutable")) {
        Some(tmp) => map!(tmp).0,
        None => ImOrdMap::new(),
    };
    for (key, val) in def_mutable.iter() {
        def_env.insert(id!(key), (val.clone(), true));
    }

    let def_immutable = match map.0.get(&Value::kw_str("def-immutable")) {
        Some(tmp) => map!(tmp).0,
        None => ImOrdMap::new(),
    };
    for (key, val) in def_immutable.iter() {
        def_env.insert(id!(key), (val.clone(), false));
    }

    let macro_remove = match map.0.get(&Value::kw_str("macro-remove")) {
        Some(tmp) => set!(tmp).0,
        None => ImOrdSet::new(),
    };
    for val in macro_remove.iter() {
        macro_env.remove(&id!(val));
    }

    let macro_mutable = match map.0.get(&Value::kw_str("macro-add")) {
        Some(tmp) => map!(tmp).0,
        None => ImOrdMap::new(),
    };
    for (key, val) in macro_mutable.iter() {
        macro_env.insert(id!(key), val.clone());
    }

    match expand_(v, &def_env, &macro_env, cx) {
        Err(_) => return Err(expand_error()),
        Ok(yay) => return Ok(yay),
    }
}

pub fn exval(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let options = args.0[1].clone();

    match expand(args, cx) {
        Err(err) => return Err(err),
        Ok(expanded) => {
            let mut eval_args = ImVector::new();
            eval_args.push_back(expanded);
            eval_args.push_back(options);
            return eval(Vector(eval_args), cx);
        }
    }
}

pub fn require(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let opts = map!(args.0[1]);

    let mut expand_opts = ImOrdMap::new();
    if let Some(v) = opts.0.get(&Value::kw_str("def-remove")) {
        expand_opts.insert(Value::kw_str("def-remove"), v.clone());
    }
    if let Some(v) = opts.0.get(&Value::kw_str("def-mutable")) {
        expand_opts.insert(Value::kw_str("def-mutable"), v.clone());
    }
    if let Some(v) = opts.0.get(&Value::kw_str("def-immutable")) {
        expand_opts.insert(Value::kw_str("def-immutable"), v.clone());
    }
    if let Some(v) = opts.0.get(&Value::kw_str("macro-remove")) {
        expand_opts.insert(Value::kw_str("macro-remove"), v.clone());
    }
    if let Some(v) = opts.0.get(&Value::kw_str("macro-add")) {
        expand_opts.insert(Value::kw_str("macro-add"), v.clone());
    }

    let mut eval_opts = ImOrdMap::new();
    if let Some(v) = opts.0.get(&Value::kw_str("remove")) {
        eval_opts.insert(Value::kw_str("remove"), v.clone());
    }
    if let Some(v) = opts.0.get(&Value::kw_str("mutable")) {
        eval_opts.insert(Value::kw_str("muable"), v.clone());
    }
    if let Some(v) = opts.0.get(&Value::kw_str("immutable")) {
        eval_opts.insert(Value::kw_str("immutable"), v.clone());
    }

    return cx.require(&args.0[0], &expand_opts, &eval_opts);
}

/////////////////////////////////////////////////////////////////////////////

pub fn typeof_(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(typeof__(&args.0[0]))
}

pub fn is_truthy(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(match args.0[0] {
        Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(false)) => Value::bool_(false),
        _ => Value::bool_(true),
    })
}

pub fn not(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    Ok(match args.0[0] {
        Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(false)) => Value::bool_(true),
        _ => Value::bool_(false),
    })
}

pub fn diverge(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let mut buf = String::new();
    value::debug_print(&args.0[0], 0, 2, &mut buf);
    panic!("Called diverge: {}", buf);
}

pub fn trace(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let mut buf = String::new();
    value::debug_print(&args.0[0].clone(), 0, 2, &mut buf);
    println!("{}", buf);
    Ok(args.0[0].clone())
}

/////////////////////////////////////////////////////////////////////////////

fn macro_do_(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    let mut tmp = Vector(ImVector::new());

    for (i, form) in args.0.iter().enumerate() {
        match form.as_app() {
            Some(app) => {
                if app.0.len() == 0 {
                    tmp.0.push_back(form.clone());
                } else {
                    if let Some(kw) = app.0[0].as_kw() {
                        if kw == "let" {
                            if app.0.len() != 3 {
                                return Err(num_args_error());
                            } else {
                                let let_body = macro_do_(Vector(args.0.skip(i + 1)), _cx)?;
                                tmp.0.push_back(Value::app_from_vec(vec![
                                        Value::id_str("let"),
                                        app.0[1].clone(),
                                        app.0[2].clone(),
                                        let_body,
                                    ]));
                                    return Ok(Value::app_from_vec(vec![
                                            Value::id_str("sf-do"),
                                            Value::arr(tmp)
                                        ]));
                            }
                        } else {
                            tmp.0.push_back(form.clone());
                        }
                    } else {
                        tmp.0.push_back(form.clone());
                    }
                }
            }
            None => tmp.0.push_back(form.clone()),
        }
    }

    return Ok(Value::app_from_vec(vec![
            Value::id_str("sf-do"),
            Value::arr(tmp)
        ]));
}

pub fn macro_do(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let arr = arr!(args.0[0]);

    return Ok(macro_do_(arr, _cx)?);
}

pub fn macro_set_bang(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;

    let mut tmp = args.clone();
    tmp.0.push_front(Value::id_str("sf-set!"));
    return Ok(Value::app(tmp));
}

pub fn macro_throw(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;

    let mut tmp = args.clone();
    tmp.0.push_front(Value::id_str("sf-throw"));
    return Ok(Value::app(tmp));
}

pub fn macro_if(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;

    return Ok(Value::app_from_vec(vec![
        Value::id_str("sf-case"),
        Value::app_from_vec(vec![
            Value::builtin(value::Builtin::IsTruthy),
            args.0[0].clone(),
            ]),
        Value::arr_from_vec(vec![
            Value::bool_(true), args.0[1].clone(),
            Value::bool_(false), args.0[2].clone(),
            ]),
        ]));
}

fn macro_cond_(arr: &Vector<Value>) -> Result<Value, Value> {
    match arr.0.len() {
        0 => return Ok(Value::nil()),
        1 => return Ok(arr.0[0].clone()),
        2 => return Ok(Value::app_from_vec(vec![
                Value::id_str("if"),
                arr.0[0].clone(),
                arr.0[1].clone(),
                Value::nil(),
            ])),
        3 => return Ok(Value::app_from_vec(vec![
                Value::id_str("if"),
                arr.0[0].clone(),
                arr.0[1].clone(),
                arr.0[2].clone(),
            ])),
        _ => return Ok(Value::app_from_vec(vec![
                Value::id_str("if"),
                arr.0[0].clone(),
                arr.0[1].clone(),
                macro_cond_(&Vector(arr.0.skip(2)))?,
            ])),
    }
}

pub fn macro_cond(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let arr = arr!(args.0[0]);

    return macro_cond_(&arr);
}

pub fn macro_let(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;

    Ok(Value::app_from_vec(vec![
            Value::app_from_vec(vec![
                    Value::id_str("lambda"),
                    Value::arr_from_vec(vec![args.0[0].clone()]),
                    args.0[2].clone(),
                ]),
            args.0[1].clone(),
        ]))
}

fn macro_thread_first_(v: Value, arr: &Vector<Value>) -> Result<Value, Value> {
    match arr.0.len() {
        0 => return Ok(v),
        _ => {
            let mut app = app!(arr.0[0]);
            if app.0.len() == 0 {
                return Err(index_error());
            }
            app.0.insert(1, v);

            if arr.0.len() == 1 {
                return Ok(Value::app(app));
            } else {
                return macro_thread_first_(Value::app(app), &Vector(arr.0.skip(1)));
            }
        }
    }
}

pub fn macro_thread_first(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[1]);

    return macro_thread_first_(args.0[0].clone(), &arr);
}

fn macro_thread_last_(v: Value, arr: &Vector<Value>) -> Result<Value, Value> {
    match arr.0.len() {
        0 => return Ok(v),
        _ => {
            let mut app = app!(arr.0[0]);
            app.0.insert(app.0.len(), v);

            if arr.0.len() == 1 {
                return Ok(Value::app(app));
            } else {
                return macro_thread_last_(Value::app(app), &Vector(arr.0.skip(1)));
            }
        }
    }
}

pub fn macro_thread_last(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[1]);

    return macro_thread_last_(args.0[0].clone(), &arr);
}

fn macro_thread_as_(pattern: Value, v: Value, arr: &Vector<Value>) -> Result<Value, Value> {
    match arr.0.len() {
        0 => return Ok(v),
        1 => return Ok(Value::app_from_vec(vec![
                Value::id_str("let"),
                pattern,
                v,
                arr.0[0].clone(),
            ])),
        _ => return macro_thread_as_(
                pattern.clone(),
                Value::app_from_vec(vec![
                        Value::id_str("let"),
                        pattern,
                        v,
                        arr.0[0].clone(),
                    ]),
                &Vector(arr.0.skip(1))
            ),
    }
}

pub fn macro_thread_as(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let arr = arr!(args.0[2]);

    return macro_thread_as_(args.0[0].clone(), args.0[1].clone(), &arr);
}

fn macro_or_(arr: &Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    match arr.0.len() {
        0 => return Ok(Value::bool_(false)),
        1 => return Ok(arr.0[0].clone()),
        _ => {
            let id = Value::id(Id::Symbol(cx.next_symbol_id()));
            return Ok(Value::app_from_vec(vec![
                    Value::id_str("let"),
                    id.clone(),
                    arr.0[0].clone(),
                    Value::app_from_vec(vec![
                            Value::id_str("if"),
                            id.clone(),
                            id.clone(),
                            macro_or_(&Vector(arr.0.skip(1)), cx)?,
                        ]),
                ]));
        }
    }
}

pub fn macro_or(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let arr = arr!(args.0[0]);

    return macro_or_(&arr, cx);
}

pub fn macro_or2(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let id = Value::id(Id::Symbol(cx.next_symbol_id()));

    return Ok(Value::app_from_vec(vec![
            Value::id_str("let"),
            id.clone(),
            args.0[0].clone(),
            Value::app_from_vec(vec![
                    Value::id_str("if"),
                    id.clone(),
                    id.clone(),
                    args.0[1].clone(),
                ]),
        ]));
}

fn macro_and_(arr: &Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    match arr.0.len() {
        0 => return Ok(Value::bool_(true)),
        1 => return Ok(arr.0[0].clone()),
        _ => {
            let id = Value::id(Id::Symbol(cx.next_symbol_id()));
            return Ok(Value::app_from_vec(vec![
                    Value::id_str("let"),
                    id.clone(),
                    arr.0[0].clone(),
                    Value::app_from_vec(vec![
                            Value::id_str("if"),
                            id.clone(),
                            macro_and_(&Vector(arr.0.skip(1)), cx)?,
                            id.clone(),
                        ]),
                ]));
        }
    }
}

pub fn macro_and(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    let arr = arr!(args.0[0]);

    return macro_and_(&arr, cx);
}

pub fn macro_and2(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let id = Value::id(Id::Symbol(cx.next_symbol_id()));

    return Ok(Value::app_from_vec(vec![
            Value::id_str("let"),
            id.clone(),
            args.0[0].clone(),
            Value::app_from_vec(vec![
                    Value::id_str("if"),
                    id.clone(),
                    args.0[1].clone(),
                    id.clone(),
                ]),
        ]));
}

fn quasiquote(v: &Value, fresh_names: &mut HashMap<Id, u64>, cx: &mut Context) -> Result<Value, Value> {
    match v {
        Value::Atomic(_) | Value::Fun(..) | Value::Cell(..) | Value::Opaque(..) => Ok(v.clone()),
        Value::Id(_) => Ok(Value::app_from_vec(vec![Value::id_str("sf-quote"), v.clone()])),
        Value::Arr(arr) => {
            let mut new_arr = ImVector::new();
            for w in arr.0.iter() {
                new_arr.push_back(quasiquote(w, fresh_names, cx)?);
            }
            Ok(Value::arr(Vector(new_arr)))
        }
        Value::Set(set) => {
            let mut new_set = ImOrdSet::new();
            for w in set.0.iter() {
                new_set.insert(quasiquote(w, fresh_names, cx)?);
            }
            Ok(Value::set(OrdSet(new_set)))
        }
        Value::Map(map) => {
            let mut new_map = ImOrdMap::new();
            for (key, value) in map.0.iter() {
                new_map.insert(
                    quasiquote(key, fresh_names, cx)?,
                    quasiquote(value, fresh_names, cx)?,
                );
            }
            Ok(Value::map(OrdMap(new_map)))
        }
        Value::App(app) => {
            if app.0.len() == 0 {
                return Ok(Value::app_from_vec(vec![
                        Value::builtin(value::Builtin::ArrToApp),
                        Value::arr(app.clone()),
                    ]));
            }

            if let Some(kw) = app.0[0].as_kw() {
                match kw {
                    "unquote" => {
                        if app.0.len() != 2 {
                            return Err(num_args_error());
                        } else {
                            return Ok(app.0[1].clone());
                        }
                    }
                    "fresh-name" => {
                        if app.0.len() != 2 {
                            return Err(num_args_error());
                        } else {
                            match &app.0[1].as_id() {
                                None => return Err(type_error()),
                                Some(id) => {
                                    if let Some(symbol_id) = fresh_names.get(id) {
                                        return Ok(Value::app_from_vec(vec![
                                            Value::id_str("sf-quote"),
                                            Value::Id(Id::Symbol(*symbol_id)),
                                            ]));
                                    } else {
                                        let symbol_id = cx.next_symbol_id();
                                        fresh_names.insert((*id).clone(), symbol_id);
                                        return Ok(Value::app_from_vec(vec![
                                            Value::id_str("sf-quote"),
                                            Value::Id(Id::Symbol(symbol_id)),
                                            ]));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // we are in an application, our first entry is neither `:unquote` nor `:fresh-name`
            let mut ret = ImVector::new();

            let mut current = ImVector::new();

            for w in app.0.iter() {
                if let Some(inner_app) = w.as_app() {
                    if inner_app.0.len() > 0 {
                        if let Some("unquote-splice") = inner_app.0[0].as_kw() {
                            if inner_app.0.len() != 2 {
                                return Err(num_args_error());
                            }
                            // got an unquote-splice form

                            if current.len() != 0 {
                                ret.push_back(Value::app_from_vec(vec![
                                        Value::builtin(value::Builtin::AppConcat),
                                        Value::app_from_vec(vec![
                                                Value::builtin(value::Builtin::ArrToApp),
                                                Value::arr(Vector(current)),
                                            ])
                                    ]));
                                current = ImVector::new();
                            }

                            ret.push_back(Value::app_from_vec(vec![
                                    Value::builtin(value::Builtin::AppConcat),
                                    inner_app.0[1].clone(),
                                ]));

                            continue;
                        }
                    }
                }
                // not an unquote-splice form

                current.push_back(quasiquote(w, fresh_names, cx)?);
            }

            ret.push_back(Value::app_from_vec(vec![
                    Value::builtin(value::Builtin::AppConcat),
                    Value::app_from_vec(vec![
                            Value::builtin(value::Builtin::ArrToApp),
                            Value::arr(Vector(current)),
                        ])
                ]));

            return macro_thread_first_(
                Value::app_from_vec(vec![
                    Value::id_str("sf-quote"),
                    Value::app_from_vec(vec![]),
                ]),
                &Vector(ret),
            );
        }
    }
}

pub fn macro_quasiquote(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 1)?;
    return quasiquote(&args.0[0], &mut HashMap::new(), cx);
}

pub fn macro_letfn(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let map = map!(args.0[0]);
    let mut new_map = ImOrdMap::new();

    for (key, val) in map.0.iter() {
        match val.as_app() {
            Some(fundef) if fundef.0.len() == 2 => {
                let pats_arr = arr!(fundef.0[0]);

                let mut fn_args = ImVector::new();
                for _ in 0..pats_arr.0.len() {
                    fn_args.push_back(Value::Id(Id::Symbol(cx.next_symbol_id())));
                }

                new_map.insert(key.clone(), Value::app_from_vec(vec![
                    Value::arr(Vector(fn_args.clone())),
                    Value::app_from_vec(vec![
                        Value::id_str("case"),
                        Value::arr(Vector(fn_args.clone())),
                        Value::arr_from_vec(vec![
                            fundef.0[0].clone(),
                            fundef.0[1].clone(),
                            ])
                        ]),
                    ]));
            }
            _ => return Err(type_error()),
        }
    }

    return Ok(Value::app_from_vec(vec![
        Value::id_str("sf-letfn"),
        Value::map(OrdMap(new_map)),
        args.0[1].clone(),
        ]));
}

pub fn macro_fn(args: Vector<Value>, _cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;

    return Ok(Value::app_from_vec(vec![
        Value::id_str("letfn"),
        Value::map_from_vec(vec![
            (args.0[0].clone(), Value::app_from_vec(vec![
                args.0[1].clone(),
                args.0[2].clone(),
                ]))
            ]),
        args.0[0].clone(),
        ]));
}

pub fn macro_try(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 3)?;
    let sym = Value::Id(Id::Symbol(cx.next_symbol_id()));

    return Ok(Value::app_from_vec(vec![
        Value::id_str("sf-try"),
        args.0[0].clone(),
        sym.clone(),
        Value::app_from_vec(vec![
            Value::id_str("case"),
            sym,
            Value::arr_from_vec(vec![
                args.0[1].clone(),
                args.0[2].clone(),
                ]),
            ]),
        ]));
}

pub fn macro_while(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let rec = Value::Id(Id::Symbol(cx.next_symbol_id()));
    let prev = Value::Id(Id::Symbol(cx.next_symbol_id()));

    return Ok(Value::app_from_vec(vec![
            Value::app_from_vec(vec![
                Value::id_str("fn"),
                rec.clone(),
                Value::arr(Vector(ImVector::unit(prev.clone()))),
                Value::app_from_vec(vec![
                    Value::id_str("if"),
                    args.0[0].clone(),
                    Value::app_from_vec(vec![
                        rec,
                        args.0[1].clone(),
                        ]),
                    prev,
                ]),
            ]),
            Value::nil(),
        ]));
}

pub fn macro_match(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 4)?;
    let sym = Value::Id(Id::Symbol(cx.next_symbol_id()));

    return Ok(Value::app_from_vec(vec![
        Value::id_str("case"),
        args.0[0].clone(),
        Value::arr_from_vec(vec![
            args.0[1].clone(), args.0[2].clone(),
            sym, args.0[3].clone(),
            ]),
        ]));
}

pub fn macro_case(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[1]);

    if arr.0.len() % 2 == 0 {
        return Ok(Value::app_from_vec(vec![
            Value::id_str("sf-case"),
            args.0[0].clone(),
            args.0[1].clone(),
            ]));
    } else {
        let sym = Value::Id(Id::Symbol(cx.next_symbol_id()));
        let mut new_arr = arr.clone();
        new_arr.0.insert(arr.0.len() - 1, sym);
        return Ok(Value::app_from_vec(vec![
            Value::id_str("sf-case"),
            args.0[0].clone(),
            Value::arr(new_arr),
            ]));
    };
}

// `((fn <sym> [<sym'>] (case v [p1 (<sym> then1), p2 (<sym> then2), ..., <sym''> <sym'>])) nil)`
pub fn macro_loop(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;
    let arr = arr!(args.0[1]);
    if arr.0.len() % 2 != 0 {
        return Err(type_error());
    }

    let rec = Value::Id(Id::Symbol(cx.next_symbol_id()));
    let prev = Value::Id(Id::Symbol(cx.next_symbol_id()));
    let fallback = Value::Id(Id::Symbol(cx.next_symbol_id()));

    let mut new_arr = ImVector::new();
    for (i, v) in arr.0.iter().enumerate() {
        if i % 2 == 0 {
            new_arr.push_back(v.clone());
        } else {
            new_arr.push_back(Value::app_from_vec(vec![
                rec.clone(),
                v.clone(),
                ]));
        }
    }
    new_arr.push_back(fallback);
    new_arr.push_back(prev.clone());

    return Ok(Value::app_from_vec(vec![
            Value::app_from_vec(vec![
                Value::id_str("fn"),
                rec.clone(),
                Value::arr(Vector(ImVector::unit(prev))),
                Value::app_from_vec(vec![
                    Value::id_str("case"),
                    args.0[0].clone(),
                    Value::arr(Vector(new_arr)),
                ]),
            ]),
            Value::nil(),
        ]));
}

pub fn macro_lambda(args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
    num_args(&args, 2)?;

    let pats_arr = arr!(args.0[0]);

    let mut fn_args = ImVector::new();
    for _ in 0..pats_arr.0.len() {
        fn_args.push_back(Value::Id(Id::Symbol(cx.next_symbol_id())));
    }


    return Ok(Value::app_from_vec(vec![
        Value::id_str("sf-lambda"),
        Value::arr(Vector(fn_args.clone())),
        Value::app_from_vec(vec![
            Value::id_str("case"),
            Value::arr(Vector(fn_args.clone())),
            Value::arr_from_vec(vec![
                args.0[0].clone(),
                args.0[1].clone(),
                ])
            ]),
        ]));
}

//////////////////////////////////////////////////////////////////////////////

pub fn write_spaces(indent: usize, out: &mut String) {
    for _ in 0..indent {
        out.push(' ');
    }
}

pub fn write_atomic(a: &Atomic, mut indent: usize, indent_inc: usize, out: &mut String) {
    match a {
        Atomic::Nil => out.push_str("nil"),
        Atomic::Bool(true) => out.push_str("true"),
        Atomic::Bool(false) => out.push_str("false"),
        Atomic::Int(n) => out.push_str(&n.to_string()),
        Atomic::Float(n) => {
            let mut b = Buffer::new();
            let ecmaliteral = b.format(n.0.into_inner());

            match ecmaliteral.find('.') {
                Some(_) => out.push_str(ecmaliteral),
                None => {
                    match ecmaliteral.find('e') {
                        Some(index) => {
                            let (fst, snd) = ecmaliteral.split_at(index);
                            out.push_str(fst);
                            out.push_str(".0");
                            out.push_str(snd)
                        }
                        None => {
                            out.push_str(ecmaliteral);
                            out.push_str(".0")
                        }
                    }
                }
            }
        }
        Atomic::Char('\\') => out.push_str("'\\\\'"),
        Atomic::Char('\'') => out.push_str("'\\''"),
        Atomic::Char('\t') => out.push_str("'\\t'"),
        Atomic::Char('\n') => out.push_str("'\\n'"),
        Atomic::Char(other) => {
            out.push('\'');
            out.push(*other);
            out.push('\'');
        }
        Atomic::String(chars) => {
            out.push('"');
            for c in chars.0.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '\"' => out.push_str("\\\""),
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    _ => out.push(c),
                }
            }
            out.push('"');
        }
        Atomic::Bytes(bytes) => {
            if bytes.0.len() == 0 {
                out.push_str("@[]");
                return;
            } else if bytes.0.len() == 1 || indent_inc == 0 {
                out.push_str("@[");
                for (i, b) in bytes.0.iter().enumerate() {
                    out.push_str(&b.to_string());
                    if i + 1 < bytes.0.len() {
                        out.push_str(" ")
                    }
                }
                out.push_str("]");
                return;
            } else {
                out.push_str("@[\n");
                indent += indent_inc;

                for b in bytes.0.iter() {
                    write_spaces(indent, out);
                    out.push_str(&b.to_string());
                    out.push_str(",\n");
                }

                indent -= indent_inc;
                write_spaces(indent, out);
                out.push_str("]");
                return;
            }
        }
        Atomic::Keyword(kw) => {
            out.push(':');
            out.push_str(kw);
        }
    }
}

pub fn write_(v: &Value, out: &mut String) -> Result<(), Value> {
    match v {
        Value::Atomic(a) => Ok(write_atomic(a, 0, 0, out)),
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
