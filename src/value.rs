//! Definition of the objects that the language manipulates at runtime.

use std::{
    cmp::Ordering,
    num::FpCategory,
};

use gc::{Gc, GcCell};
use gc_derive::{Trace, Finalize};
use im_rc::{
    Vector as ImVector,
    OrdSet as ImOrdSet,
    OrdMap as ImOrdMap,
};
use ropey::Rope as Ropey;

use crate::builtins::{self, type_error, type_error_, num_args_error};
use crate::context::Context;
use crate::gc_foreign::{Vector, OrdSet, OrdMap, NotNan, Rope};
use crate::vm::Closure;

#[derive(Debug, Clone, PartialEq, Eq, Trace, Finalize)]
pub enum Value {
    Atomic(Atomic),
    Id(Id),
    Arr(Vector<Value>),
    App(Vector<Value>),
    Set(OrdSet<Value>),
    Map(OrdMap<Value, Value>),
    Fun(Fun),
    Cell(Gc<GcCell<Value>>, u64),
    Opaque(Box<Value>, u64),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Value) -> Ordering {
        match self {
            Value::Atomic(Atomic::Nil) => match other {
                Value::Atomic(Atomic::Nil) => Ordering::Equal,
                _ => Ordering::Less,
            }

            Value::Atomic(Atomic::Bool(lhs)) => match other {
                Value::Atomic(Atomic::Nil) => Ordering::Greater,
                Value::Atomic(Atomic::Bool(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Less,
            }

            Value::Atomic(Atomic::Int(lhs)) => match other {
                Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(..)) => Ordering::Greater,
                Value::Atomic(Atomic::Int(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Less,
            }

            Value::Atomic(Atomic::Float(lhs)) => match other {
                Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(..)) | Value::Atomic(Atomic::Int(..)) => Ordering::Greater,
                Value::Atomic(Atomic::Float(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Less,
            }

            Value::Atomic(Atomic::Keyword(lhs)) => match other {
                Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(..)) | Value::Atomic(Atomic::Int(..)) | Value::Atomic(Atomic::Float(..)) => Ordering::Greater,
                Value::Atomic(Atomic::Keyword(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Less,
            }

            Value::Id(Id::User(lhs)) => match other {
                Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(..)) | Value::Atomic(Atomic::Int(..)) | Value::Atomic(Atomic::Float(..)) | Value::Atomic(Atomic::Keyword(..)) => Ordering::Greater,
                Value::Id(Id::User(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Less,
            }

            Value::Id(Id::Symbol(lhs)) => match other {
                Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(..)) | Value::Atomic(Atomic::Int(..)) | Value::Atomic(Atomic::Float(..)) | Value::Atomic(Atomic::Keyword(..)) | Value::Id(Id::User(..)) => Ordering::Greater,
                Value::Id(Id::Symbol(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Less,
            }

            Value::Atomic(Atomic::Char(lhs)) => match other {
                Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(..)) | Value::Atomic(Atomic::Int(..)) | Value::Atomic(Atomic::Float(..)) | Value::Atomic(Atomic::Keyword(..)) | Value::Id(Id::User(..)) | Value::Id(Id::Symbol(..)) => Ordering::Greater,
                Value::Atomic(Atomic::Char(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Less,
            }

            Value::Atomic(Atomic::String(lhs)) => match other {
                Value::Opaque(..) | Value::Cell(..) | Value::Fun(..) | Value::Map(..) | Value::Set(..) | Value::App(..) | Value::Arr(..) | Value::Atomic(Atomic::Bytes(..)) => Ordering::Less,
                Value::Atomic(Atomic::String(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Greater,
            }

            Value::Atomic(Atomic::Bytes(lhs)) => match other {
                Value::Opaque(..) | Value::Cell(..) | Value::Fun(..) | Value::Map(..) | Value::Set(..) | Value::App(..) | Value::Arr(..) => Ordering::Less,
                Value::Atomic(Atomic::Bytes(rhs)) => lhs.cmp(rhs),
                _ => Ordering::Greater,
            }

            Value::Arr(lhs) => match other {
                Value::Opaque(..) | Value::Cell(..) | Value::Fun(..) | Value::Map(..) | Value::Set(..) | Value::App(..) => Ordering::Less,
                Value::Arr(rhs) => lhs.cmp(rhs),
                _ => Ordering::Greater,
            }

            Value::App(lhs) => match other {
                Value::Opaque(..) | Value::Cell(..) | Value::Fun(..) | Value::Map(..) | Value::Set(..) => Ordering::Less,
                Value::App(rhs) => lhs.cmp(rhs),
                _ => Ordering::Greater,
            }

            Value::Set(lhs) => match other {
                Value::Opaque(..) | Value::Cell(..) | Value::Fun(..) | Value::Map(..) => Ordering::Less,
                Value::Set(rhs) => lhs.cmp(rhs),
                _ => Ordering::Greater,
            }

            Value::Map(lhs) => match other {
                Value::Opaque(..) | Value::Cell(..) | Value::Fun(..) => Ordering::Less,
                Value::Map(rhs) => lhs.cmp(rhs),
                _ => Ordering::Greater,
            }

            Value::Fun(lhs) => match other {
                Value::Opaque(..) | Value::Cell(..) => Ordering::Less,
                Value::Fun(rhs) => lhs.cmp(rhs),
                _ => Ordering::Greater,
            }

            Value::Cell(_, left_id) => match other {
                Value::Opaque(..) => Ordering::Less,
                Value::Cell(_, right_id) => left_id.cmp(right_id),
                _ => Ordering::Greater,
            }

            Value::Opaque(left_inner, left_type) => match other {
                Value::Opaque(right_inner, right_type) => {
                    match left_type.cmp(right_type) {
                        Ordering::Less => return Ordering::Less,
                        Ordering::Greater => return Ordering::Greater,
                        Ordering::Equal => return left_inner.cmp(right_inner),
                    }
                }

                _ => Ordering::Greater,
            }
        }
    }
}

impl Value {
    pub fn nil() -> Value {
        Value::Atomic(Atomic::Nil)
    }

    pub fn bool_(b: bool) -> Value {
        Value::Atomic(Atomic::Bool(b))
    }

    pub fn int(n: i64) -> Value {
        Value::Atomic(Atomic::Int(n))
    }

    // Panics if given an infinity or NaN, converts -0.0 to 0.0.
    pub fn float(n: f64) -> Value {
        let n = match n.classify() {
            FpCategory::Nan | FpCategory::Infinite => panic!("Floats must not be NaN or infinite"),
            FpCategory::Zero => 0.0,
            _ => n,
        };

        Value::Atomic(Atomic::Float(unsafe { NotNan::unchecked_new(n) }))
    }

    pub fn char_(c: char) -> Value {
        Value::Atomic(Atomic::Char(c))
    }

    pub fn id(id: Id) -> Value {
        Value::Id(id)
    }

    pub fn id_str(id: &str) -> Value {
        Value::id(Id::user(id))
    }

    pub fn kw(kw: String) -> Value {
        Value::Atomic(Atomic::Keyword(kw))
    }

    pub fn kw_str(kw: &str) -> Value {
        Value::kw(kw.to_string())
    }

    pub fn bytes(b: Vector<u8>) -> Value {
        Value::Atomic(Atomic::Bytes(b))
    }

    pub fn bytes_from_vec(vals: Vec<u8>) -> Value {
        Value::bytes(Vector(ImVector::from(vals)))
    }

    pub fn string(s: Rope) -> Value {
        Value::Atomic(Atomic::String(s))
    }

    pub fn string_from_vec(vals: Vec<char>) -> Value {
        let s: String = vals.into_iter().collect();
        Value::string(Rope(Ropey::from(s)))
    }

    pub fn string_from_str(s: &str) -> Value {
        Value::string(Rope(Ropey::from_str(s)))
    }

    pub fn arr(vals: Vector<Value>) -> Value {
        Value::Arr(vals)
    }

    pub fn arr_from_vec(vals: Vec<Value>) -> Value {
        Value::arr(Vector(ImVector::from(vals)))
    }

    pub fn app(vals: Vector<Value>) -> Value {
        Value::App(vals)
    }

    pub fn app_from_vec(vals: Vec<Value>) -> Value {
        Value::app(Vector(ImVector::from(vals)))
    }

    pub fn set(vals: OrdSet<Value>) -> Value {
        Value::Set(vals)
    }

    pub fn set_from_vec(vals: Vec<Value>) -> Value {
        Value::set(OrdSet(ImOrdSet::from(vals)))
    }

    pub fn map(vals: OrdMap<Value, Value>) -> Value {
        Value::Map(vals)
    }

    pub fn map_from_vec(vals: Vec<(Value, Value)>) -> Value {
        Value::map(OrdMap(ImOrdMap::from(vals)))
    }

    pub fn closure(c: Closure, cx: &mut Context) -> Value {
        Value::Fun(Fun::Closure(c, cx.next_fun_id()))
    }

    pub fn builtin(b: Builtin) -> Value {
        Value::Fun(Fun::Builtin(b))
    }

    pub fn hide(type_id: u64, cx: &mut Context) -> Value {
        Value::Fun(Fun::Opaque {
            hide: true,
            fun_id: cx.next_fun_id(),
            type_id: type_id,
        })
    }

    pub fn unhide(type_id: u64, cx: &mut Context) -> Value {
        Value::Fun(Fun::Opaque {
            hide: false,
            fun_id: cx.next_fun_id(),
            type_id: type_id,
        })
    }

    pub fn symbol(cx: &mut Context) -> Value {
        Value::Id(Id::Symbol(cx.next_symbol_id()))
    }

    pub fn cell(v: &Value, cx: &mut Context) -> Value {
        Value::Cell(Gc::new(GcCell::new(v.clone())), cx.next_cell_id())
    }

    pub fn as_id(&self) -> Option<&Id> {
        match self {
            Value::Id(id) => Some(id),
            _ => None,
        }
    }

    pub fn as_user_id(&self) -> Option<&str> {
        match self {
            Value::Id(Id::User(id)) => Some(id),
            _ => None,
        }
    }

    pub fn as_symbol(&self) -> Option<u64> {
        match self {
            Value::Id(Id::Symbol(id)) => Some(*id),
            _ => None,
        }
    }

    pub fn as_atomic(&self) -> Option<&Atomic> {
        match self {
            Value::Atomic(atomic) => Some(atomic),
            _ => None,
        }
    }

    pub fn as_kw(&self) -> Option<&str> {
        self.as_atomic().and_then(|atomic| match atomic {
            Atomic::Keyword(kw) => Some(kw.as_str()),
            _ => None,
        })
    }

    pub fn is_kw(&self, expected: &str) -> bool {
        match self.as_kw() {
            Some(kw) => kw == expected,
            None => false,
        }
    }

    pub fn as_app(&self) -> Option<&Vector<Value>> {
        match self {
            Value::App(app) => Some(app),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&Vector<Value>> {
        match self {
            Value::Arr(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&OrdMap<Value, Value>> {
        match self {
            Value::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_fun(&self) -> Option<&Fun> {
        match self {
            Value::Fun(fun) => Some(fun),
            _ => None,
        }
    }

    pub fn as_cell(&self) -> Option<&GcCell<Value>> {
        match self {
            Value::Cell(the_gc, _) => Some(the_gc),
            _ => None,
        }
    }

    pub fn as_opaque(&self) -> Option<(&Value, u64)> {
        match self {
            Value::Opaque(the_box, type_id) => Some((the_box, *type_id)),
            _ => None,
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Atomic(Atomic::Nil) | Value::Atomic(Atomic::Bool(false)) => false,
            _ => true,
        }
    }

    pub fn compute(&self, args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
        match self {
            Value::Fun(fun) => fun.compute(args, cx),
            _ => Err(type_error(self, "function")),
        }
    }
}

/// The atomic values are those that do not contain other objects and that use synactic equality.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Atomic {
    Nil,
    Bool(bool),
    Int(i64),
    Float(NotNan),
    Keyword(String),
    Char(char),
    String(Rope),
    Bytes(Vector<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize, Hash)]
pub enum Id {
    User(String),
    Symbol(u64),
}

impl Id {
    pub fn user(id: &str) -> Id {
        Id::User(id.to_string())
    }
}

#[derive(Debug, Clone, Trace, Finalize)]
pub enum Fun {
    Closure(Closure, u64),
    Builtin(Builtin),
    Opaque { hide: bool, fun_id: u64, type_id: u64 },
}

impl PartialEq for Fun {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Fun::Builtin(a), Fun::Builtin(b)) => a.eq(b),
            (Fun::Closure(_, id_a), Fun::Closure(_, id_b)) => id_a.eq(id_b),
            (Fun::Opaque { fun_id: id_a, ..}, Fun::Opaque { fun_id: id_b, ..}) => id_a.eq(id_b),
            (Fun::Opaque { fun_id: id_a, ..}, Fun::Closure(_, id_b)) => id_a.eq(id_b),
            (Fun::Closure(_, id_a), Fun::Opaque { fun_id: id_b, ..}) => id_a.eq(id_b),
            _ => false,
        }
    }
}

impl Eq for Fun {}

impl Ord for Fun {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Fun::Builtin(a), Fun::Builtin(b)) => a.cmp(b),
            (Fun::Builtin(..), _) => Ordering::Less,
            (Fun::Closure(..), Fun::Builtin(..)) => Ordering::Greater,
            (Fun::Opaque {..}, Fun::Builtin(..)) => Ordering::Greater,
            (Fun::Closure(_, id_a), Fun::Closure(_, id_b)) => id_a.cmp(id_b),
            (Fun::Opaque { fun_id: id_a, ..}, Fun::Opaque { fun_id: id_b, ..}) => id_a.cmp(id_b),
            (Fun::Opaque { fun_id: id_a, ..}, Fun::Closure(_, id_b)) => id_a.cmp(id_b),
            (Fun::Closure(_, id_a), Fun::Opaque { fun_id: id_b, ..}) => id_a.cmp(id_b),
        }
    }
}

impl PartialOrd for Fun {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Fun {
    pub fn compute(&self, args: Vector<Value>, cx: &mut Context) -> Result<Value, Value> {
        match self {
            Fun::Closure(c, _) => c.compute(args, cx),

            Fun::Opaque { hide, type_id, ..} => {
                if args.0.len() != 1 {
                    return Err(num_args_error());
                }

                if *hide {
                    return Ok(Value::Opaque(Box::new(args.0[0].clone()), *type_id));
                } else {
                    let arg = &args.0[0];
                    match arg.as_opaque() {
                        None => return Err(type_error_(arg, &Value::Id(Id::Symbol(*type_id)))),
                        Some((inner, actual_type_id)) => {
                            if actual_type_id == *type_id {
                                return Ok(inner.clone());
                            } else {
                                return Err(type_error_(arg, &Value::Id(Id::Symbol(*type_id))));
                            }
                        }
                    }
                }
            }

            Fun::Builtin(Builtin::BoolNot) => builtins::bool_not(args, cx),
            Fun::Builtin(Builtin::BoolAnd) => builtins::bool_and(args, cx),
            Fun::Builtin(Builtin::BoolOr) => builtins::bool_or(args, cx),
            Fun::Builtin(Builtin::BoolIf) => builtins::bool_if(args, cx),
            Fun::Builtin(Builtin::BoolIff) => builtins::bool_iff(args, cx),
            Fun::Builtin(Builtin::BoolXor) => builtins::bool_xor(args, cx),

            Fun::Builtin(Builtin::IntCountOnes) => builtins::int_count_ones(args, cx),
            Fun::Builtin(Builtin::IntCountZeros) => builtins::int_count_zeros(args, cx),
            Fun::Builtin(Builtin::IntLeadingOnes) => builtins::int_leading_ones(args, cx),
            Fun::Builtin(Builtin::IntLeadingZeros) => builtins::int_leading_zeros(args, cx),
            Fun::Builtin(Builtin::IntTrailingOnes) => builtins::int_trailing_ones(args, cx),
            Fun::Builtin(Builtin::IntTrailingZeros) => builtins::int_trailing_zeros(args, cx),
            Fun::Builtin(Builtin::IntRotateLeft) => builtins::int_rotate_left(args, cx),
            Fun::Builtin(Builtin::IntRotateRight) => builtins::int_rotate_right(args, cx),
            Fun::Builtin(Builtin::IntReverseBytes) => builtins::int_reverse_bytes(args, cx),
            Fun::Builtin(Builtin::IntReverseBits) => builtins::int_reverse_bits(args, cx),
            Fun::Builtin(Builtin::IntAdd) => builtins::int_add(args, cx),
            Fun::Builtin(Builtin::IntSub) => builtins::int_sub(args, cx),
            Fun::Builtin(Builtin::IntMul) => builtins::int_mul(args, cx),
            Fun::Builtin(Builtin::IntDiv) => builtins::int_div(args, cx),
            Fun::Builtin(Builtin::IntDivTrunc) => builtins::int_div_trunc(args, cx),
            Fun::Builtin(Builtin::IntMod) => builtins::int_mod(args, cx),
            Fun::Builtin(Builtin::IntModTrunc) => builtins::int_mod_trunc(args, cx),
            Fun::Builtin(Builtin::IntNeg) => builtins::int_neg(args, cx),
            Fun::Builtin(Builtin::IntShl) => builtins::int_shl(args, cx),
            Fun::Builtin(Builtin::IntShr) => builtins::int_shr(args, cx),
            Fun::Builtin(Builtin::IntAbs) => builtins::int_abs(args, cx),
            Fun::Builtin(Builtin::IntPow) => builtins::int_pow(args, cx),
            Fun::Builtin(Builtin::IntAddSat) => builtins::int_add_sat(args, cx),
            Fun::Builtin(Builtin::IntSubSat) => builtins::int_sub_sat(args, cx),
            Fun::Builtin(Builtin::IntMulSat) => builtins::int_mul_sat(args, cx),
            Fun::Builtin(Builtin::IntPowSat) => builtins::int_pow_sat(args, cx),
            Fun::Builtin(Builtin::IntAddWrap) => builtins::int_add_wrap(args, cx),
            Fun::Builtin(Builtin::IntSubWrap) => builtins::int_sub_wrap(args, cx),
            Fun::Builtin(Builtin::IntMulWrap) => builtins::int_mul_wrap(args, cx),
            Fun::Builtin(Builtin::IntDivWrap) => builtins::int_div_wrap(args, cx),
            Fun::Builtin(Builtin::IntDivTruncWrap) => builtins::int_div_trunc_wrap(args, cx),
            Fun::Builtin(Builtin::IntModWrap) => builtins::int_mod_wrap(args, cx),
            Fun::Builtin(Builtin::IntModTruncWrap) => builtins::int_mod_trunc_wrap(args, cx),
            Fun::Builtin(Builtin::IntNegWrap) => builtins::int_neg_wrap(args, cx),
            Fun::Builtin(Builtin::IntAbsWrap) => builtins::int_abs_wrap(args, cx),
            Fun::Builtin(Builtin::IntPowWrap) => builtins::int_pow_wrap(args, cx),
            Fun::Builtin(Builtin::IntSignum) => builtins::int_signum(args, cx),

            Fun::Builtin(Builtin::BytesCount) => builtins::bytes_count(args, cx),
            Fun::Builtin(Builtin::BytesGet) => builtins::bytes_get(args, cx),
            Fun::Builtin(Builtin::BytesInsert) => builtins::bytes_insert(args, cx),
            Fun::Builtin(Builtin::BytesRemove) => builtins::bytes_remove(args, cx),
            Fun::Builtin(Builtin::BytesUpdate) => builtins::bytes_update(args, cx),
            Fun::Builtin(Builtin::BytesSlice) => builtins::bytes_slice(args, cx),
            Fun::Builtin(Builtin::BytesSplice) => builtins::bytes_splice(args, cx),
            Fun::Builtin(Builtin::BytesConcat) => builtins::bytes_concat(args, cx),
            Fun::Builtin(Builtin::BytesIter) => builtins::bytes_iter(args, cx),
            Fun::Builtin(Builtin::BytesIterBack) => builtins::bytes_iter_back(args, cx),

            Fun::Builtin(Builtin::IntToChar) => builtins::int_to_char(args, cx),
            Fun::Builtin(Builtin::IsIntToChar) => builtins::is_int_to_char(args, cx),
            Fun::Builtin(Builtin::CharToInt) => builtins::char_to_int(args, cx),

            Fun::Builtin(Builtin::StrCount) => builtins::str_count(args, cx),
            Fun::Builtin(Builtin::StrCountUtf8) => builtins::str_count_utf8(args, cx),
            Fun::Builtin(Builtin::StrGet) => builtins::str_get(args, cx),
            Fun::Builtin(Builtin::StrGetUtf8) => builtins::str_get_utf8(args, cx),
            Fun::Builtin(Builtin::StrIndexCharToUtf8) => builtins::str_index_char_to_utf8(args, cx),
            Fun::Builtin(Builtin::StrIndexUtf8ToChar) => builtins::str_index_utf8_to_char(args, cx),
            Fun::Builtin(Builtin::StrInsert) => builtins::str_insert(args, cx),
            Fun::Builtin(Builtin::StrRemove) => builtins::str_remove(args, cx),
            Fun::Builtin(Builtin::StrUpdate) => builtins::str_update(args, cx),
            Fun::Builtin(Builtin::StrSlice) => builtins::str_slice(args, cx),
            Fun::Builtin(Builtin::StrSplice) => builtins::str_splice(args, cx),
            Fun::Builtin(Builtin::StrConcat) => builtins::str_concat(args, cx),
            Fun::Builtin(Builtin::StrIter) => builtins::str_iter(args, cx),
            Fun::Builtin(Builtin::StrIterBack) => builtins::str_iter_back(args, cx),
            Fun::Builtin(Builtin::StrIterUtf8) => builtins::str_iter_utf8(args, cx),
            Fun::Builtin(Builtin::StrIterUtf8Back) => builtins::str_iter_utf8_back(args, cx),

            Fun::Builtin(Builtin::FloatAdd) => builtins::float_add(args, cx),
            Fun::Builtin(Builtin::FloatSub) => builtins::float_sub(args, cx),
            Fun::Builtin(Builtin::FloatMul) => builtins::float_mul(args, cx),
            Fun::Builtin(Builtin::FloatDiv) => builtins::float_div(args, cx),
            Fun::Builtin(Builtin::FloatMulAdd) => builtins::float_mul_add(args, cx),
            Fun::Builtin(Builtin::FloatNeg) => builtins::float_neg(args, cx),
            Fun::Builtin(Builtin::FloatFloor) => builtins::float_floor(args, cx),
            Fun::Builtin(Builtin::FloatCeil) => builtins::float_ceil(args, cx),
            Fun::Builtin(Builtin::FloatRound) => builtins::float_round(args, cx),
            Fun::Builtin(Builtin::FloatTrunc) => builtins::float_trunc(args, cx),
            Fun::Builtin(Builtin::FloatFract) => builtins::float_fract(args, cx),
            Fun::Builtin(Builtin::FloatAbs) => builtins::float_abs(args, cx),
            Fun::Builtin(Builtin::FloatSignum) => builtins::float_signum(args, cx),
            Fun::Builtin(Builtin::FloatPow) => builtins::float_pow(args, cx),
            Fun::Builtin(Builtin::FloatSqrt) => builtins::float_sqrt(args, cx),
            Fun::Builtin(Builtin::FloatExp) => builtins::float_exp(args, cx),
            Fun::Builtin(Builtin::FloatExp2) => builtins::float_exp2(args, cx),
            Fun::Builtin(Builtin::FloatLn) => builtins::float_ln(args, cx),
            Fun::Builtin(Builtin::FloatLog2) => builtins::float_log2(args, cx),
            Fun::Builtin(Builtin::FloatLog10) => builtins::float_log10(args, cx),
            Fun::Builtin(Builtin::FloatHypot) => builtins::float_hypot(args, cx),
            Fun::Builtin(Builtin::FloatSin) => builtins::float_sin(args, cx),
            Fun::Builtin(Builtin::FloatCos) => builtins::float_cos(args, cx),
            Fun::Builtin(Builtin::FloatTan) => builtins::float_tan(args, cx),
            Fun::Builtin(Builtin::FloatAsin) => builtins::float_asin(args, cx),
            Fun::Builtin(Builtin::FloatAcos) => builtins::float_acos(args, cx),
            Fun::Builtin(Builtin::FloatAtan) => builtins::float_atan(args, cx),
            Fun::Builtin(Builtin::FloatAtan2) => builtins::float_atan2(args, cx),
            Fun::Builtin(Builtin::FloatExpM1) => builtins::float_exp_m1(args, cx),
            Fun::Builtin(Builtin::FloatLn1P) => builtins::float_ln_1p(args, cx),
            Fun::Builtin(Builtin::FloatSinH) => builtins::float_sinh(args, cx),
            Fun::Builtin(Builtin::FloatCosH) => builtins::float_cosh(args, cx),
            Fun::Builtin(Builtin::FloatTanH) => builtins::float_tanh(args, cx),
            Fun::Builtin(Builtin::FloatAsinH) => builtins::float_asinh(args, cx),
            Fun::Builtin(Builtin::FloatAcosH) => builtins::float_acosh(args, cx),
            Fun::Builtin(Builtin::FloatAtanH) => builtins::float_atanh(args, cx),
            Fun::Builtin(Builtin::FloatIsNormal) => builtins::float_is_normal(args, cx),
            Fun::Builtin(Builtin::FloatToDegrees) => builtins::float_to_degrees(args, cx),
            Fun::Builtin(Builtin::FloatToRadians) => builtins::float_to_radians(args, cx),
            Fun::Builtin(Builtin::FloatToInt) => builtins::float_to_int(args, cx),
            Fun::Builtin(Builtin::IntToFloat) => builtins::int_to_float(args, cx),
            Fun::Builtin(Builtin::FloatToBits) => builtins::float_to_bits(args, cx),
            Fun::Builtin(Builtin::BitsToFloat) => builtins::bits_to_float(args, cx),
            Fun::Builtin(Builtin::IsBitsToFloat) => builtins::is_bits_to_float(args, cx),

            Fun::Builtin(Builtin::StrToId) => builtins::str_to_id(args, cx),
            Fun::Builtin(Builtin::IsStrToId) => builtins::is_str_to_id(args, cx),
            Fun::Builtin(Builtin::IdToStr) => builtins::id_to_str(args, cx),

            Fun::Builtin(Builtin::StrToKw) => builtins::str_to_kw(args, cx),
            Fun::Builtin(Builtin::IsStrToKw) => builtins::is_str_to_kw(args, cx),
            Fun::Builtin(Builtin::KwToStr) => builtins::kw_to_str(args, cx),

            Fun::Builtin(Builtin::ArrCount) => builtins::arr_count(args, cx),
            Fun::Builtin(Builtin::ArrGet) => builtins::arr_get(args, cx),
            Fun::Builtin(Builtin::ArrInsert) => builtins::arr_insert(args, cx),
            Fun::Builtin(Builtin::ArrRemove) => builtins::arr_remove(args, cx),
            Fun::Builtin(Builtin::ArrUpdate) => builtins::arr_update(args, cx),
            Fun::Builtin(Builtin::ArrSlice) => builtins::arr_slice(args, cx),
            Fun::Builtin(Builtin::ArrSplice) => builtins::arr_splice(args, cx),
            Fun::Builtin(Builtin::ArrConcat) => builtins::arr_concat(args, cx),
            Fun::Builtin(Builtin::ArrIter) => builtins::arr_iter(args, cx),
            Fun::Builtin(Builtin::ArrIterBack) => builtins::arr_iter_back(args, cx),

            Fun::Builtin(Builtin::AppCount) => builtins::app_count(args, cx),
            Fun::Builtin(Builtin::AppGet) => builtins::app_get(args, cx),
            Fun::Builtin(Builtin::AppInsert) => builtins::app_insert(args, cx),
            Fun::Builtin(Builtin::AppRemove) => builtins::app_remove(args, cx),
            Fun::Builtin(Builtin::AppUpdate) => builtins::app_update(args, cx),
            Fun::Builtin(Builtin::AppSlice) => builtins::app_slice(args, cx),
            Fun::Builtin(Builtin::AppSplice) => builtins::app_splice(args, cx),
            Fun::Builtin(Builtin::AppConcat) => builtins::app_concat(args, cx),
            Fun::Builtin(Builtin::AppIter) => builtins::app_iter(args, cx),
            Fun::Builtin(Builtin::AppIterBack) => builtins::app_iter_back(args, cx),
            Fun::Builtin(Builtin::AppApply) => builtins::app_apply(args, cx),

            Fun::Builtin(Builtin::SetCount) => builtins::set_count(args, cx),
            Fun::Builtin(Builtin::SetContains) => builtins::set_contains(args, cx),
            Fun::Builtin(Builtin::SetMin) => builtins::set_min(args, cx),
            Fun::Builtin(Builtin::SetMax) => builtins::set_max(args, cx),
            Fun::Builtin(Builtin::SetInsert) => builtins::set_insert(args, cx),
            Fun::Builtin(Builtin::SetRemove) => builtins::set_remove(args, cx),
            Fun::Builtin(Builtin::SetUnion) => builtins::set_union(args, cx),
            Fun::Builtin(Builtin::SetIntersection) => builtins::set_intersection(args, cx),
            Fun::Builtin(Builtin::SetDifference) => builtins::set_difference(args, cx),
            Fun::Builtin(Builtin::SetSymmetricDifference) => builtins::set_symmetric_difference(args, cx),
            Fun::Builtin(Builtin::SetIter) => builtins::set_iter(args, cx),
            Fun::Builtin(Builtin::SetIterBack) => builtins::set_iter_back(args, cx),

            Fun::Builtin(Builtin::MapCount) => builtins::map_count(args, cx),
            Fun::Builtin(Builtin::MapGet) => builtins::map_get(args, cx),
            Fun::Builtin(Builtin::MapContains) => builtins::map_contains(args, cx),
            Fun::Builtin(Builtin::MapMin) => builtins::map_min(args, cx),
            Fun::Builtin(Builtin::MapMinKey) => builtins::map_min_key(args, cx),
            Fun::Builtin(Builtin::MapMinEntry) => builtins::map_min_entry(args, cx),
            Fun::Builtin(Builtin::MapMax) => builtins::map_max(args, cx),
            Fun::Builtin(Builtin::MapMaxKey) => builtins::map_max_key(args, cx),
            Fun::Builtin(Builtin::MapMaxEntry) => builtins::map_max_entry(args, cx),
            Fun::Builtin(Builtin::MapInsert) => builtins::map_insert(args, cx),
            Fun::Builtin(Builtin::MapRemove) => builtins::map_remove(args, cx),
            Fun::Builtin(Builtin::MapUnion) => builtins::map_union(args, cx),
            Fun::Builtin(Builtin::MapIntersection) => builtins::map_intersection(args, cx),
            Fun::Builtin(Builtin::MapDifference) => builtins::map_difference(args, cx),
            Fun::Builtin(Builtin::MapSymmetricDifference) => builtins::map_symmetric_difference(args, cx),
            Fun::Builtin(Builtin::MapIter) => builtins::map_iter(args, cx),
            Fun::Builtin(Builtin::MapIterBack) => builtins::map_iter_back(args, cx),

            Fun::Builtin(Builtin::Symbol) => builtins::symbol(args, cx),

            Fun::Builtin(Builtin::Cell) => builtins::cell(args, cx),
            Fun::Builtin(Builtin::CellGet) => builtins::cell_get(args, cx),
            Fun::Builtin(Builtin::CellSet) => builtins::cell_set(args, cx),

            Fun::Builtin(Builtin::Opaque) => builtins::opaque(args, cx),

            Fun::Builtin(Builtin::Cmp) => builtins::pavo_cmp(args, cx),
            Fun::Builtin(Builtin::Eq) => builtins::pavo_eq(args, cx),
            Fun::Builtin(Builtin::Lt) => builtins::pavo_lt(args, cx),
            Fun::Builtin(Builtin::Lte) => builtins::pavo_lte(args, cx),
            Fun::Builtin(Builtin::Gt) => builtins::pavo_gt(args, cx),
            Fun::Builtin(Builtin::Gte) => builtins::pavo_gte(args, cx),

            Fun::Builtin(Builtin::Read) => builtins::read(args, cx),
            Fun::Builtin(Builtin::Write) => builtins::write(args, cx),
            Fun::Builtin(Builtin::Check) => builtins::check(args, cx),
            Fun::Builtin(Builtin::Eval) => builtins::eval(args, cx),
            Fun::Builtin(Builtin::Expand) => builtins::expand(args, cx),
            Fun::Builtin(Builtin::Exval) => builtins::exval(args, cx),

            Fun::Builtin(Builtin::Typeof) => builtins::typeof_(args, cx),

            Fun::Builtin(Builtin::MacroQuote) => builtins::macro_quote(args, cx),
            Fun::Builtin(Builtin::MacroDo) => builtins::macro_do(args, cx),
            Fun::Builtin(Builtin::MacroSetBang) => builtins::macro_set_bang(args, cx),
            Fun::Builtin(Builtin::MacroThrow) => builtins::macro_throw(args, cx),
            Fun::Builtin(Builtin::MacroIf) => builtins::macro_if(args, cx),
            Fun::Builtin(Builtin::MacroLet) => builtins::macro_let(args, cx),
            Fun::Builtin(Builtin::MacroFn) => builtins::macro_fn(args, cx),

            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Builtin {
    Lt,
    Lte,
    Eq,
    Gt,
    Gte,

    AppApply,
    AppConcat,
    AppCount,
    AppGet,
    AppInsert,
    AppIter,
    AppIterBack,
    AppRemove,
    AppSlice,
    AppSplice,
    AppUpdate,

    ArrConcat,
    ArrCount,
    ArrGet,
    ArrInsert,
    ArrIter,
    ArrIterBack,
    ArrRemove,
    ArrSlice,
    ArrSplice,
    ArrUpdate,

    BitsToFloat,
    IsBitsToFloat,

    BoolAnd,
    BoolIf,
    BoolIff,
    BoolNot,
    BoolOr,
    BoolXor,

    BytesConcat,
    BytesCount,
    BytesGet,
    BytesInsert,
    BytesIter,
    BytesIterBack,
    BytesRemove,
    BytesSlice,
    BytesSplice,
    BytesUpdate,

    Cell,
    CellGet,
    CellSet,

    CharToInt,

    Check,

    Cmp,

    Diverge,

    Eval,
    Expand,
    Exval,

    FloatToBits,
    FloatToDegrees,
    FloatToInt,
    FloatToRadians,

    FloatAbs,
    FloatAcos,
    FloatAcosH,
    FloatAdd,
    FloatAsin,
    FloatAsinH,
    FloatAtan,
    FloatAtanH,
    FloatAtan2,
    FloatCeil,
    FloatCos,
    FloatCosH,
    FloatDiv,
    FloatExp,
    FloatExpM1,
    FloatExp2,
    FloatFloor,
    FloatFract,
    FloatHypot,
    FloatLn,
    FloatLn1P,
    FloatLog10,
    FloatLog2,
    FloatMul,
    FloatMulAdd,
    FloatNeg,
    FloatIsNormal,
    FloatPow,
    FloatRound,
    FloatSignum,
    FloatSin,
    FloatSinH,
    FloatSqrt,
    FloatSub,
    FloatTan,
    FloatTanH,
    FloatTrunc,

    IdToStr,

    IntToFloat,

    IntAbs,
    IntAbsWrap,
    IntAdd,
    IntAddSat,
    IntAddWrap,
    IntCountOnes,
    IntCountZeros,
    IntDiv,
    IntDivTrunc,
    IntDivTruncWrap,
    IntDivWrap,
    IntLeadingOnes,
    IntLeadingZeros,
    IntNeg,
    IntNegWrap,
    IntMod,
    IntModTrunc,
    IntModTruncWrap,
    IntModWrap,
    IntMul,
    IntMulSat,
    IntMulWrap,
    IntPow,
    IntPowSat,
    IntPowWrap,
    IntReverseBits,
    IntReverseBytes,
    IntRotateLeft,
    IntRotateRight,
    IntShl,
    IntShr,
    IntSignum,
    IntSub,
    IntSubWrap,
    IntSubSat,
    IntTrailingOnes,
    IntTrailingZeros,

    IntToChar,
    IsIntToChar,

    KwToStr,

    MacroDo,
    MacroFn,
    MacroIf,
    MacroLet,
    MacroQuote,
    MacroSetBang,
    MacroThrow,

    MapContains,
    MapCount,
    MapDifference,
    MapGet,
    MapInsert,
    MapIntersection,
    MapIter,
    MapIterBack,
    MapMax,
    MapMaxEntry,
    MapMaxKey,
    MapMin,
    MapMinEntry,
    MapMinKey,
    MapRemove,
    MapSymmetricDifference,
    MapUnion,

    Not,

    Opaque,

    Read,
    Require,

    SetContains,
    SetCount,
    SetDifference,
    SetInsert,
    SetIntersection,
    SetIterBack,
    SetIter,
    SetMax,
    SetMin,
    SetRemove,
    SetSymmetricDifference,
    SetUnion,

    StrConcat,
    StrCount,
    StrCountUtf8,
    StrGet,
    StrGetUtf8,
    StrIndexCharToUtf8,
    StrIndexUtf8ToChar,
    StrIter,
    StrIterBack,
    StrIterUtf8,
    StrIterUtf8Back,
    StrInsert,
    StrRemove,
    StrSlice,
    StrSplice,
    StrUpdate,

    StrToId,
    IsStrToId,
    StrToKw,
    IsStrToKw,

    Symbol,

    Typeof,

    Write,
}
