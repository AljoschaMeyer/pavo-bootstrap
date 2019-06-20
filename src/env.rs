use std::collections::HashMap;

use gc::GcCell;
use gc_derive::{Trace, Finalize};
use im_rc::OrdMap as ImOrdMap;

use crate::builtins;
use crate::context::Context;
use crate::gc_foreign::OrdMap;
use crate::value::{Value, Id, Builtin};

// TODO sort this lexicographically
static BUILTINS_IDS: [&'static str; 147] = ["bool-not", "bool-and", "bool-or", "bool-if", "bool-iff", "bool-xor", "int-max-val", "int-min-val", "int-count-ones", "int-count-zeroes", "int-leading-ones", "int-leading-zeros", "int-trailing-ones", "int-trailing-zeros", "int-rotate-left", "int-rotate-right", "int-reverse-bytes", "int-reverse-bits", "int-add", "int-sub", "int-mul", "int-div", "int-div-trunc", "int-mod", "int-mod-trunc", "int-neg", "int-shl", "int-shr", "int-abs", "int-pow", "int-add-sat", "int-sub-sat", "int-mul-sat", "int-pow-sat", "int-add-wrap", "int-sub-wrap", "int-mul-wrap", "int-div-wrap", "int-div-trunc-wrap", "int-mod-wrap", "int-mod-trunc-wrap", "int-neg-wrap", "int-abs-wrap", "int-pow-wrap", "int-signum", "bytes-count", "bytes-get", "bytes-insert", "bytes-remove", "bytes-update", "bytes-slice", "bytes-splice", "bytes-concat", "bytes-iter", "bytes-iter-back", "char-max-val", "int=>char", "int=>char?", "char->int", "str-count", "str-count-utf8", "str-get", "str-get-utf8", "str-insert", "str-remove", "str-update", "str-slice", "str-splice", "str-concat", "str-iter", "str-iter-back", "str-iter-utf8", "str-iter-back-utf8", "str=>id", "str=>id?", "id->str", "str=>kw", "str=>kw?", "kw->str", "arr-count", "arr-get", "arr-insert", "arr-remove", "arr-update", "arr-slice", "arr-splice", "arr-concat", "arr-iter", "arr-iter-back", "app-count", "app-get", "app-insert", "app-remove", "app-update", "app-slice", "app-splice", "app-concat", "app-iter", "app-iter-back", "set-count", "set-contains?", "set-min", "set-max", "set-insert", "set-remove", "set-union", "set-intersection", "set-difference", "set-symmetric-difference", "set-iter", "set-iter-back", "map-count", "map-get", "map-contains?", "map-min", "map-min-key", "map-min-entry", "map-max", "map-max-key", "map-max-entry", "map-insert", "map-remove", "map-union", "map-intersection", "map-difference", "map-symmetric-difference", "map-iter", "map-iter-back", "symbol", "cell", "cell-get", "cell-set", "=", "<", "<=", ">", ">=", "read", "write", "expand", "check", "eval", "exval", "apply", "typeof", "truthy?", "diverge"];

pub fn default(cx: &mut Context) -> HashMap<Id, Value> {
    let mut m = HashMap::new();

    // TODO sort these to be added in lexicographical order such that the ordering between builtin functions is equal to the lexicographical ordering of their names

    env_add(&mut m, "bool-not", builtins::bool_not, cx);
    env_add(&mut m, "bool-and", builtins::bool_and, cx);
    env_add(&mut m, "bool-or", builtins::bool_or, cx);
    env_add(&mut m, "bool-if", builtins::bool_if, cx);
    env_add(&mut m, "bool-iff", builtins::bool_iff, cx);
    env_add(&mut m, "bool-xor", builtins::bool_xor, cx);

    env_add_val(&mut m, "int-max-val", Value::int(std::i64::MAX), cx);
    env_add_val(&mut m, "int-min-val", Value::int(std::i64::MIN), cx);
    env_add(&mut m, "int-count-ones", builtins::int_count_ones, cx);
    env_add(&mut m, "int-count-zeros", builtins::int_count_zeros, cx);
    env_add(&mut m, "int-leading-ones", builtins::int_leading_ones, cx);
    env_add(&mut m, "int-leading-zeros", builtins::int_leading_zeros, cx);
    env_add(&mut m, "int-trailing-ones", builtins::int_trailing_ones, cx);
    env_add(&mut m, "int-trailing-zeros", builtins::int_trailing_zeros, cx);
    env_add(&mut m, "int-rotate-left", builtins::int_rotate_left, cx);
    env_add(&mut m, "int-rotate-right", builtins::int_rotate_right, cx);
    env_add(&mut m, "int-reverse-bytes", builtins::int_reverse_bytes, cx);
    env_add(&mut m, "int-reverse-bits", builtins::int_reverse_bits, cx);
    env_add(&mut m, "int-add", builtins::int_add, cx);
    env_add(&mut m, "int-sub", builtins::int_sub, cx);
    env_add(&mut m, "int-mul", builtins::int_mul, cx);
    env_add(&mut m, "int-div", builtins::int_div, cx);
    env_add(&mut m, "int-div-trunc", builtins::int_div_trunc, cx);
    env_add(&mut m, "int-mod", builtins::int_mod, cx);
    env_add(&mut m, "int-mod-trunc", builtins::int_mod_trunc, cx);
    env_add(&mut m, "int-neg", builtins::int_neg, cx);
    env_add(&mut m, "int-shl", builtins::int_shl, cx);
    env_add(&mut m, "int-shr", builtins::int_shr, cx);
    env_add(&mut m, "int-abs", builtins::int_abs, cx);
    env_add(&mut m, "int-pow", builtins::int_pow, cx);
    env_add(&mut m, "int-add-sat", builtins::int_add_sat, cx);
    env_add(&mut m, "int-sub-sat", builtins::int_sub_sat, cx);
    env_add(&mut m, "int-mul-sat", builtins::int_mul_sat, cx);
    env_add(&mut m, "int-pow-sat", builtins::int_pow_sat, cx);
    env_add(&mut m, "int-add-wrap", builtins::int_add_wrap, cx);
    env_add(&mut m, "int-sub-wrap", builtins::int_sub_wrap, cx);
    env_add(&mut m, "int-mul-wrap", builtins::int_mul_wrap, cx);
    env_add(&mut m, "int-div-wrap", builtins::int_div_wrap, cx);
    env_add(&mut m, "int-div-trunc-wrap", builtins::int_div_trunc_wrap, cx);
    env_add(&mut m, "int-mod-wrap", builtins::int_mod_wrap, cx);
    env_add(&mut m, "int-mod-trunc-wrap", builtins::int_mod_trunc_wrap, cx);
    env_add(&mut m, "int-neg-wrap", builtins::int_neg_wrap, cx);
    env_add(&mut m, "int-abs-wrap", builtins::int_abs_wrap, cx);
    env_add(&mut m, "int-pow-wrap", builtins::int_pow_wrap, cx);
    env_add(&mut m, "int-signum", builtins::int_signum, cx);

    env_add(&mut m, "bytes-count", builtins::bytes_count, cx);
    env_add(&mut m, "bytes-get", builtins::bytes_get, cx);
    env_add(&mut m, "bytes-insert", builtins::bytes_insert, cx);
    env_add(&mut m, "bytes-remove", builtins::bytes_remove, cx);
    env_add(&mut m, "bytes-update", builtins::bytes_update, cx);
    env_add(&mut m, "bytes-slice", builtins::bytes_slice, cx);
    env_add(&mut m, "bytes-splice", builtins::bytes_splice, cx);
    env_add(&mut m, "bytes-concat", builtins::bytes_concat, cx);
    env_add(&mut m, "bytes-iter", builtins::bytes_iter, cx);
    env_add(&mut m, "bytes-iter-back", builtins::bytes_iter_back, cx);

    env_add_val(&mut m, "char-max-val", Value::char_(std::char::MAX), cx);
    env_add(&mut m, "int=>char", builtins::int_to_char, cx);
    env_add(&mut m, "int=>char?", builtins::is_int_to_char, cx);
    env_add(&mut m, "char->int", builtins::char_to_int, cx);

    env_add(&mut m, "str-count", builtins::str_count, cx);
    env_add(&mut m, "str-get", builtins::str_get, cx);
    env_add(&mut m, "str-insert", builtins::str_insert, cx);
    env_add(&mut m, "str-remove", builtins::str_remove, cx);
    env_add(&mut m, "str-update", builtins::str_update, cx);
    env_add(&mut m, "str-slice", builtins::str_slice, cx);
    env_add(&mut m, "str-splice", builtins::str_splice, cx);
    env_add(&mut m, "str-concat", builtins::str_concat, cx);
    env_add(&mut m, "str-iter", builtins::str_iter, cx);
    env_add(&mut m, "str-iter-back", builtins::str_iter_back, cx);

    env_add(&mut m, "str=>id", builtins::str_to_id, cx);
    env_add(&mut m, "str=>id?", builtins::is_str_to_id, cx);
    env_add(&mut m, "id->str", builtins::id_to_str, cx);

    env_add(&mut m, "str=>kw", builtins::str_to_kw, cx);
    env_add(&mut m, "str=>kw?", builtins::is_str_to_kw, cx);
    env_add(&mut m, "kw->str", builtins::kw_to_str, cx);

    env_add(&mut m, "arr-count", builtins::arr_count, cx);
    env_add(&mut m, "arr-get", builtins::arr_get, cx);
    env_add(&mut m, "arr-insert", builtins::arr_insert, cx);
    env_add(&mut m, "arr-remove", builtins::arr_remove, cx);
    env_add(&mut m, "arr-update", builtins::arr_update, cx);
    env_add(&mut m, "arr-slice", builtins::arr_slice, cx);
    env_add(&mut m, "arr-splice", builtins::arr_splice, cx);
    env_add(&mut m, "arr-concat", builtins::arr_concat, cx);
    env_add(&mut m, "arr-iter", builtins::arr_iter, cx);
    env_add(&mut m, "arr-iter-back", builtins::arr_iter_back, cx);

    env_add(&mut m, "app-count", builtins::app_count, cx);
    env_add(&mut m, "app-get", builtins::app_get, cx);
    env_add(&mut m, "app-insert", builtins::app_insert, cx);
    env_add(&mut m, "app-remove", builtins::app_remove, cx);
    env_add(&mut m, "app-update", builtins::app_update, cx);
    env_add(&mut m, "app-slice", builtins::app_slice, cx);
    env_add(&mut m, "app-splice", builtins::app_splice, cx);
    env_add(&mut m, "app-concat", builtins::app_concat, cx);
    env_add(&mut m, "app-iter", builtins::app_iter, cx);
    env_add(&mut m, "app-iter-back", builtins::app_iter_back, cx);

    env_add(&mut m, "set-count", builtins::set_count, cx);
    env_add(&mut m, "set-contains?", builtins::set_contains, cx);
    env_add(&mut m, "set-min", builtins::set_min, cx);
    env_add(&mut m, "set-max", builtins::set_max, cx);
    env_add(&mut m, "set-insert", builtins::set_insert, cx);
    env_add(&mut m, "set-remove", builtins::set_remove, cx);
    env_add(&mut m, "set-union", builtins::set_union, cx);
    env_add(&mut m, "set-intersection", builtins::set_intersection, cx);
    env_add(&mut m, "set-difference", builtins::set_difference, cx);
    env_add(&mut m, "set-symmetric-difference", builtins::set_symmetric_difference, cx);
    env_add(&mut m, "set-iter", builtins::set_iter, cx);
    env_add(&mut m, "set-iter-back", builtins::set_iter_back, cx);

    env_add(&mut m, "map-count", builtins::map_count, cx);
    env_add(&mut m, "map-get", builtins::map_get, cx);
    env_add(&mut m, "map-contains?", builtins::map_contains, cx);
    env_add(&mut m, "map-min", builtins::map_min, cx);
    env_add(&mut m, "map-min-key", builtins::map_min_key, cx);
    env_add(&mut m, "map-min-entry", builtins::map_min_entry, cx);
    env_add(&mut m, "map-max", builtins::map_max, cx);
    env_add(&mut m, "map-max-key", builtins::map_max_key, cx);
    env_add(&mut m, "map-max-entry", builtins::map_max_entry, cx);
    env_add(&mut m, "map-insert", builtins::map_insert, cx);
    env_add(&mut m, "map-remove", builtins::map_remove, cx);
    env_add(&mut m, "map-union", builtins::map_union, cx);
    env_add(&mut m, "map-intersection", builtins::map_intersection, cx);
    env_add(&mut m, "map-difference", builtins::map_difference, cx);
    env_add(&mut m, "map-symmetric-difference", builtins::map_symmetric_difference, cx);
    env_add(&mut m, "map-iter", builtins::map_iter, cx);
    env_add(&mut m, "map-iter-back", builtins::map_iter_back, cx);

    env_add(&mut m, "symbol", builtins::symbol, cx);

    env_add(&mut m, "=", builtins::pavo_eq, cx);
    env_add(&mut m, "<", builtins::pavo_lt, cx);
    env_add(&mut m, "<=", builtins::pavo_lte, cx);
    env_add(&mut m, ">", builtins::pavo_gt, cx);
    env_add(&mut m, ">=", builtins::pavo_gte, cx);

    // TODO read, read_prefix
    env_add(&mut m, "write", builtins::write, cx);

    env_add(&mut m, "typeof", builtins::typeof_, cx);
    env_add(&mut m, "truthy?", builtins::is_truthy, cx);
    env_add(&mut m, "diverge", builtins::diverge, cx);

    // TODO read, read-prefix, write, (expand, evaluate, etc.)
    // TODO floats
    // TODO cell? opaque?

    m
}

fn env_add(
    m: &mut HashMap<Id, Value>,
    name: &str,
    b: fn(Value, &mut Context) -> Result<Value, Value>,
    cx: &mut Context,
) {
    m.insert(
        Id::user(name),
        Value::builtin(Builtin(b), cx)
    );
}

fn env_add_val(
    m: &mut HashMap<Id, Value>,
    name: &str,
    v: Value,
    _cx: &mut Context,
) {
    m.insert(
        Id::user(name),
        v,
    );
}
