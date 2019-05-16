# The Pavo Language Reference

## Syntax

## Values

## Evaluation

## Macro Expansion

## Toplevel Macros

## Toplevel Values

These are all the values that are bound to an identifier in the toplevel environment in which code is executed by default. All of these bindings are immutable.

In the example code blocks, all "statements" evaluate to `nil`, none throws. If you put all examples into a `sf-do` special form, it would evaluate to `nil`. TODO introduce assertion functions/macros.

The given time complexities are the minimum that a pavo implementation must provide. An implementation is of course free to provide *better* complexity bounds than those required. In particular, any amortized complexity bound can be implemented as non-amortized. The converse is not true: If a complexity requirement is unamortized, then implementations are not allowed to provide only amortized bounds.

All time complexities are allowed to be probabilistic, but they must be preserved under adversarial input (under the assumption that the adversary can not predict the source of randomness).

Whenever a function is described to "throw a type error", it throws a map `{ :tag :err-type, :expected <expected>, :got <got>}` where `<expected>` and `<got>` are the keywords denoting the respective types (see `(typeof x)`). Type errors are also trown when an argument is described as having a certain type, but an argument of a different type is supplied. For example "Do foo to the int `n`" throws a type error with `:expected :int` if `n` is not an int.

Whenever an argument is referred to as a "positive int", but an int less than zero is supplied, the function throws `{ :tag :err-negative, :got <got>}`, where `<got>` is the supplied, negative int.

TODO Specify errors that are thrown on incorrect number of args
TODO specify in which order all errors apply.

### Bool

#### `(bool-not b)`

Computes [logical negation](https://en.wikipedia.org/wiki/Negation) `¬b` on bools.

Throws a type error if the arguments is not a bool.

```pavo
(assert (bool-not false))
(assert-not (bool-not true))

(assert-throw (bool-not 0) {
    :tag :err-type,
    :expected :bool,
    :got :int,
})
```

#### `(bool-and b0 b1)`

Computes [logical conjunction](https://en.wikipedia.org/wiki/Logical_conjunction) `b0 ∧ b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert-not (bool-and false false))
(assert-not (bool-and false true))
(assert-not (bool-and true false))
(assert (bool-and true true))

(assert-throw (bool-and false 0) {
    :tag :err-type,
    :expected :bool,
    :got :int,
})
```

#### `(bool-or b0 b1)`

Computes [logical disjunction](https://en.wikipedia.org/wiki/Logical_disjunction) `b0 ∨ b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert-not (bool-or false false))
(assert (bool-or false true))
(assert (bool-or true false))
(assert (bool-or true true))

(assert-throw (bool-or true 1) {
    :tag :err-type,
    :expected :bool,
    :got :int,
})
```

#### `(bool-if b0 b1)`

Computes [logical implication](https://en.wikipedia.org/wiki/https://en.wikipedia.org/wiki/Material_conditional) `b0 → b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert (bool-if false false))
(assert (bool-if false true))
(assert-not (bool-if true false))
(assert (bool-if true true))

(assert-throw (bool-if false 1) {
    :tag :err-type,
    :expected :bool,
    :got :int,
})
```

#### `(bool-iff b0 b1)`

Computes [logical biimplication](https://en.wikipedia.org/wiki/Logical_biconditional) `b0 ↔ b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert (bool-iff false false))
(assert-not (bool-iff false true))
(assert-not (bool-iff true false))
(assert (bool-iff true true))

(assert-throw (bool-iff false 1) {
    :tag :err-type,
    :expected :bool,
    :got :int,
})
```

#### `(bool-xor b0 b1)`

Computes [logical exclusive disjunction](https://en.wikipedia.org/wiki/Exclusive_or) `b0 ⊕ b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert-not (bool-xor false false))
(assert (bool-xor false true))
(assert (bool-xor true false))
(assert-not (bool-xor true true))

(assert-throw (bool-xor false 1) {
    :tag :err-type,
    :expected :bool,
    :got :int,
})
```

### Int

Most of this has been taken/adapted from the [rust i64 docs](https://doc.rust-lang.org/std/primitive.i64.html). A helpful discussion of various design choices for the behavior of the modulus and division operations is [Boute, Raymond T. "The Euclidean definition of the functions div and mod."](https://biblio.ugent.be/publication/314490/file/452146.pdf).

#### `int-max`

The largest integer, `2^63 - 1`.

```pavo
(assert-eq int-max 9223372036854775807)
(assert-throw (+ int-max 1) { :tag :err-wrap-int })
```

#### `int-min`

The smallest integer, `- 2^63`.

```pavo
(assert-eq int-min -9223372036854775808)
(assert-throw (- int-min 1) { :tag :err-wrap-int })
```

#### `(int-count-ones n)`

Returns the number of ones in the binary representation of the int `n`.

```pavo
(assert-eq (int-count-ones 126) 6)
```

#### `(int-count-zeros n)`

Returns the number of zeros in the binary representation of the int `n`.

```pavo
(assert-eq (int-count-zeros 126) 58)
```

#### `(int-leading-ones n)`

Returns the number of leading ones in the binary representation of the int `n`.

```pavo
(assert-eq (int-leading-ones 13) 2)
```

#### `(int-leading-zeros n)`

Returns the number of leading zeros in the binary representation of the int `n`.

```pavo
(assert-eq (int-leading-ones 13) 60)
```

#### `(int-trailing-ones n)`

Returns the number of trailing ones in the binary representation of the int `n`.

```pavo
(assert-eq (int-leading-ones 3) 2)
```

#### `(int-trailing-zeros n)`

Returns the number of trailing zeros in the binary representation of the int `n`.

```pavo
(assert-eq (int-leading-zeros 4) 2)
```

#### `(int-rotate-left n by)`

Shifts the bits of the int `n` to the left by the amount `by`, wrapping the truncated bits to the end of the resulting int.

```pavo
(assert-eq (int-rotate-left 12 0xaa00000000006e1) 0x6e10aa)
```

#### `(int-rotate-right n by)`

Shifts the bits of the int `n` to the right by the positive int `by`, wrapping the truncated bits to the beginning of the resulting int.

```pavo
(assert-eq (int-rotate-left 12 0x6e10aa) 0xaa00000000006e1)
```

#### `(int-reverse-bytes n)`

Reverses the [byte order](https://en.wikipedia.org/wiki/Endianness) of the int `n`.

```pavo
(assert-eq (int-reverse-bytes 0x1234567890123456) 0x5634129078563412)
```

#### `(int-reverse-bits n)`

Reverses the binary representation of the int `n`.

```pavo
(assert-eq (int-reverse-bits 0x1234567890123456) 0x6a2c48091e6a2c48)
```

#### `(int-add n m)` `(int-add n m default)`

Adds the int `n` to the int `m`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow.

```pavo
(assert-eq (int-add 1 2) 3)
(assert-eq (int-add 1 -2) -1)
(assert-throw (int-add int-max 1) { :tag :err-wrap-int })
(assert-eq (int-add int-max 1 nil) nil)
```

#### `(int-sub n m)` `(int-sub n m default)`

Subtracts the int `m` from the int `n`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow.

```pavo
(assert-eq (int-sub 1 2) -1)
(assert-eq (int-sub 1 -2) 3)
(assert-throw (int-sub int-min 1) { :tag :err-wrap-int })
(assert-eq (int-sub int-min 1 nil) nil)
```

#### `(int-mul n m)` `(int-mul n m default)`

Multiplies the int `n` with the int `m`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow.

```pavo
(assert-eq (int-mul 2 3) 6)
(assert-eq (int-mul 2 -3) -6)
(assert-throw (int-mul int-max 2) { :tag :err-wrap-int })
(assert-eq (int-mul int-min 2 nil) nil)
```

#### `(int-div n m)` `(int-div n m default)`

Divides the int `n` by the int `m`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow. If `default` is not supplied, throws `{ :tag :err-zero }` if `m` is `0`. If `default` is supplied, returns `default` if `m` is `0`.

This computes the quotient of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-div 8 3) 2)
(assert-eq (int-div -8 3) -3)
(assert-throw (int-div int-min -1) { :tag :err-wrap-int })
(assert-eq (int-div int-min -1 nil) nil)
(assert-throw (int-div 1 0) { :tag :err-zero })
(assert-eq (int-div 1 0 nil) nil)
```

#### `(int-div-trunc n m)` `(int-div-trunc n m default)`

Divides the int `n` by the int `m`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow. If `default` is not supplied, throws `{ :tag :err-zero }` if `m` is `0`. If `default` is supplied, returns `default` if `m` is `0`.

