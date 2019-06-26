use std::collections::HashMap;

use im_rc::OrdMap as ImOrdMap;

use crate::value::{Value, Id, Builtin};

pub fn default() -> ImOrdMap<Id, Value> {
    let mut m = ImOrdMap::new();

    env_add(&mut m, "quote", Builtin::MacroQuote);
    env_add(&mut m, "do", Builtin::MacroDo);
    env_add(&mut m, "set!", Builtin::MacroSetBang);
    env_add(&mut m, "throw", Builtin::MacroThrow);
    env_add(&mut m, "if", Builtin::MacroIf);
    env_add(&mut m, "let", Builtin::MacroLet);
    env_add(&mut m, "fn", Builtin::MacroFn);

    return m;

    // TODO macro functions
    // TODO trace

    // env_add(&mut m, "bool-not", Builtin::BoolNot);
    // env_add(&mut m, "bool-and", Builtin::BoolAnd);
    // env_add(&mut m, "bool-or", Builtin::BoolOr);
    // env_add(&mut m, "bool-if", Builtin::BoolIf);
    // env_add(&mut m, "bool-iff", Builtin::BoolIff);
    // env_add(&mut m, "bool-xor", Builtin::BoolXor);
    //
    // env_add_val(&mut m, "int-max-val", Value::int(std::i64::MAX));
    // env_add_val(&mut m, "int-min-val", Value::int(std::i64::MIN));
    // env_add(&mut m, "int-count-ones", Builtin::IntCountOnes);
    // env_add(&mut m, "int-count-zeros", Builtin::IntCountZeros);
    // env_add(&mut m, "int-leading-ones", Builtin::IntLeadingOnes);
    // env_add(&mut m, "int-leading-zeros", Builtin::IntLeadingZeros);
    // env_add(&mut m, "int-trailing-ones", Builtin::IntTrailingOnes);
    // env_add(&mut m, "int-trailing-zeros", Builtin::IntTrailingZeros);
    // env_add(&mut m, "int-rotate-left", Builtin::IntRotateLeft);
    // env_add(&mut m, "int-rotate-right", Builtin::IntRotateRight);
    // env_add(&mut m, "int-reverse-bytes", Builtin::IntReverseBytes);
    // env_add(&mut m, "int-reverse-bits", Builtin::IntReverseBits);
    // env_add(&mut m, "int-add", Builtin::IntAdd);
    // env_add(&mut m, "int-sub", Builtin::IntSub);
    // env_add(&mut m, "int-mul", Builtin::IntMul);
    // env_add(&mut m, "int-div", Builtin::IntDiv);
    // env_add(&mut m, "int-div-trunc", Builtin::IntDivTrunc);
    // env_add(&mut m, "int-mod", Builtin::IntMod);
    // env_add(&mut m, "int-mod-trunc", Builtin::IntModTrunc);
    // env_add(&mut m, "int-neg", Builtin::IntNeg);
    // env_add(&mut m, "int-shl", Builtin::IntShl);
    // env_add(&mut m, "int-shr", Builtin::IntShr);
    // env_add(&mut m, "int-abs", Builtin::IntAbs);
    // env_add(&mut m, "int-pow", Builtin::IntPow);
    // env_add(&mut m, "int-add-sat", Builtin::IntAddSat);
    // env_add(&mut m, "int-sub-sat", Builtin::IntSubSat);
    // env_add(&mut m, "int-mul-sat", Builtin::IntMulSat);
    // env_add(&mut m, "int-pow-sat", Builtin::IntPowSat);
    // env_add(&mut m, "int-add-wrap", Builtin::IntAddWrap);
    // env_add(&mut m, "int-sub-wrap", Builtin::IntSubWrap);
    // env_add(&mut m, "int-mul-wrap", Builtin::IntMulWrap);
    // env_add(&mut m, "int-div-wrap", Builtin::IntDivWrap);
    // env_add(&mut m, "int-div-trunc-wrap", Builtin::IntDivTruncWrap);
    // env_add(&mut m, "int-mod-wrap", Builtin::IntModWrap);
    // env_add(&mut m, "int-mod-trunc-wrap", Builtin::IntModTruncWrap);
    // env_add(&mut m, "int-neg-wrap", Builtin::IntNegWrap);
    // env_add(&mut m, "int-abs-wrap", Builtin::IntAbsWrap);
    // env_add(&mut m, "int-pow-wrap", Builtin::IntPowWrap);
    // env_add(&mut m, "int-signum", Builtin::IntSignum);
    //
    // env_add(&mut m, "bytes-count", Builtin::BytesCount);
    // env_add(&mut m, "bytes-get", Builtin::BytesGet);
    // env_add(&mut m, "bytes-insert", Builtin::BytesInsert);
    // env_add(&mut m, "bytes-remove", Builtin::BytesRemove);
    // env_add(&mut m, "bytes-update", Builtin::BytesUpdate);
    // env_add(&mut m, "bytes-slice", Builtin::BytesSlice);
    // env_add(&mut m, "bytes-splice", Builtin::BytesSplice);
    // env_add(&mut m, "bytes-concat", Builtin::BytesConcat);
    // env_add(&mut m, "bytes-iter", Builtin::BytesIter);
    // env_add(&mut m, "bytes-iter-back", Builtin::BytesIterBack);
    //
    // env_add_val(&mut m, "char-max-val", Value::char_(std::char::MAX));
    // env_add(&mut m, "int=>char", Builtin::IntToChar);
    // env_add(&mut m, "int=>char?", Builtin::IsIntToChar);
    // env_add(&mut m, "char->int", Builtin::CharToInt);
    //
    // env_add(&mut m, "str-count", Builtin::StrCount);
    // env_add(&mut m, "str-count-utf8", Builtin::StrCountUtf8);
    // env_add(&mut m, "str-get", Builtin::StrGet);
    // env_add(&mut m, "str-get-utf8", Builtin::StrGetUtf8);
    // env_add(&mut m, "str-insert", Builtin::StrInsert);
    // env_add(&mut m, "str-remove", Builtin::StrRemove);
    // env_add(&mut m, "str-update", Builtin::StrUpdate);
    // env_add(&mut m, "str-slice", Builtin::StrSlice);
    // env_add(&mut m, "str-splice", Builtin::StrSplice);
    // env_add(&mut m, "str-concat", Builtin::StrConcat);
    // env_add(&mut m, "str-iter", Builtin::StrIter);
    // env_add(&mut m, "str-iter-back", Builtin::StrIterBack);
    // env_add(&mut m, "str-iter-utf8", Builtin::StrIterUtf8);
    // env_add(&mut m, "str-iter-utf8-back", Builtin::StrIterUtf8Back);
    //
    // env_add_val(&mut m, "float-max-val", Value::float(std::f64::MAX));
    // env_add_val(&mut m, "float-min-val", Value::float(std::f64::MIN));
    // env_add(&mut m, "float-add", Builtin::FloatAdd);
    // env_add(&mut m, "float-sub", Builtin::FloatSub);
    // env_add(&mut m, "float-mul", Builtin::FloatMul);
    // env_add(&mut m, "float-div", Builtin::FloatDiv);
    // env_add(&mut m, "float-mul-add", Builtin::FloatMulAdd);
    // env_add(&mut m, "float-neg", Builtin::FloatNeg);
    // env_add(&mut m, "float-floor", Builtin::FloatFloor);
    // env_add(&mut m, "float-ceil", Builtin::FloatCeil);
    // env_add(&mut m, "float-round", Builtin::FloatRound);
    // env_add(&mut m, "float-trunc", Builtin::FloatTrunc);
    // env_add(&mut m, "float-fract", Builtin::FloatFract);
    // env_add(&mut m, "float-abs", Builtin::FloatAbs);
    // env_add(&mut m, "float-signum", Builtin::FloatSignum);
    // env_add(&mut m, "float-powi", Builtin::FloatPowi);
    // env_add(&mut m, "float-powf", Builtin::FloatPowf);
    // env_add(&mut m, "float-sqrt", Builtin::FloatSqrt);
    // env_add(&mut m, "float-exp", Builtin::FloatExp);
    // env_add(&mut m, "float-exp2", Builtin::FloatExp2);
    // env_add(&mut m, "float-ln", Builtin::FloatLn);
    // env_add(&mut m, "float-log2", Builtin::FloatLog2);
    // env_add(&mut m, "float-log10", Builtin::FloatLog10);
    // env_add(&mut m, "float-hypot", Builtin::FloatHypot);
    // env_add(&mut m, "float-sin", Builtin::FloatSin);
    // env_add(&mut m, "float-cos", Builtin::FloatCos);
    // env_add(&mut m, "float-tan", Builtin::FloatTan);
    // env_add(&mut m, "float-asin", Builtin::FloatAsin);
    // env_add(&mut m, "float-acos", Builtin::FloatAcos);
    // env_add(&mut m, "float-atan", Builtin::FloatAtan);
    // env_add(&mut m, "float-atan2", Builtin::FloatAtan2);
    // env_add(&mut m, "float-exp-m1", Builtin::FloatExpM1);
    // env_add(&mut m, "float-ln-1p", Builtin::FloatLn1P);
    // env_add(&mut m, "float-sinh", Builtin::FloatSinH);
    // env_add(&mut m, "float-cosh", Builtin::FloatCosH);
    // env_add(&mut m, "float-tanh", Builtin::FloatTanH);
    // env_add(&mut m, "float-asinh", Builtin::FloatAsinH);
    // env_add(&mut m, "float-acosh", Builtin::FloatAcosH);
    // env_add(&mut m, "float-atanh", Builtin::FloatAtanH);
    // env_add(&mut m, "float-classify", Builtin::FloatClassify);
    // env_add(&mut m, "float-recip", Builtin::FloatRecip);
    // env_add(&mut m, "float->degrees", Builtin::FloatToDegrees);
    // env_add(&mut m, "float-radians", Builtin::FloatToRadians);
    // env_add(&mut m, "float->int", Builtin::FloatToInt);
    // env_add(&mut m, "int->float", Builtin::IntToFloat);
    // env_add(&mut m, "float->bits", Builtin::FloatToBits);
    // env_add(&mut m, "bits=>float", Builtin::BitsToFloat);
    // env_add(&mut m, "bits=>float?", Builtin::IsBitsToFloat);
    //
    // env_add(&mut m, "str=>id", Builtin::StrToId);
    // env_add(&mut m, "str=>id?", Builtin::IsStrToId);
    // env_add(&mut m, "id->str", Builtin::IdToStr);
    //
    // env_add(&mut m, "str=>kw", Builtin::StrToKw);
    // env_add(&mut m, "str=>kw?", Builtin::IsStrToKw);
    // env_add(&mut m, "kw->str", Builtin::KwToStr);
    //
    // env_add(&mut m, "arr-count", Builtin::ArrCount);
    // env_add(&mut m, "arr-get", Builtin::ArrGet);
    // env_add(&mut m, "arr-insert", Builtin::ArrInsert);
    // env_add(&mut m, "arr-remove", Builtin::ArrRemove);
    // env_add(&mut m, "arr-update", Builtin::ArrUpdate);
    // env_add(&mut m, "arr-slice", Builtin::ArrSlice);
    // env_add(&mut m, "arr-splice", Builtin::ArrSplice);
    // env_add(&mut m, "arr-concat", Builtin::ArrConcat);
    // env_add(&mut m, "arr-iter", Builtin::ArrIter);
    // env_add(&mut m, "arr-iter-back", Builtin::ArrIterBack);
    //
    // env_add(&mut m, "app-count", Builtin::AppCount);
    // env_add(&mut m, "app-get", Builtin::AppGet);
    // env_add(&mut m, "app-insert", Builtin::AppInsert);
    // env_add(&mut m, "app-remove", Builtin::AppRemove);
    // env_add(&mut m, "app-update", Builtin::AppUpdate);
    // env_add(&mut m, "app-slice", Builtin::AppSlice);
    // env_add(&mut m, "app-splice", Builtin::AppSplice);
    // env_add(&mut m, "app-concat", Builtin::AppConcat);
    // env_add(&mut m, "app-iter", Builtin::AppIter);
    // env_add(&mut m, "app-iter-back", Builtin::AppIterBack);
    // env_add(&mut m, "app-applyr", Builtin::AppApply);
    //
    // env_add(&mut m, "set-count", Builtin::SetCount);
    // env_add(&mut m, "set-contains", Builtin::SetContains);
    // env_add(&mut m, "set-min", Builtin::SetMin);
    // env_add(&mut m, "set-max", Builtin::SetMax);
    // env_add(&mut m, "set-insert", Builtin::SetInsert);
    // env_add(&mut m, "set-remove", Builtin::SetRemove);
    // env_add(&mut m, "set-union", Builtin::SetUnion);
    // env_add(&mut m, "set-intersection", Builtin::SetIntersection);
    // env_add(&mut m, "set-difference", Builtin::SetDifference);
    // env_add(&mut m, "set-symmetric-difference", Builtin::SetSymmetricDifference);
    // env_add(&mut m, "set-iter", Builtin::SetIter);
    // env_add(&mut m, "set-iter-back", Builtin::SetIterBack);
    //
    // env_add(&mut m, "map-count", Builtin::MapCount);
    // env_add(&mut m, "map-get", Builtin::MapGet);
    // env_add(&mut m, "map-contains", Builtin::MapContains);
    // env_add(&mut m, "map-min", Builtin::MapMin);
    // env_add(&mut m, "map-min-key", Builtin::MapMinKey);
    // env_add(&mut m, "map-min-entry", Builtin::MapMinEntry);
    // env_add(&mut m, "map-max", Builtin::MapMax);
    // env_add(&mut m, "map-max-key", Builtin::MapMaxKey);
    // env_add(&mut m, "map-max-entry", Builtin::MapMaxEntry);
    // env_add(&mut m, "map-insert", Builtin::MapInsert);
    // env_add(&mut m, "map-remove", Builtin::MapRemove);
    // env_add(&mut m, "map-union", Builtin::MapUnion);
    // env_add(&mut m, "map-intersection", Builtin::MapIntersection);
    // env_add(&mut m, "map-difference", Builtin::MapDifference);
    // env_add(&mut m, "map-symmetric-difference", Builtin::MapSymmetricDifference);
    // env_add(&mut m, "map-iter", Builtin::MapIter);
    // env_add(&mut m, "map-iter-back", Builtin::MapIterBack);
    //
    // env_add(&mut m, "symbol", Builtin::Symbol);
    //
    // env_add(&mut m, "cell", Builtin::Cell);
    // env_add(&mut m, "cell-get", Builtin::CellGet);
    // env_add(&mut m, "cell-set", Builtin::CellSet);
    //
    // env_add(&mut m, "opaque", Builtin::Opaque);
    //
    // env_add(&mut m, "=", Builtin::Eq);
    // env_add(&mut m, "<", Builtin::Lt);
    // env_add(&mut m, "<=", Builtin::Lte);
    // env_add(&mut m, ">", Builtin::Gt);
    // env_add(&mut m, ">=", Builtin::Gte);
    //
    // env_add(&mut m, "read", Builtin::Read);
    // env_add(&mut m, "write", Builtin::Write);
    // env_add(&mut m, "expand", Builtin::Expand);
    // env_add(&mut m, "check", Builtin::Check);
    // env_add(&mut m, "eval", Builtin::Eval);
    // env_add(&mut m, "exval", Builtin::Exval);
    //
    // env_add(&mut m, "typeof", Builtin::Typeof);
    // env_add(&mut m, "truthy?", Builtin::Truthy);
    // env_add(&mut m, "falsey?", Builtin::Falsey);
    // env_add(&mut m, "diverge", Builtin::Diverge);
    //
    // env_add(&mut m, "require", Builtin::Require);
    //
    // m
}

fn env_add(
    m: &mut ImOrdMap<Id, Value>,
    name: &str,
    b: Builtin,
) {
    m.insert(
        Id::user(name),
        Value::builtin(b),
    );
}
