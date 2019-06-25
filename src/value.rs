//! Definition of the objects that the language manipulates at runtime.

use std::{
    cmp::Ordering,
    num::FpCategory,
};

use gc::GcCell;
use gc_derive::{Trace, Finalize};
use im_rc::{
    Vector as ImVector,
    OrdSet as ImOrdSet,
    OrdMap as ImOrdMap,
};
use ropey::Rope as Ropey;

use crate::builtins::{self, type_error};
use crate::context::Context;
use crate::gc_foreign::{Vector, OrdSet, OrdMap, NotNan, Rope};
use crate::vm::Closure;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Value {
    Atomic(Atomic),
    Id(Id),
    Arr(Vector<Value>),
    App(Vector<Value>),
    Set(OrdSet<Value>),
    Map(OrdMap<Value, Value>),
    Fun(Fun),
    Cell(Box<GcCell<Value>>, u64),
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

    pub fn symbol(cx: &mut Context) -> Value {
        Value::Id(Id::Symbol(cx.next_symbol_id()))
    }

    pub fn cell(v: &Value, cx: &mut Context) -> Value {
        Value::Cell(Box::new(GcCell::new(v.clone())), cx.next_cell_id())
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
    Char(char),
    String(Rope),
    Bytes(Vector<u8>),
    Keyword(String),
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
}

impl PartialEq for Fun {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Fun::Builtin(a), Fun::Builtin(b)) => a.eq(b),
            (Fun::Closure(_, id_a), Fun::Closure(_, id_b)) => id_a.eq(id_b),
            _ => false,
        }
    }
}

impl Eq for Fun {}

impl Ord for Fun {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Fun::Builtin(a), Fun::Builtin(b)) => a.cmp(b),
            (Fun::Builtin(..), Fun::Closure(..)) => Ordering::Less,
            (Fun::Closure(..), Fun::Builtin(..)) => Ordering::Greater,
            (Fun::Closure(_, id_a), Fun::Closure(_, id_b)) => id_a.cmp(id_b),
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


            Fun::Builtin(Builtin::AppInsert) => builtins::app_insert(args, cx),

            Fun::Builtin(Builtin::Eq) => builtins::pavo_eq(args, cx),

            Fun::Builtin(Builtin::Typeof) => builtins::typeof_(args, cx),

            _ => unimplemented!(),
        }
    }
}

// TODO, FIXME, XXX: Sort summands lexicographically by their pavo name
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Trace, Finalize)]
pub enum Builtin {
    BoolNot,
    BoolAnd,
    BoolOr,
    BoolIf,
    BoolIff,
    BoolXor,

    IntCountOnes,
    IntCountZeros,
    IntLeadingOnes,
    IntLeadingZeros,
    IntTrailingOnes,
    IntTrailingZeros,
    IntRotateLeft,
    IntRotateRight,
    IntReverseBytes,
    IntReverseBits,
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntDivTrunc,
    IntMod,
    IntModTrunc,
    IntNeg,
    IntShl,
    IntShr,
    IntAbs,
    IntPow,
    IntAddSat,
    IntSubSat,
    IntMulSat,
    IntPowSat,
    IntAddWrap,
    IntSubWrap,
    IntMulWrap,
    IntDivWrap,
    IntDivTruncWrap,
    IntModWrap,
    IntModTruncWrap,
    IntNegWrap,
    IntAbsWrap,
    IntPowWrap,
    IntSignum,

    BytesCount,
    BytesGet,
    BytesInsert,
    BytesRemove,
    BytesUpdate,
    BytesSlice,
    BytesSplice,
    BytesConcat,
    BytesIter,
    BytesIterBack,

    IntToChar,
    IsIntToChar,
    CharToInt,

    StrCount,
    StrCountUtf8,
    StrGet,
    StrGetUtf8,
    StrInsert,
    StrRemove,
    StrUpdate,
    StrSlice,
    StrSplice,
    StrConcat,
    StrIter,
    StrIterBack,
    StrIterUtf8,
    StrIterUtf8Back,

    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatMulAdd,
    FloatNeg,
    FloatFloor,
    FloatCeil,
    FloatRound,
    FloatTrunc,
    FloatFract,
    FloatAbs,
    FloatSignum,
    FloatPowi,
    FloatPowf,
    FloatSqrt,
    FloatExp,
    FloatExp2,
    FloatLn,
    FloatLog2,
    FloatLog10,
    FloatHypot,
    FloatSin,
    FloatCos,
    FloatTan,
    FloatAsin,
    FloatAcos,
    FloatAtan,
    FloatAtan2,
    FloatExpM1,
    FloatLn1P,
    FloatSinH,
    FloatCosH,
    FloatTanH,
    FloatAsinH,
    FloatAcosH,
    FloatAtanH,
    FloatClassify,
    FloatRecip,
    FloatToDegrees,
    FloatToRadians,
    FloatToInt,
    IntToFloat,
    FloatToBits,
    BitsToFloat,
    IsBitsToFloat,

    StrToId,
    IsStrToId,
    IdToStr,

    StrToKw,
    IsStrToKw,
    KwToStr,

    ArrCount,
    ArrGet,
    ArrInsert,
    ArrRemove,
    ArrUpdate,
    ArrSlice,
    ArrSplice,
    ArrConcat,
    ArrIter,
    ArrIterBack,

    AppCount,
    AppGet,
    AppInsert,
    AppRemove,
    AppUpdate,
    AppSlice,
    AppSplice,
    AppConcat,
    AppIter,
    AppIterBack,
    AppApply,

    SetCount,
    SetContains,
    SetMin,
    SetMax,
    SetInsert,
    SetRemove,
    SetUnion,
    SetIntersection,
    SetDifference,
    SetSymmetricDifference,
    SetIter,
    SetIterBack,

    MapCount,
    MapGet,
    MapContains,
    MapMin,
    MapMinKey,
    MapMinEntry,
    MapMax,
    MapMaxKey,
    MapMaxEntry,
    MapInsert,
    MapRemove,
    MapUnion,
    MapIntersection,
    MapDifference,
    MapSymmetricDifference,
    MapIter,
    MapIterBack,

    Symbol,

    Cell,
    CellGet,
    CellSet,

    Opaque,

    Eq,
    Lt,
    Lte,
    Gt,
    Gte,

    Read,
    Write,
    Expand,
    Check,
    Eval,
    Exval,

    Typeof,
    Truthy,
    Falsey,
    Diverge,

    Require,

    // TODO macros as functions
}