This computes the quotient of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-div-trunc 8 3) 2)
(assert-eq (int-div-trunc -8 3) -2)
(assert-throw (int-div-trunc int-min -1) { :tag :err-wrap-int })
(assert-eq (int-div-trunc int-min -1 nil) nil)
(assert-throw (int-div-trunc 1 0) { :tag :err-zero })
(assert-eq (int-div-trunc 1 0 nil) nil)
```

#### `(int-mod n m)` `(int-mod n m default)`

Computes the int `n` modulo the int `m`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow. If `default` is not supplied, throws `{ :tag :err-zero }` if `m` is `0`. If `default` is supplied, returns `default` if `m` is `0`.

This computes the remainder of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-mod 8 3) 2)
(assert-eq (int-mod -8 3) 1)
(assert-throw (int-mod int-min -1) { :tag :err-wrap-int })
(assert-eq (int-mod int-min -1 nil) nil)
(assert-throw (int-mod 1 0) { :tag :err-zero })
(assert-eq (int-mod 1 0 nil) nil)
```

#### `(int-mod-trunc n m)` `(int-mod-trunc n m default)`

Computes the int `n` modulo the int `m`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow. If `default` is not supplied, throws `{ :tag :err-zero }` if `m` is `0`. If `default` is supplied, returns `default` if `m` is `0`.

This computes the remainder of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-mod-trunc 8 3) 2)
(assert-eq (int-mod-trunc -8 3) -2)
(assert-throw (int-mod-trunc int-min -1) { :tag :err-wrap-int })
(assert-eq (int-mod-trunc int-min -1 nil) nil)
(assert-throw (int-mod-trunc 1 0) { :tag :err-zero })
(assert-eq (int-mod-trunc 1 0 nil) nil)
```

#### `(int-neg n)` `(int-neg n default)`

Negates the int `n`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow.

```pavo
(assert-eq (int-neg 42) -42)
(assert-eq (int-neg -42) 42)
(assert-eq (int-neg 0) 0)
(assert-throw (int-neg int-min) { :tag :err-wrap-int })
(assert-eq (int-neg int-min nil) nil)
```

#### `(int-shl n m)`

Performs a [logical left shift](https://en.wikipedia.org/wiki/Logical_shift) of the int `n` by the positive int `m` many bits. This always results in `0` if `m` is greater than `63`.

```pavo
(assert-eq (int-shl 5 1) 13)
(assert-eq (int-shl 42 64) 0)
```

#### `(int-shr n m)`

Performs a [logical right shift](https://en.wikipedia.org/wiki/Logical_shift) of the int `n` by the int `m` many bits. This always results in `0` if `m` is greater than `63`.

```pavo
(assert-eq (int-shr 5 1) 2)
(assert-eq (int-shr 42 64) 0)
```

#### `(int-abs n)` `(int-abs n default)`

Returns the absolute value of the int `n`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow.

```pavo
(assert-eq (int-abs 42) 42)
(assert-eq (int-abs -42) 42)
(assert-eq (int-abs 0) 0)
(assert-throw (int-abs int-min) { :tag :err-wrap-int })
(assert-eq (int-abs int-min nil) nil)
```

#### `(int-pow n m)` `(int-pow n m default)`

Computes the int `n` to the power of the positive int `m`. If `default` is not supplied, throws `{ :tag :err-wrap-int }` in case of an overflow. If `default` is supplied, returns `default` in case of an overflow.

```pavo
(assert-eq (int-pow 2 3) 8)
(assert-eq (int-pow 2 0) 1)
(assert-eq (int-pow 0 999) 0)
(assert-eq (int-pow 1 999) 1)
(assert-eq (int-pow -1 999) -1)
(assert-throw (int-pow 99 99) { :tag :err-wrap-int })
(assert-eq (int-pow 99 99 nil) nil)
```

#### `(int-add-sat n m)`

Adds the int `n` to the int `m`, saturating at the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-add-sat 1 2) 3)
(assert-eq (int-add-sat 1 -2) -1)
(assert-eq (int-add-sat int-max 1) int-max)
(assert-eq (int-add-sat int-min -1) int-min)
```

#### `(int-sub-sat n m)`

Subtracts the int `n` from the int `m`, saturating at the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-sub-sat 1 2) -1)
(assert-eq (int-sub-sat 1 -2) 3)
(assert-eq (int-sub-sat int-min 1) int-min)
(assert-eq (int-sub-sat int-max -1) int-max)
```

#### `(int-mul-sat n m)`

Multiplies the int `n` with the int `m`, saturating at the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-mul-sat 2 3) 6)
(assert-eq (int-mul-sat 2 -3) -6)
(assert-eq (int-mul-sat int-max 2) int-max)
(assert-eq (int-mul-sat int-min 2) int-min)
```

#### `(int-pow-sat n m)` `(int-pow n m default)`

Computes the int `n` to the power of the positive int `m`, saturating at the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-pow 2 3) 8)
(assert-eq (int-pow 2 0) 1)
(assert-eq (int-pow 0 999) 0)
(assert-eq (int-pow 1 999) 1)
(assert-eq (int-pow -1 999) -1)
(assert-eq (int-pow 99 99) int-max)
(assert-eq (int-pow -99 99) int-min)
```

#### `(int-add-wrap n m)`

Adds the int `n` to the int `m`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-add-wrap 1 2) 3)
(assert-eq (int-add-wrap int-max 1) int-min)
(assert-eq (int-add-wrap int-min -1) int-max)
```

#### `(int-sub-wrap n m)`

Subtracts the int `n` from the int `m`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-sub-wrap 1 2) -1)
(assert-eq (int-sub-wrap int-min 1) int-max)
(assert-eq (int-sub-wrap int-max -1) int-min)
```

#### `(int-mul-wrap n m)`

Muliplies the int `n` with the int `m`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-mul-wrap 2 3) 6)
(assert-eq (int-mul-wrap int-max 2) 2)
(assert-eq (int-mul-wrap int-max -2) 2)
(assert-eq (int-mul-wrap int-min 2) 0)
(assert-eq (int-mul-wrap int-min -2) 0)
```

#### `(int-div-wrap n m)` `(int-div-wrap n m default)`

Divides the int `n` by the int `m`, wrapping around the numeric bounds instead of overflowing. If `default` is not supplied, throws `{ :tag :err-zero }` if `m` is `0`. If `default` is supplied, returns `default` if `m` is `0`.

This computes the quotient of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-div-wrap 8 3) 2)
(assert-eq (int-div-wrap -8 3) -3)
(assert-eq (int-div-wrap int-min -1) int-min)
(assert-throw (int-div-wrap 1 0) { :tag :err-zero })
(assert-eq (int-div-wrap 1 0 nil) nil)
```

#### `(int-div-trunc-wrap n m)` `(int-div-trunc-wrap n m default)`

Divides the int `n` by the int `m`, wrapping around the numeric bounds instead of overflowing. If `default` is not supplied, throws `{ :tag :err-zero }` if `m` is `0`. If `default` is supplied, returns `default` if `m` is `0`.

This computes the quotient of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-div-trunc-wrap 8 3) 2)
(assert-eq (int-div-trunc-wrap -8 3) -2)
(assert-eq (int-div-trunc-wrap int-min -1) int-min)
(assert-throw (int-div-trunc-wrap 1 0) { :tag :err-zero })
(assert-eq (int-div-trunc-wrap 1 0 nil) nil)
```

#### `(int-mod-wrap n m)` `(int-mod-wrap n m default)`

Computes the int `n` modulo the int `m`, wrapping around the numeric bounds instead of overflowing. If `default` is not supplied, throws `{ :tag :err-zero }` if `m` is `0`. If `default` is supplied, returns `default` if `m` is `0`.

This computes the remainder of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-mod-wrap 8 3) 2)
(assert-eq (int-mod-wrap -8 3) 1)
(assert-eq (int-mod-wrap int-min -1) 0)
(assert-throw (int-mod-wrap 1 0) { :tag :err-zero })
(assert-eq (int-mod-wrap 1 0 nil) nil)
```

#### `(int-mod-trunc-wrap n m)` `(int-mod-trunc-wrap n m default)`

Computes the int `n` modulo the int `m`, wrapping around the numeric bounds instead of overflowing. If `default` is not supplied, throws `{ :tag :err-zero }` if `m` is `0`. If `default` is supplied, returns `default` if `m` is `0`.

This computes the remainder of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-mod-trunc-wrap 8 3) 2)
(assert-eq (int-mod-trunc-wrap -8 3) -2)
(assert-eq (int-mod-trunc-wrap int-min -1) 0)
(assert-throw (int-mod-trunc-wrap 1 0) { :tag :err-zero })
(assert-eq (int-mod-trunc-wrap 1 0 nil) nil)
```

#### `(int-neg-wrap n)`

Negates the int `n`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-neg 42) -42)
(assert-eq (int-neg -42) 42)
(assert-eq (int-neg 0) 0)
(assert-eq (int-neg int-min) int-min)
```

#### `(int-abs-wrap n)` `(int-abs-wrap n default)`

Returns the absolute value of the int `n`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-abs-wrap 42) 42)
(assert-eq (int-abs-wrap -42) 42)
(assert-eq (int-abs-wrap 0) 0)
(assert-eq (int-abs-wrap int-min) int-min)
```

#### `(int-pow-wrap n m)` `(int-pow-wrap n m default)`

Computes the int `n` to the power of the positive int `m`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-pow 2 3) 8)
(assert-eq (int-pow 2 0) 1)
(assert-eq (int-pow 0 999) 0)
(assert-eq (int-pow 1 999) 1)
(assert-eq (int-pow -1 999) -1)
(assert-eq (int-pow 99 99) -7394533151961528133)
```

#### `(int-signum n)`

Returns `-1` if the int `n` is less than `0`, `0` if `n` is equal to `0`, `1` if `n` is greater than `0`.

```pavo
(assert-eq (int-signum -42) -1)
(assert-eq (int-signum 0) 0)
(assert-eq (int-signum 42) 1)
```

### Bytes

#### `(bytes-count b)`

Returns the number of bytes in the bytes `b`.

Time: O(1).

```pavo
(assert-eq (bytes-count @[]) 0)
(assert-eq (bytes-count @[0]) 1)
(assert-eq (bytes-count @[0, 1, 2]) 3)
```

#### `(bytes-get b index)` `(bytes-get b index default)`

Returns the byte at the int `index` in the bytes `b`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-get @[42] 0) 42)
(assert-throw (bytes-get [] 0) { :tag :err-lookup, :got 0})
(assert-eq (bytes-get [] 0 nil) nil)
```

#### `(bytes-insert b index new)` `(bytes-insert b index new default)`

Inserts the byte `new` into the bytes `b` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-not-byte, :got new}` if `new` is not a byte (an int between 0 and 255 inclusive).
Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-insert @[0 1] 0 42) [42 0 1])
(assert-eq (bytes-insert @[0 1] 1 42) [0 42 1])
(assert-eq (bytes-insert @[0 1] 2 42) [0 1 42])
(assert-throw (bytes-insert @[0 1] 3 42) { :tag :err-lookup, :got 3})
(assert-eq (bytes-insert @[0 1] 3 42 nil) nil)
(assert-throw (bytes-insert @[] 0 256) { :tag :err-not-byte, :got 256})
(assert-throw (bytes-insert @[] 0 256 nil) { :tag :err-not-byte, :got 256})
```

#### `(bytes-remove b index)` `(bytes-remove b index default)`

Returns the bytes obtained by removing the byte at the index int `index` from the bytes `b`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-remove @[0 1] 0) @[1])
(assert-eq (bytes-remove @[0 1] 1) @[0])
(assert-throw (bytes-remove @[0 1] 3) { :tag :err-lookup, :got 3})
(assert-eq (bytes-remove @[0 1] 3 nil) nil)
```

#### `(bytes-update b index new)` `(bytes-update b index new default)`

Returns the bytes obtained by replacing the byte at the index int `index` in the bytes `b` with the byte `new`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-not-byte, :got new}` if `new` is not a byte (an int between 0 and 255 inclusive).

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-update @[0 1] 0 42) @[42 1])
(assert-eq (bytes-update @[0 1] 1 42) @[0 42])
(assert-throw (bytes-update @[0 1] 2 42) { :tag :err-lookup, :got 2})
(assert-eq (bytes-update @[0 1] 2 42 nil) nil)
(assert-throw (bytes-update @[] 0 256) { :tag :err-not-byte, :got 256})
(assert-throw (bytes-update @[] 0 256 nil) { :tag :err-not-byte, :got 256})
```

#### `(bytes-slice b start end)` `(bytes-slice b start end default)`

Returns a subsequence of the bytes `b`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-slice @[42 43] 1 1) [])
(assert-eq (bytes-slice @[42 43] 0 1) [42])
(assert-eq (bytes-slice @[42 43] 1 2) [43])
(assert-eq (bytes-slice @[42 43] 0 2) [42 43])
(assert-throw (bytes-slice @[] 0 1) { :tag :err-lookup, :got 1})
(assert-throw (bytes-slice @[] 2 3) { :tag :err-lookup, :got 2})
(assert-throw (bytes-slice @[0 1 2 3] 2 1) { :tag :err-lookup, :got 1})
(assert-eq (bytes-slice @[] 0 1 nil) nil)
```

#### `(bytes-splice old index new)` `(bytes-splice old index new default)`

Inserts the elements of the bytes `new` into the bytes `old`, starting at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds (of the `old` bytes).
Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log (n + m)), where n is `(bytes-count old)` and m is `(bytes-count new)`.

```pavo
(assert-eq (bytes-splice @[0 1] @[10 11] 0) @[10 11 0 1])
(assert-eq (bytes-splice @[0 1] @[10 11] 1) @[0 10 11 1])
(assert-eq (bytes-splice @[0 1] @[10 11] 2) @[0 1 10 11])
(assert-throw (bytes-splice @[0 1] @[10 11] 3) { :tag :err-lookup, :got 3})
(assert-eq (bytes-splice @[0 1] @[10 11] 3 nil) nil)
```

#### `(bytes-concat left right)`

Returns a bytes that contains all elements of the bytes `left` followed by all elements of the bytes `right`.

Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(bytes-count left)` and m is `(bytes-count right)`.

```pavo
(assert-eq (bytes-concat @[0 1] @[2 3]) @[0 1 2 3])
(assert-eq (bytes-concat @[] @[0 1]) @[0 1])
(assert-eq (bytes-concat @[0 1] @[]) @[0 1])
```

#### `(bytes-iter b fun)`

Starting from the beginning of the bytes `b`, applies the function `fun` to the elements of `b` in sequence until either `fun` returns a truthy value or the end of the bytes is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(bytes-count b)`.

```pavo
(let :mut product 1 (do
    (bytes-iter @[1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (bytes-iter @[1 2 3 4] (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 2)
))
```

#### `(bytes-iter-back b fun)`

Starting from the back of the bytes `b`, applies the function `fun` to the elements of `b` in reverse order until either `fun` returns a truthy value or the end of the bytes is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(bytes-count b)`.

```pavo
(let :mut product 1 (do
    (bytes-iter-back @[1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (bytes-iter-back @[1 2 3 4] (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 4)
))
```

#### `(bytes-push-front b new)`

Returns the bytes obtained by inserting the byte `new` at the front of the bytes `b`.

Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.
Throws `{ :tag :err-not-byte, :got new}` if `new` is not a byte (an int between 0 and 255 inclusive).

Time: Amortized O(1) (amortized across `bytes-push-front` and `bytes-pop-front` applications)

```pavo
(assert-eq (bytes-push-front @[] 42) @[42])
(assert-eq (bytes-push-front @[43] 42) @[42 43])
```

#### `(bytes-front b)` `(bytes-front b default)`

Returns the first byte in the bytes `b`.

Throws `{ :tag :err-collection-empty }` if the bytes are empty.

If `default` is supplied, returns `default` instead of throwing.

Time: O(1)

```pavo
(assert-eq (bytes-front @[41 42]) 41)
(assert-eq (bytes-front @[41]) 41)
(assert-throw (bytes-front @[]) { :tag :err-collection-empty})
(assert-eq (bytes-front @[] nil) nil)
```

#### `(bytes-pop-front b)` `(bytes-pop-front b default)`

Returns all but the first bytes in the bytes `b`.

Throws `{ :tag :err-collection-empty }` if the bytes are empty.

If `default` is supplied, returns `default` instead of throwing.

Time: Amortized O(1) (amortized across `bytes-push-front` and `bytes-pop-front` applications)

```pavo
(assert-eq (bytes-pop-front @[41 42]) @[42])
(assert-eq (bytes-pop-front @[41]) @[])
(assert-throw (bytes-pop-front @[]) { :tag :err-collection-empty})
(assert-eq (bytes-pop-front @[] nil) nil)
```

#### `(bytes-push-back b new)`

Returns the bytes obtained by inserting the byte `new` at the back of the bytes `b`.

Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.
Throws `{ :tag :err-not-byte, :got new}` if `new` is not a byte (an int between 0 and 255 inclusive).

Time: Amortized O(1) (amortized across `bytes-push-back` and `bytes-pop-back` applications)

```pavo
(assert-eq (bytes-push-back @[] 41) @[41])
(assert-eq (bytes-push-back @[42] 41) @[42 41])
```

#### `(bytes-back b)` `(bytes-back b default)`

Returns the last byte in the bytes `b`.

Throws `{ :tag :err-collection-empty }` if the bytes is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: O(1)

```pavo
(assert-eq (bytes-back @[41 42]) 42)
(assert-eq (bytes-back @[41]) 41)
(assert-throw (bytes-back @[]) { :tag :err-collection-empty})
(assert-eq (bytes-back @[] nil) nil)
```

#### `(bytes-pop-back b)` `(bytes-pop-back b default)`

Returns all but the last bytes in the bytes `b`.

Throws `{ :tag :err-collection-empty }` if the bytes is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: Amortized O(1) (amortized across `bytes-push-back` and `bytes-pop-back` applications)

```pavo
(assert-eq (bytes-pop-back @[41 42]) @[41])
(assert-eq (bytes-pop-back @[41]) @[])
(assert-throw (bytes-pop-back @[]) { :tag :err-collection-empty})
(assert-eq (bytes-pop-back @[] nil) nil)
```

### Chars

#### `char-max`

The largest char (numerically the largest unicode scalar value).

```pavo
(assert-eq char-max '\{10ffff}')
```

#### `(int=>char n)` `(int=>char n default)`

Returns the unicode scalar value denoted by the int `n`.

Throws `{ :tag :err-not-unicode-scalar, :got n}` if `n` is `n` is not a unicode scalar value.

If `default` is supplied, returns `default` instead of throwing.

```pavo
(assert-eq (int=>char 0x41) 'A')
(assert-throw (int=>char 0x110000) { :tag :err-not-unicode-scalar, :got 0x110000})
(assert-eq (int=>char 0x110000 nil) nil)
```

#### `(char->int c)`

Returns the unicode scalar value of the char `c` as an int.

```pavo
(assert-eq (char->int 'A') 0x41)
```

### Strings

#### `(str-count s)`

Returns the number of chars in the string `s`.

Time: O(1).

```pavo
(assert-eq (str-count "") 0)
(assert-eq (str-count "a") 1)
(assert-eq (str-count "⚗") 1)
(assert-eq (str-count "abc") 3)
```

#### `(str-get s index)` `(str-get s index default)`

Returns the char at the int `index` in the string `s`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-get "a" 0) 'a')
(assert-eq (str-get "⚗b" 1) 'b')
(assert-throw (str-get "" 0) { :tag :err-lookup, :got 0})
(assert-eq (str-get "" 0 nil) nil)
```

#### `(str-insert s index c)` `(str-insert s index cdefault)`

Inserts the char `c` into the string `s` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-collection-full }` if the resulting string would contain 2^63 or more elements.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-insert "ab" 0 'z') "zab")
(assert-eq (str-insert "ab" 1 'z') "azb")
(assert-eq (str-insert "ab" 2 'z') "abz")
(assert-throw (str-insert "ab" 3 'z') { :tag :err-lookup, :got 3})
(assert-eq (str-insert "ab" 3 'z' nil) nil)
```

#### `(str-remove s index)` `(str-remove s index default)`

Returns the string obtained by removing the char at the index int `index` from the string `s`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-remove "ab" 0) "b")
(assert-eq (str-remove "ab" 1) "a")
(assert-throw (str-remove "ab" 3) { :tag :err-lookup, :got 3})
(assert-eq (str-remove "ab" 3 nil) nil)
```

#### `(str-update s index c)` `(str-update s index c default)`

Returns the string obtained by replacing the char at the index int `index` in the string `s` with the char `c`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-update "ab" 0 'z') "zb")
(assert-eq (str-update "ab" 1 'z') "az")
(assert-throw (str-update "ab" 2 'z') { :tag :err-lookup, :got 2})
(assert-eq (str-update "ab" 2 'z' nil) nil)
```

#### `(str-slice s start end)` `(str-slice s start end default)`

Returns a substring of the string `b`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-slice "ab" 1 1) "")
(assert-eq (str-slice "ab" 0 1) "a")
(assert-eq (str-slice "ab" 1 2) "b")
(assert-eq (str-slice "ab" 0 2) "ab")
(assert-throw (str-slice "" 0 1) { :tag :err-lookup, :got 1})
(assert-throw (str-slice "" 2 3) { :tag :err-lookup, :got 2})
(assert-throw (str-slice "abcd" 2 1) { :tag :err-lookup, :got 1})
(assert-eq (str-slice "" 0 1 nil) nil)
```

#### `(str-splice old index new)` `(str-splice old index new default)`

Inserts the string `new` into the string `old`, starting at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds (of the `old` bytes).
Throws `{ :tag :err-collection-full }` if the resulting string would contain 2^63 or more elements.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log (n + m)), where n is `(str-count old)` and m is `(str-count new)`.

```pavo
(assert-eq (str-splice "ab" "cd" 0) "cdab")
(assert-eq (str-splice "ab" "cd" 1) "acdb")
(assert-eq (str-splice "ab" "cd" 2) "abcd")
(assert-throw (str-splice "ab" "cd" 3) { :tag :err-lookup, :got 3})
(assert-eq (str-splice "ab" "cd" 3 nil) nil)
```

#### `(str-concat left right)`

Returns a string that contains all chars of the string `left` followed by all chars of the string `right`.

Throws `{ :tag :err-collection-full }` if the resulting string would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(str-count left)` and m is `(str-count right)`.

```pavo
(assert-eq (str-concat "ab" "cd") "abcd")
(assert-eq (str-concat "" "cd") "cd")
(assert-eq (str-concat "ab" "") "ab")
```

#### `(str-iter s fun)`

Starting from the beginning of the string `s`, applies the function `fun` to the chars of `s` in sequence until either `fun` returns a truthy value or the end of the string is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(str-count s)`.

```pavo
(let :mut out "z" (do
    (str-iter "abcd" (fn [elem] (set! out (str-push-back out elem))))
    (assert-eq out "zabcd")
))
(let :mut out "z" (do
    (str-iter "abcd" (fn [elem] (if
            (= elem 'c') true
            (set! out (str-push-back out elem))
        )))
    (assert-eq out "zab")
))
```

#### `(str-iter-back s fun)`

Starting from the back of the string `s`, applies the function `fun` to the chars of `s` in reverse order until either `fun` returns a truthy value or the end of the string is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(str-count s)`.

```pavo
(let :mut out "z" (do
    (str-iter-back "abcd" (fn [elem] (set! out (str-push-back out elem))))
    (assert-eq out "zdcba")
))
(let :mut out "z" (do
    (str-iter-back "abcd" (fn [elem] (if
            (= elem 'c') true
            (set! out (str-push-back out elem))
        )))
    (assert-eq out "zd")
))
```

#### `(str-push-front s c)`

Returns the string obtained by inserting the char `c` at the front of the string `s`.

Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.

Time: Amortized O(1) (amortized across `str-push-front` and `str-pop-front` applications)

```pavo
(assert-eq (str-push-front "" 'a') "a")
(assert-eq (str-push-front "b" 'a') "ab")
```

#### `(str-front s)` `(str-front s default)`

Returns the first char in the string `s`.

Throws `{ :tag :err-collection-empty }` if the string is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: O(1)

```pavo
(assert-eq (str-front "ab") 'a')
(assert-eq (str-front "a") 'a')
(assert-throw (str-front "") { :tag :err-collection-empty})
(assert-eq (str-front "" nil) nil)
```

#### `(str-pop-front s)` `(str-pop-front s default)`

Returns a string consisting of all but the first char in the string `s`.

Throws `{ :tag :err-collection-empty }` if the string is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: Amortized O(1) (amortized across `str-push-front` and `str-pop-front` applications)

```pavo
(assert-eq (str-pop-front "ab") "a")
(assert-eq (str-pop-front "a") "")
(assert-throw (str-pop-front "") { :tag :err-collection-empty})
(assert-eq (str-pop-front "" nil) nil)
```

#### `(str-push-back s c)`

Returns the string obtained by inserting the char `c` at the back of the string `s`.

Throws `{ :tag :err-collection-full }` if the resulting string would contain 2^63 or more elements.

Time: Amortized O(1) (amortized across `str-push-back` and `str-pop-back` applications)

```pavo
(assert-eq (str-push-back "" 'a') "a")
(assert-eq (str-push-back "b" 'a') "ba")
```

#### `(str-back s)` `(str-back s default)`

Returns the last char in the string `s`.

Throws `{ :tag :err-collection-empty }` if the string is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: O(1)

```pavo
(assert-eq (str-back "ab") 'b')
(assert-eq (str-back "a") 'a')
(assert-throw (str-back "") { :tag :err-collection-empty})
(assert-eq (str-back "" nil) nil)
```

#### `(str-pop-back s)` `(str-pop-back s default)`

Returns the string consisting of all but the last char in the string `s`.

Throws `{ :tag :err-collection-empty }` if the string is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: Amortized O(1) (amortized across `str-push-back` and `str-pop-back` applications)

```pavo
(assert-eq (str-pop-back "ab") "a")
(assert-eq (str-pop-back "a") "")
(assert-throw (str-pop-back "") { :tag :err-collection-empty})
(assert-eq (str-pop-back "" nil) nil)
```

### Floats

### Arrays

#### `(arr-count arr)`

Returns the number of elements in the array `arr`.

Time: O(1).

```pavo
(assert-eq (arr-count []) 0)
(assert-eq (arr-count [nil]) 1)
(assert-eq (arr-count [0, 1, 2]) 3)
```

#### `(arr-get arr index)` `(arr-get arr index default)`

Returns the element at the int `index` in the array `arr`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-get [true] 0) true)
(assert-throw (arr-get [] 0) { :tag :err-lookup, :got 0})
(assert-eq (arr-get [] 0 nil) nil)
```

#### `(arr-insert arr index new)` `(arr-insert arr index new default)`

Inserts the value `new` into the array `arr` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-collection-full }` if the resulting array would contain 2^63 or more elements.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-insert [0 1] 0 42) [42 0 1])
(assert-eq (arr-insert [0 1] 1 42) [0 42 1])
(assert-eq (arr-insert [0 1] 2 42) [0 1 42])
(assert-throw (arr-insert [0 1] 3 42) { :tag :err-lookup, :got 3})
(assert-eq (arr-insert [0 1] 3 42 nil) nil)
```

#### `(arr-remove arr index)` `(arr-remove arr index default)`

Returns the array obtained by removing the element at the index int `index` from the array `arr`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-remove [0 1] 0) [1])
(assert-eq (arr-remove [0 1] 1) [0])
(assert-throw (arr-remove [0 1] 3) { :tag :err-lookup, :got 3})
(assert-eq (arr-remove [0 1] 3 nil) nil)
```

#### `(arr-update arr index new)` `(arr-update arr index new default)`

Returns the array obtained by replacing the element at the index int `index` in the array `arr` with the value `new`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-update [0 1] 0 42) [42 1])
(assert-eq (arr-update [0 1] 1 42) [0 42])
(assert-throw (arr-update [0 1] 2 42) { :tag :err-lookup, :got 2})
(assert-eq (arr-update [0 1] 2 42 nil) nil)
```

#### `(arr-slice arr start end)` `(arr-slice arr start end default)`

Returns an array containing a subsequence of the elements of the array `arr`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-slice [true false] 1 1) [])
(assert-eq (arr-slice [true false] 0 1) [true])
(assert-eq (arr-slice [true false] 1 2) [false])
(assert-eq (arr-slice [true false] 0 2) [true false])
(assert-throw (arr-slice [] 0 1) { :tag :err-lookup, :got 1})
(assert-throw (arr-slice [] 2 3) { :tag :err-lookup, :got 2})
(assert-throw (arr-slice [0 1 2 3] 2 1) { :tag :err-lookup, :got 1})
(assert-eq (arr-slice [] 0 1 nil) nil)
```

#### `(arr-splice old index new)` `(arr-splice old index new default)`

Inserts the elements of the array `new` into the array `old`, starting at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds (of the `old` array).
Throws `{ :tag :err-collection-full }` if the resulting array would contain 2^63 or more elements.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log (n + m)), where n is `(arr-count old)` and m is `(arr-count new)`.

```pavo
(assert-eq (arr-splice [0 1] [10 11] 0) [10 11 0 1])
(assert-eq (arr-splice [0 1] [10 11] 1) [0 10 11 1])
(assert-eq (arr-splice [0 1] [10 11] 2) [0 1 10 11])
(assert-throw (arr-splice [0 1] [10 11] 3) { :tag :err-lookup, :got 3})
(assert-eq (arr-splice [0 1] [10 11] 3 nil) nil)
```

#### `(arr-concat left right)`

Returns an array that contains all elements of the array `left` followed by all elements of the array `right`.

Throws `{ :tag :err-collection-full }` if the resulting array would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(arr-count left)` and m is `(arr-count right)`.

```pavo
(assert-eq (arr-concat [0 1] [2 3]) [0 1 2 3])
(assert-eq (arr-concat [] [0 1]) [0 1])
(assert-eq (arr-concat [0 1] []) [0 1])
```

#### `(arr-iter arr fun)`

Starting from the beginning of the array `arr`, applies the function `fun` to the elements of `arr` in sequence until either `fun` returns a truthy value or the end of the array is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(arr-count arr)`.

```pavo
(let :mut product 1 (do
    (arr-iter [1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (arr-iter [1 2 3 4] (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 2)
))
```

#### `(arr-iter-back arr fun)`

Starting from the back of the array `arr`, applies the function `fun` to the elements of `arr` in reverse order until either `fun` returns a truthy value or the end of the array is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(arr-count arr)`.

```pavo
(let :mut product 1 (do
    (arr-iter-back [1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (arr-iter-back [1 2 3 4] (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 4)
))
```

#### `(arr-push-front arr new)`

Returns the array obtained by inserting the value `new` at the front of the array `arr`.

Throws `{ :tag :err-collection-full }` if the resulting array would contain 2^63 or more elements.

Time: Amortized O(1) (amortized across `arr-push-front` and `arr-pop-front` applications)

```pavo
(assert-eq (arr-push-front [] true) [true])
(assert-eq (arr-push-front [false] true) [true false])
```

#### `(arr-front arr)` `(arr-front arr default)`

Returns the first element in array `arr`.

Throws `{ :tag :err-collection-empty }` if the array is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: O(1)

```pavo
(assert-eq (arr-front [true false]) true)
(assert-eq (arr-front [true]) true)
(assert-throw (arr-front []) { :tag :err-collection-empty})
(assert-eq (arr-front [] nil) nil)
```

#### `(arr-pop-front arr)` `(arr-pop-front arr default)`

Returns all but the first elements in array `arr`.

Throws `{ :tag :err-collection-empty }` if the array is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: Amortized O(1) (amortized across `arr-push-front` and `arr-pop-front` applications)

```pavo
(assert-eq (arr-pop-front [true false]) [false])
(assert-eq (arr-pop-front [true]) [])
(assert-throw (arr-pop-front []) { :tag :err-collection-empty})
(assert-eq (arr-pop-front [] nil) nil)
```

#### `(arr-push-back arr new)`

Returns the array obtained by inserting the value `new` at the back of the array `arr`.

Throws `{ :tag :err-collection-full }` if the resulting array would contain 2^63 or more elements.

Time: Amortized O(1) (amortized across `arr-push-back` and `arr-pop-back` applications)

```pavo
(assert-eq (arr-push-back [] true) [true])
(assert-eq (arr-push-back [false] true) [false true])
```

#### `(arr-back arr)` `(arr-back arr default)`

Returns the last element in array `arr`.

Throws `{ :tag :err-collection-empty }` if the array is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: O(1)

```pavo
(assert-eq (arr-back [true false]) false)
(assert-eq (arr-back [true]) true)
(assert-throw (arr-back []) { :tag :err-collection-empty})
(assert-eq (arr-back [] nil) nil)
```

#### `(arr-pop-back arr)` `(arr-pop-back arr default)`

Returns all but the last elements in array `arr`.

Throws `{ :tag :err-collection-empty }` if the array is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: Amortized O(1) (amortized across `arr-push-back` and `arr-pop-back` applications)

```pavo
(assert-eq (arr-pop-back [true false]) [true])
(assert-eq (arr-pop-back [true]) [])
(assert-throw (arr-pop-back []) { :tag :err-collection-empty})
(assert-eq (arr-pop-back [] nil) nil)
```

### Applications

#### `(app-count app)`

Returns the number of elements in the application `app`.

Time: O(1).

```pavo
(assert-eq (app-count $()) 0)
(assert-eq (app-count $(nil)) 1)
(assert-eq (app-count $(0, 1, 2)) 3)
```

#### `(app-get app index)` `(app-get app index default)`

Returns the element at the int `index` in the application `app`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-get $(true) 0) true)
(assert-throw (app-get $() 0) { :tag :err-lookup, :got 0})
(assert-eq (app-get $() 0 nil) nil)
```

#### `(app-insert app index new)` `(app-insert app index new default)`

Inserts the value `new` into the application `app` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-collection-full }` if the resulting application would contain 2^63 or more elements.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-insert $(0 1) 0 42) $(42 0 1))
(assert-eq (app-insert $(0 1) 1 42) $(0 42 1))
(assert-eq (app-insert $(0 1) 2 42) $(0 1 42))
(assert-throw (app-insert $(0 1) 3 42) { :tag :err-lookup, :got 3})
(assert-eq (app-insert $(0 1) 3 42 nil) nil)
```

#### `(app-remove app index)` `(app-remove app index default)`

Returns the application obtained by removing the element at the index int `index` from the application `app`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-remove $(0 1) 0) $(1))
(assert-eq (app-remove $(0 1) 1) $(0))
(assert-throw (app-remove $(0 1) 3) { :tag :err-lookup, :got 3})
(assert-eq (app-remove $(0 1) 3 nil) nil)
```

#### `(app-update app index new)` `(app-update app index new default)`

Returns the application obtained by replacing the element at the index int `index` in the application `app` with the value `new`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-update $(0 1) 0 42) $(42 1))
(assert-eq (app-update $(0 1) 1 42) $(0 42))
(assert-throw (app-update $(0 1) 2 42) { :tag :err-lookup, :got 2})
(assert-eq (app-update $(0 1) 2 42 nil) nil)
```

#### `(app-slice app start end)` `(app-slice app start end default)`

Returns an application containing a subsequence of the elements of the application `app`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-slice $(true false) 1 1) $())
(assert-eq (app-slice $(true false) 0 1) $(true))
(assert-eq (app-slice $(true false) 1 2) $(false))
(assert-eq (app-slice $(true false) 0 2) $(true false))
(assert-throw (app-slice $() 0 1) { :tag :err-lookup, :got 1})
(assert-throw (app-slice $() 2 3) { :tag :err-lookup, :got 2})
(assert-throw (app-slice $(0 1 2 3) 2 1) { :tag :err-lookup, :got 1})
(assert-eq (app-slice $() 0 1 nil) nil)
```

#### `(app-splice old index new)` `(app-splice old index new default)`

Inserts the elements of the application `new` into the application `old`, starting at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds (of the `old` application).
Throws `{ :tag :err-collection-full }` if the resulting application would contain 2^63 or more elements.

If `default` is supplied, returns `default` instead of throwing a lookup error.

Time: O(log (n + m)), where n is `(app-count old)` and m is `(app-count new)`.

```pavo
(assert-eq (app-splice $(0 1) $(10 11) 0) $(10 11 0 1))
(assert-eq (app-splice $(0 1) $(10 11) 1) $(0 10 11 1))
(assert-eq (app-splice $(0 1) $(10 11) 2) $(0 1 10 11))
(assert-throw (app-splice $(0 1) $(10 11) 3) { :tag :err-lookup, :got 3})
(assert-eq (app-splice $(0 1) $(10 11) 3 nil) nil)
```

#### `(app-concat left right)`

Returns an application that contains all elements of the application `left` followed by all elements of the application `right`.

Throws `{ :tag :err-collection-full }` if the resulting application would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(app-count left)` and m is `(app-count right)`.

```pavo
(assert-eq (app-concat $(0 1) $(2 3)) $(0 1 2 3))
(assert-eq (app-concat $() $(0 1)) $(0 1))
(assert-eq (app-concat $(0 1) $()) $(0 1))
```

#### `(app-iter app fun)`

Starting from the beginning of the application `app`, applies the function `fun` to the elements of `app` in sequence until either `fun` returns a truthy value or the end of the application is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(app-count app)`.

```pavo
(let :mut product 1 (do
    (app-iter $(1 2 3 4) (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (app-iter $(1 2 3 4) (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 2)
))
```

#### `(app-iter-back app fun)`

Starting from the back of the application `app`, applies the function `fun` to the elements of `app` in reverse order until either `fun` returns a truthy value or the end of the application is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(app-count app)`.

```pavo
(let :mut product 1 (do
    (app-iter-back $(1 2 3 4) (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (app-iter-back $(1 2 3 4) (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 4)
))
```

#### `(app-push-front app new)`

Returns the application obtained by inserting the value `new` at the front of the application `app`.

Throws `{ :tag :err-collection-full }` if the resulting application would contain 2^63 or more elements.

Time: Amortized O(1) (amortized across `app-push-front` and `app-pop-front` applications)

```pavo
(assert-eq (app-push-front $() true) $(true))
(assert-eq (app-push-front $(false) true) $(true false))
```

#### `(app-front app)` `(app-front app default)`

Returns the first element in application `app`.

Throws `{ :tag :err-collection-empty }` if the application is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: O(1)

```pavo
(assert-eq (app-front $(true false)) true)
(assert-eq (app-front $(true)) true)
(assert-throw (app-front $()) { :tag :err-collection-empty})
(assert-eq (app-front $() nil) nil)
```

#### `(app-pop-front app)` `(app-pop-front app default)`

Returns all but the first elements in application `app`.

Throws `{ :tag :err-collection-empty }` if the application is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: Amortized O(1) (amortized across `app-push-front` and `app-pop-front` applications)

```pavo
(assert-eq (app-pop-front $(true false)) $(false))
(assert-eq (app-pop-front $(true)) $())
(assert-throw (app-pop-front $()) { :tag :err-collection-empty})
(assert-eq (app-pop-front $() nil) nil)
```

#### `(app-push-back app new)`

Returns the application obtained by inserting the value `new` at the back of the application `app`.

Throws `{ :tag :err-collection-full }` if the resulting application would contain 2^63 or more elements.

Time: Amortized O(1) (amortized across `app-push-back` and `app-pop-back` applications)

```pavo
(assert-eq (app-push-back $() true) $(true))
(assert-eq (app-push-back $(false) true) $(false true))
```

#### `(app-back app)` `(app-back app default)`

Returns the last element in application `app`.

Throws `{ :tag :err-collection-empty }` if the application is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: O(1)

```pavo
(assert-eq (app-back $(true false)) false)
(assert-eq (app-back $(true)) true)
(assert-throw (app-back $()) { :tag :err-collection-empty})
(assert-eq (app-back $() nil) nil)
```

#### `(app-pop-back app)` `(app-pop-back app default)`

Returns all but the last elements in application `app`.

Throws `{ :tag :err-collection-empty }` if the application is empty.

If `default` is supplied, returns `default` instead of throwing.

Time: Amortized O(1) (amortized across `app-push-back` and `app-pop-back` applications)

```pavo
(assert-eq (app-pop-back $(true false)) $(true))
(assert-eq (app-pop-back $(true)) $())
(assert-throw (app-pop-back $()) { :tag :err-collection-empty})
(assert-eq (app-pop-back $() nil) nil)
```

### Sets

#### `(set-count set)`

Returns the number of elements in the set `set`.

Time: O(1).

```pavo
(assert-eq (set-count @{}) 0)
(assert-eq (set-count @{nil}) 1)
(assert-eq (set-count @{0, 1, 2}) 3)
```

#### `(set-contains? set elem)`

Returns `true` if the value `elem` is in the set `set`, `false` otherwise.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-contains? @{ nil } nil) true)
(assert-eq (set-contains? @{ 42 } 43) false)
(assert-eq (set-contains? @{} nil) false)
```

#### `(set-min set)` `(set-min set default)`

Returns the minimal element in the set `set`.

Throws `{ :tag :err-collection-empty }` if `set` is the empty set.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-min @{ 4 3 }) 3)
(assert-throw (set-min @{}) { :tag :err-collection-empty })
(assert-eq (set-min @{} nil) nil)
```

#### `(set-max set)` `(set-max set default)`

Returns the maximal element in the set `set`.

Throws `{ :tag :err-collection-empty }` if `set` is the empty set.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-max @{ 4 3 }) 4)
(assert-throw (set-max @{}) { :tag :err-collection-empty })
(assert-eq (set-max @{} nil) nil)
```

#### `(set-insert set new)`

Inserts the value `new` into the set `set`.

Throws `{ :tag :err-collection-full }` if the resulting set would contain 2^63 or more elements.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-insert @{} nil) @{nil})
(assert-eq (set-insert @{nil} nil) @{nil})
```

#### `(set-remove set elem)`

Returns the set obtained by removing the element `elem` from the set `set`.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-remove @{nil} nil) @{})
(assert-eq (set-remove @{} nil) @{})
```

#### `(set-union lhs rhs)`

Returns the set that contains all the elements in the set `lhs` and all the elements in the set `rhs`.

Throws `{ :tag :err-collection-full }` if the resulting set would contain 2^63 or more elements.

```pavo
(assert-eq (set-union @{1 2} @{2 3}) @{1 2 3})
(assert-eq (set-union @{1 2} @{}) @{1 2})
(assert-eq (set-union @{} @{2 3}) @{2 3})
(assert-eq (set-union @{} @{}) @{})
```

Time: O((n + m) log (n + m)), where n is `(set-count left)` and m is `(set-count right)`. TODO this can probably be stricter?

#### `(set-intersection lhs rhs)`

Returns the set that contains all the elements contained in both the set `lhs` and the set `rhs`.

```pavo
(assert-eq (set-intersection @{1 2} @{2 3}) @{2})
(assert-eq (set-intersection @{1 2} @{}) @{})
(assert-eq (set-intersection @{} @{2 3}) @{})
(assert-eq (set-intersection @{} @{}) @{})
```

Time: O((n + m) log (n + m)), where n is `(set-count left)` and m is `(set-count right)`. TODO this can probably be stricter?

#### `(set-difference lhs rhs)`

Returns the set that contains all the elements contained in the set `lhs` but not contained in the set `rhs`.

```pavo
(assert-eq (set-difference @{1 2} @{2 3}) @{1})
(assert-eq (set-difference @{1 2} @{}) @{})
(assert-eq (set-difference @{} @{2 3}) @{})
(assert-eq (set-difference @{} @{}) @{})
```

Time: O((n + m) log (n + m)), where n is `(set-count left)` and m is `(set-count right)`. TODO this can probably be stricter?

#### `(set-symmetric-difference lhs rhs)`

Returns the set that contains all the elements exactly one of the sets `lhs` and `rhs`.

```pavo
(assert-eq (set-symmetric-difference @{1 2} @{2 3}) @{1 3})
(assert-eq (set-symmetric-difference @{1 2} @{}) @{1 2})
(assert-eq (set-symmetric-difference @{} @{2 3}) @{2 3})
(assert-eq (set-symmetric-difference @{} @{}) @{})
```

Time: O((n + m) log (n + m)), where n is `(set-count left)` and m is `(set-count right)`. TODO this can probably be stricter?

#### `(set-iter set fun)`

Starting from the minimal element of the set `set`, applies the function `fun` to the elements of `set` in ascending order until either `fun` returns a truthy value or the end of the set is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(set-count set)`.

```pavo
(let :mut product 1 (do
    (set-iter @{4 2 3 1} (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (set-iter @{4 2 3 1} (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 2)
))
```

#### `(set-iter-back set fun)`

Starting from the maximal element of the set `set`, applies the function `fun` to the elements of `set` in descending order until either `fun` returns a truthy value or the end of the set is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(set-count set)`.

```pavo
(let :mut product 1 (do
    (set-iter-back @{4 2 3 1} (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (set-iter-back @{4 2 3 1} (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 4)
))
```

### Maps

#### `(map-count map)`

Returns the number of entries in the map `map`.

Time: O(1).

```pavo
(assert-eq (map-count {}) 0)
(assert-eq (map-count {{} nil}) 1)
(assert-eq (map-count {0 42, 1 41, 2 40}) 3)
```

#### `(map-get map key)` `(map-get map key default)`

Returns the value associated with the key `key` in the map `map`.

Throws `{ :tag :err-lookup, :got key}` if the map contains no entry with this key.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-get {0 42} 0) 42)
(assert-throw (map-get {} 0) { :tag :err-lookup, :got 0})
(assert-eq (map-get {} 0 nil) nil)
```

#### `(map-contains? map key)`

Returns `true` if the map `map` contains an entry with key `key`, `false` otherwise.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-contains? { nil 0 } nil) true)
(assert-eq (map-contains? { 42 0 } 43) false)
(assert-eq (map-contains? {} nil) false)
```

#### `(map-min map)` `(map-min map default)`

Returns the value of the entry with the minimal key in the map `map`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min @{0 42, 1 41}) 42)
(assert-throw (map-min {}) { :tag :err-collection-empty })
(assert-eq (map-min {} nil) nil)
```

#### `(map-min-key map)` `(map-min-key map default)`

Returns the minimal key in the map `map`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min-key @{0 42, 1 41}) 0)
(assert-throw (map-min-key {}) { :tag :err-collection-empty })
(assert-eq (map-min-key {} nil) nil)
```

#### `(map-min-entry map)` `(map-min-entry map default)`

Returns the entry with the minimal key in the map `map`, as an array `[key value]`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min-entry @{0 42, 1 41}) [0 42])
(assert-throw (map-min-entry {}) { :tag :err-collection-empty })
(assert-eq (map-min-entry {} nil) nil)
```

#### `(map-max map)` `(map-max map default)`

Returns the value of the entry with the maximal key in the map `map`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max @{0 42, 1 41}) 41)
(assert-throw (map-max {}) { :tag :err-collection-empty })
(assert-eq (map-max {} nil) nil)
```

#### `(map-max-key map)` `(map-max-key map default)`

Returns the maximal key in the map `map`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max-key @{0 42, 1 41}) 1)
(assert-throw (map-max-key {}) { :tag :err-collection-empty })
(assert-eq (map-max-key {} nil) nil)
```

#### `(map-max-entry map)` `(map-max-entry map default)`

Returns the entry with the maximal key in the map `map`, as an array `[key value]`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

If `default` is supplied, returns `default` instead of throwing.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max-entry @{0 42, 1 41}) [1 41])
(assert-throw (map-max-entry {}) { :tag :err-collection-empty })
(assert-eq (map-max-entry {} nil) nil)
```

#### `(map-insert map key value)`

Inserts the entry `key`, `value` into the map `map`, potentially overwriting an older entry.

Throws `{ :tag :err-collection-full }` if the resulting map would contain 2^63 or more entries.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-insert {} 0 42) {0 42})
(assert-eq (map-insert {0 42} 0 43) {0 43})
```

#### `(map-remove map key)`

Returns the map obtained by removing the entry (if any) with the key `key` from the map `map`.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (set-remove {0 42} 0) {})
(assert-eq (set-remove {} 0) {})
```

#### `(map-union lhs rhs)`

Returns the map that contains all the entries in the map `lhs` and all the entries in the map `rhs`. For entries whose keys are contained in both maps, the value from the lhs map is used.

Throws `{ :tag :err-collection-full }` if the resulting map would contain 2^63 or more elements.

```pavo
(assert-eq (map-union {0 42, 1 41} {1 17, 2 40}) {0 42, 1 41, 2 40})
(assert-eq (map-union {0 42, 1 41} {}) {0 42, 1 41})
(assert-eq (map-union {} {1 41, 2 40}) {1 41, 2 40})
(assert-eq (map-union {} {}) {})
```

Time: O((n + m) log (n + m)), where n is `(set-count left)` and m is `(set-count right)`. TODO this can probably be stricter?

#### `(map-intersection lhs rhs)`

Returns the map that contains all the entries in the map `lhs` whose key is also the key of an entry in the map `rhs`.

```pavo
(assert-eq (map-intersection {0 42, 1 41} {1 17, 2 40}) {1 41})
(assert-eq (map-intersection {0 42, 1 41} {}) {})
(assert-eq (map-intersection {} {1 41, 2 40}) {})
(assert-eq (map-intersection {} {}) {})
```

Time: O((n + m) log (n + m)), where n is `(set-count left)` and m is `(set-count right)`. TODO this can probably be stricter?

#### `(map-difference lhs rhs)`

Returns the map that contains all the entries in the map `lhs` whose key is not the key of an entry in the map `rhs`.

```pavo
(assert-eq (map-difference {0 42, 1 41} {1 17, 2 40}) {0 42})
(assert-eq (map-difference {0 42, 1 41} {}) {0 42, 1 41})
(assert-eq (map-difference {} {1 41, 2 40}) {})
(assert-eq (map-difference {} {}) {})
```

Time: O((n + m) log (n + m)), where n is `(set-count left)` and m is `(set-count right)`. TODO this can probably be stricter?

#### `(map-symmetric-difference lhs rhs)`

Returns the map that contains all the entries in the maps `lhs` and `rhs` whose key does not occur in both maps.

```pavo
(assert-eq (map-symmetric-difference {0 42, 1 41} {1 17, 2 40}) {0 42, 2 40})
(assert-eq (map-symmetric-difference {0 42, 1 41} {}) {0 42, 1 41})
(assert-eq (map-symmetric-difference {} {1 41, 2 40}) {1 41, 2 40})
(assert-eq (map-symmetric-difference {} {}) {})
```

Time: O((n + m) log (n + m)), where n is `(set-count left)` and m is `(set-count right)`. TODO this can probably be stricter?

#### `(set-iter set fun)`

Starting from the entry with the minimal key in the map `map`, applies the function `fun` to the entries of `map` in ascending order until either `fun` returns a truthy value or the end of the set is reached. Returns `nil`. Propagates any value thrown by `fun`.

Fun is passed the key first and the value second.

Time: Iteration takes amortized O(n), where n is `(map-count map)`.

```pavo
(let :mut product 1 (do
    (map-iter {4 2, 3 1} (fn [key value] (set! product (int-mul product (int-mul key value)))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (map-iter {4 2, 3 1} (fn [key value] (if
            (= key 3) true
            (set! product (int-mul product (int-mul key value)))
        )))
    (assert-eq product 3)
))
```

#### `(map-iter-back set fun)`

Starting from the entry with the maximal key in the map `map`, applies the function `fun` to the entries of `map` in descending order until either `fun` returns a truthy value or the end of the set is reached. Returns `nil`. Propagates any value thrown by `fun`.

Fun is passed the key first and the value second.

Time: Iteration takes amortized O(n), where n is `(map-count map)`.

```pavo
(let :mut product 1 (do
    (map-iter-back {4 2, 3 1} (fn [key value] (set! product (int-mul product (int-mul key value)))))
    (assert-eq product 24)
))
(let :mut product 1 (do
    (map-iter-back {4 2, 3 1} (fn [key value] (if
            (= key 3) true
            (set! product (int-mul product (int-mul key value)))
        )))
    (assert-eq product 8)
))
```

### Symbols

#### `(symbol)`

Returns a fresh symbol that is only equal to itself.

```pavo
(assert (let x (symbol) (= x x)))
(assert-not (= (symbol) (symbol)))
```

#### Equality and Ordering
TODO introduction, explain equality and the total order over all values, talk about determinism

#### `(= x y)`

Returns `true` if `x` and `y` are equal, `false` otherwise. For two values to be equal, they must have the same type. The specific semantics vary by type:

- atomic values (`nil`, bools, ints, floats, chars, strings, bytes, keywords and identifiers) are equal iff they denote the same value.
- identifiers are equal iff they consist of the same characters
  - in particular, identifier equality does not reflect bindings, scoping or macro expansion phases
- arrays are equal iff they have the same length and for all indexes i the i-th entry of both arrays is equal.
- applications are equal iff they have the same length and for all indexes i the i-th entry of both arrays is equal.
- maps are equal iff they have the same length, the smalles key of both maps is equal, the value associated with the smallest key of both maps is equal, and the maps without their entries with the smalles key are equal.
- sets are equal iff they have the same length, the smallest element of both sets is equal, and the sets without their smallest elements are equal.
- symbols are only equal to themselves
- functions are only equal to themselves

`=` is an [equivalence relation](https://en.wikipedia.org/wiki/Equivalence_relation).

```pavo
(assert (= nil nil))
(assert-not (= nil false))
(assert-not (= 0 1))
(assert (= 0 -0))
(assert (= 17 0x10))
(assert (= 0.0 -0.0))
(assert (= 0.30000000000000004 0.30000000000000005)) # different float literals can round to equal values
(assert (= [[]] [[]]))
(assert-not (= [1] [2]))
(assert (= `(()) `(())))
(assert-not (= [] `()))
(assert (= {1 2} {1 2}))
(assert (= @{[]} @{[]}))
(assert-not (= (fn [x] x) (fn [x] x)))
(assert (= = =))
(assert-not (= (symbol) (symbol)))
(assert (let x (symbol) (= x x)))
```

#### `(< x y)`

Returns `true` if x is less than y, `false` otherwise. This is *not* just constrained to numbers, there is a [total order](https://en.wikipedia.org/wiki/Total_order) among *all* pavo values. It is possible to compare values of different types.

The total order is defined as follows: TODO

`<` is a [strict total order](https://en.wikipedia.org/wiki/Total_order#Strict_total_order).

```pavo
(assert (< 0 1))
(assert (< false true))
(assert (< true 0))
(assert (< 42 0.1)) # ints are less than floats in the order over all values
```

#### `(<= x y)`
TODO

#### `(> x y)`
TODO

#### `(>= x y)`
TODO

### Code as Data

#### (read s)

If the string `s` is a pavo expression, returns the value denoted by that expression.

Throws `{ :tag :err-not-expression }` if the string is not a valid pavo expression.

Time: O(n), where n is `(string-count <prefix>)`, where `<prefix>` is the longest prefix of `s` that is a pavo expression.

```pavo
(assert-eq (read "42") 42)
(assert-eq (read "(a) ") $(a))
(assert-throw (read "(a) b") { :tag :err-not-expression })
```

#### (read-prefix)

If a prefix of the string `s` is a pavo expression, returns `{ :expression <expr>, :suffix <suffix>}` where `<expr>` is the value denoted by that expression and `<suffix>` is the remainder of the string.

Throws `{ :tag :err-not-expression }` if no prefix of the string is a valid pavo expression.

Time: O(n), where n is `(string-count <prefix>)`, where `<prefix>` is the longest prefix of `s` that is a pavo expression.

```pavo
(assert-eq (read-prefix "42") {:expression 42, :suffix ""})
(assert-eq (read-prefix "(a) ") {:expression $(a), :suffix " "})
(assert-throw (read-str "(a) b") {:expression $(a), :suffix " b"})
```

#### (write v)

Returns a string `s` such that `(read s)` equals the value `v`. The precise definition is given at the end of this document.

Throws `{ :tag :err-not-writable }` if no such string exists. This is the case if `v` is or contains a function or a symbol.

Time: Linear in the length of the returned string. Yeah, that's not a proper definition...

```pavo
(assert-eq (write 42) "42")
(assert-eq (write $(a )) "(a)")
(assert-throw (write (symbol)) { :tag :err-not-writable })
```

TODO expand, evaluate, etc.

### Miscellaneous

#### `(typeof x)`

Returns a keyword indicating the type of `x`: `:nil`, `:bool`, `:int`, `:float`, `:char`, `:string`, `:bytes`, `:keyword`, `:identifier`, `:symbol`, `:function`, `:array`, `:application`, `:map` or `:set`.

```pavo
(assert-eq (typeof nil) :nil)
(assert-eq (typeof true) :bool)
(assert-eq (typeof 42) :int)
(assert-eq (typeof 0.0) :float)
(assert-eq (typeof 'a') :char)
(assert-eq (typeof "foo") :string)
(assert-eq (typeof @[]) :bytes)
(assert-eq (typeof :kw) :keyword)
(assert-eq (typeof `id) :identifier)
(assert-eq (typeof (symbol)) :symbol)
(assert-eq (typeof typeof) :function)
(assert-eq (typeof []) :array)
(assert-eq (typeof `()) :application)
(assert-eq (typeof {}) :map)
(assert-eq (typeof @{}) :set)
```

#### `(truthy? x)`

Returns `false` if `x` is `nil` or `false`, and `true` otherwise.

Equivalent to `(if x true false)`.

```pavo
(assert-not (truthy? nil))
(assert-not (truthy? false))
(assert (truthy? true)
(assert (truthy? 0))
(assert (truthy? truthy?))
```

#### `(diverge)` `(diverge v)`

Immediately and abnormally terminates the execution of the program. Semantically you can think of this as going into an infinite loop, but telling the outside about it to save resources and give feedback to the programmer. In the pavo semantics, passing a value `v` does nothing whatsoever, but the runtime should somehow pass this value to the outside world for additional context.

Note that this is different from a program terminating through an uncaught throw and you should almost always throw instead of deliberately calling `diverge` (just as there are very few situations where you'd deliberately go into an effect-free infinite loop).

## Appendix: Precise Definition of `(write v)`

This section defines the return value of `(write v)` for any expression `v`, defined through structural induction.

### Base Cases

- `(= v nil)`: `"nil"`
- `(= v true)`: `"true"`
- `(= v false)`: `"false"`
- `(= (typeof v) :int)` and `(>= v 0)`: The decimal representation of the integer (without sign).
- `(= (typeof v) :int)` and `(< v 0)`: The minus sign `-` followed by the decimal representation of the absolute value of the integer.

TODO
