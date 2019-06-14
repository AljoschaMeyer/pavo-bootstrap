TODO limit string size by utf-8 byte length rather than char count?

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

#### `(int-add n m)`

Adds the int `n` to the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-add 1 2) 3)
(assert-eq (int-add 1 -2) -1)
(assert-throw (int-add int-max 1) { :tag :err-wrap-int })
```

#### `(int-sub n m)`

Subtracts the int `m` from the int `n`.

Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-sub 1 2) -1)
(assert-eq (int-sub 1 -2) 3)
(assert-throw (int-sub int-min 1) { :tag :err-wrap-int })
```

#### `(int-mul n m)`

Multiplies the int `n` with the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-mul 2 3) 6)
(assert-eq (int-mul 2 -3) -6)
(assert-throw (int-mul int-max 2) { :tag :err-wrap-int })
```

#### `(int-div n m)`

Divides the int `n` by the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow. Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the quotient of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-div 8 3) 2)
(assert-eq (int-div -8 3) -3)
(assert-throw (int-div int-min -1) { :tag :err-wrap-int })
(assert-throw (int-div 1 0) { :tag :err-zero })
```

#### `(int-div-trunc n m)`

Divides the int `n` by the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow. Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the quotient of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-div-trunc 8 3) 2)
(assert-eq (int-div-trunc -8 3) -2)
(assert-throw (int-div-trunc int-min -1) { :tag :err-wrap-int })
(assert-throw (int-div-trunc 1 0) { :tag :err-zero })
```

#### `(int-mod n m)`

Computes the int `n` modulo the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow. Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the remainder of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-mod 8 3) 2)
(assert-eq (int-mod -8 3) 1)
(assert-throw (int-mod int-min -1) { :tag :err-wrap-int })
(assert-throw (int-mod 1 0) { :tag :err-zero })
```

#### `(int-mod-trunc n m)`

Computes the int `n` modulo the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow. Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the remainder of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-mod-trunc 8 3) 2)
(assert-eq (int-mod-trunc -8 3) -2)
(assert-throw (int-mod-trunc int-min -1) { :tag :err-wrap-int })
(assert-throw (int-mod-trunc 1 0) { :tag :err-zero })
```

#### `(int-neg n)`

Negates the int `n`.Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-neg 42) -42)
(assert-eq (int-neg -42) 42)
(assert-eq (int-neg 0) 0)
(assert-throw (int-neg int-min) { :tag :err-wrap-int })
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

#### `(int-abs n)`

Returns the absolute value of the int `n`.

Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-abs 42) 42)
(assert-eq (int-abs -42) 42)
(assert-eq (int-abs 0) 0)
(assert-throw (int-abs int-min) { :tag :err-wrap-int })
```

#### `(int-pow n m)`

Computes the int `n` to the power of the positive int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-pow 2 3) 8)
(assert-eq (int-pow 2 0) 1)
(assert-eq (int-pow 0 999) 0)
(assert-eq (int-pow 1 999) 1)
(assert-eq (int-pow -1 999) -1)
(assert-throw (int-pow 99 99) { :tag :err-wrap-int })
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

#### `(int-pow-sat n m)`

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

#### `(int-div-wrap n m)`

Divides the int `n` by the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the quotient of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-div-wrap 8 3) 2)
(assert-eq (int-div-wrap -8 3) -3)
(assert-eq (int-div-wrap int-min -1) int-min)
(assert-throw (int-div-wrap 1 0) { :tag :err-zero })
```

#### `(int-div-trunc-wrap n m)`

Divides the int `n` by the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the quotient of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-div-trunc-wrap 8 3) 2)
(assert-eq (int-div-trunc-wrap -8 3) -2)
(assert-eq (int-div-trunc-wrap int-min -1) int-min)
(assert-throw (int-div-trunc-wrap 1 0) { :tag :err-zero })
```

#### `(int-mod-wrap n m)`

Computes the int `n` modulo the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the remainder of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-mod-wrap 8 3) 2)
(assert-eq (int-mod-wrap -8 3) 1)
(assert-eq (int-mod-wrap int-min -1) 0)
(assert-throw (int-mod-wrap 1 0) { :tag :err-zero })
```

#### `(int-mod-trunc-wrap n m)`

Computes the int `n` modulo the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the remainder of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-mod-trunc-wrap 8 3) 2)
(assert-eq (int-mod-trunc-wrap -8 3) -2)
(assert-eq (int-mod-trunc-wrap int-min -1) 0)
(assert-throw (int-mod-trunc-wrap 1 0) { :tag :err-zero })
```

#### `(int-neg-wrap n)`

Negates the int `n`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-neg 42) -42)
(assert-eq (int-neg -42) 42)
(assert-eq (int-neg 0) 0)
(assert-eq (int-neg int-min) int-min)
```

#### `(int-abs-wrap n)`

Returns the absolute value of the int `n`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-abs-wrap 42) 42)
(assert-eq (int-abs-wrap -42) 42)
(assert-eq (int-abs-wrap 0) 0)
(assert-eq (int-abs-wrap int-min) int-min)
```

#### `(int-pow-wrap n m)`

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

#### `(bytes-get b index)`

Returns the byte at the int `index` in the bytes `b`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-get @[42] 0) 42)
(assert-throw (bytes-get [] 0) { :tag :err-lookup, :got 0})
```

#### `(bytes-insert b index new)`

Inserts the byte `new` into the bytes `b` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-not-byte, :got new}` if `new` is not a byte (an int between 0 and 255 inclusive).
Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-insert @[0 1] 0 42) [42 0 1])
(assert-eq (bytes-insert @[0 1] 1 42) [0 42 1])
(assert-eq (bytes-insert @[0 1] 2 42) [0 1 42])
(assert-throw (bytes-insert @[0 1] 3 42) { :tag :err-lookup, :got 3})
(assert-throw (bytes-insert @[] 0 256) { :tag :err-not-byte, :got 256})
(assert-throw (bytes-insert @[] 0 256 nil) { :tag :err-not-byte, :got 256})
```

#### `(bytes-remove b index)`

Returns the bytes obtained by removing the byte at the index int `index` from the bytes `b`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-remove @[0 1] 0) @[1])
(assert-eq (bytes-remove @[0 1] 1) @[0])
(assert-throw (bytes-remove @[0 1] 3) { :tag :err-lookup, :got 3})
```

#### `(bytes-update b index new)`

Returns the bytes obtained by replacing the byte at the index int `index` in the bytes `b` with the byte `new`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-not-byte, :got new}` if `new` is not a byte (an int between 0 and 255 inclusive).

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-update @[0 1] 0 42) @[42 1])
(assert-eq (bytes-update @[0 1] 1 42) @[0 42])
(assert-throw (bytes-update @[0 1] 2 42) { :tag :err-lookup, :got 2})
(assert-throw (bytes-update @[] 0 256) { :tag :err-not-byte, :got 256})
(assert-throw (bytes-update @[] 0 256 nil) { :tag :err-not-byte, :got 256})
```

#### `(bytes-slice b start end)`

Returns a subsequence of the bytes `b`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-slice @[42 43] 1 1) [])
(assert-eq (bytes-slice @[42 43] 0 1) [42])
(assert-eq (bytes-slice @[42 43] 1 2) [43])
(assert-eq (bytes-slice @[42 43] 0 2) [42 43])
(assert-throw (bytes-slice @[] 0 1) { :tag :err-lookup, :got 1})
(assert-throw (bytes-slice @[] 2 3) { :tag :err-lookup, :got 2})
(assert-throw (bytes-slice @[0 1 2 3] 2 1) { :tag :err-lookup, :got 1})
```

#### `(bytes-splice old index new)`

Inserts the elements of the bytes `new` into the bytes `old`, starting at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds (of the `old` bytes).
Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(bytes-count old)` and m is `(bytes-count new)`.

```pavo
(assert-eq (bytes-splice @[0 1] @[10 11] 0) @[10 11 0 1])
(assert-eq (bytes-splice @[0 1] @[10 11] 1) @[0 10 11 1])
(assert-eq (bytes-splice @[0 1] @[10 11] 2) @[0 1 10 11])
(assert-throw (bytes-splice @[0 1] @[10 11] 3) { :tag :err-lookup, :got 3})
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

### Chars

#### `char-max`

The largest char (numerically the largest unicode scalar value).

```pavo
(assert-eq char-max '\{10ffff}')
```

#### `(int=>char n)`

Returns the unicode scalar value denoted by the int `n`.

Throws `{ :tag :err-not-unicode-scalar, :got n}` if `n` is `n` is not a unicode scalar value.

```pavo
(assert-eq (int=>char 0x41) 'A')
(assert-throw (int=>char 0x110000) { :tag :err-not-unicode-scalar, :got 0x110000})
```

#### `(int=>char? n)`

Returns whether the int `n` denotes a unicode scalar value.

```pavo
(assert (int=>char? 0x41))
(assert-not (int=>char? 0x110000))
```

#### `(char->int c)`

Returns the unicode scalar value of the char `c` as an int.

```pavo
(assert-eq (char->int 'A') 0x41)
```

### Strings

TODO utf-8 stuff

#### `(str-count s)`

Returns the number of chars in the string `s`.

Time: O(1).

```pavo
(assert-eq (str-count "") 0)
(assert-eq (str-count "a") 1)
(assert-eq (str-count "⚗") 1)
(assert-eq (str-count "abc") 3)
```

<!-- #### `(str-count-utf8 s)`

Returns the number of bytes in the utf8 encoding of the string `s`.

Time: O(1).

```pavo
(assert-eq (str-count-utf8 "") 0)
(assert-eq (str-count-utf8 "a") 1)
(assert-eq (str-count-utf8 "⚗") 3)
(assert-eq (str-count-utf8 "abc") 3)
``` -->

#### `(str-get s index)`

Returns the char at the int `index` in the string `s`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-get "a" 0) 'a')
(assert-eq (str-get "⚗b" 1) 'b')
(assert-throw (str-get "" 0) { :tag :err-lookup, :got 0})
```

#### `(str-insert s index c)`

Inserts the char `c` into the string `s` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-collection-full }` if the resulting string would contain 2^63 or more elements.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-insert "ab" 0 'z') "zab")
(assert-eq (str-insert "ab" 1 'z') "azb")
(assert-eq (str-insert "ab" 2 'z') "abz")
(assert-throw (str-insert "ab" 3 'z') { :tag :err-lookup, :got 3})
```

#### `(str-remove s index)`

Returns the string obtained by removing the char at the index int `index` from the string `s`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-remove "ab" 0) "b")
(assert-eq (str-remove "ab" 1) "a")
(assert-throw (str-remove "ab" 3) { :tag :err-lookup, :got 3})
```

#### `(str-update s index c)`

Returns the string obtained by replacing the char at the index int `index` in the string `s` with the char `c`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-update "ab" 0 'z') "zb")
(assert-eq (str-update "ab" 1 'z') "az")
(assert-throw (str-update "ab" 2 'z') { :tag :err-lookup, :got 2})
```

#### `(str-slice s start end)`

Returns a substring of the string `b`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-slice "ab" 1 1) "")
(assert-eq (str-slice "ab" 0 1) "a")
(assert-eq (str-slice "ab" 1 2) "b")
(assert-eq (str-slice "ab" 0 2) "ab")
(assert-throw (str-slice "" 0 1) { :tag :err-lookup, :got 1})
(assert-throw (str-slice "" 2 3) { :tag :err-lookup, :got 2})
(assert-throw (str-slice "abcd" 2 1) { :tag :err-lookup, :got 1})
```

#### `(str-splice old index new)`

Inserts the string `new` into the string `old`, starting at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds (of the `old` bytes).
Throws `{ :tag :err-collection-full }` if the resulting string would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(str-count old)` and m is `(str-count new)`.

```pavo
(assert-eq (str-splice "ab" "cd" 0) "cdab")
(assert-eq (str-splice "ab" "cd" 1) "acdb")
(assert-eq (str-splice "ab" "cd" 2) "abcd")
(assert-throw (str-splice "ab" "cd" 3) { :tag :err-lookup, :got 3})
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
    (str-iter "abcd" (fn [elem] (set! out (str-insert out 0 elem))))
    (assert-eq out "dcbaz")
))
(let :mut out "z" (do
    (str-iter "abcd" (fn [elem] (if
            (= elem 'c') true
            (set! out (str-insert out 0 elem))
        )))
    (assert-eq out "baz")
))
```

#### `(str-iter-back s fun)`

Starting from the back of the string `s`, applies the function `fun` to the chars of `s` in reverse order until either `fun` returns a truthy value or the end of the string is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(str-count s)`.

```pavo
(let :mut out "z" (do
    (str-iter-back "abcd" (fn [elem] (set! out (str-insert out 0 elem))))
    (assert-eq out "abcdz")
))
(let :mut out "z" (do
    (str-iter-back "abcd" (fn [elem] (if
            (= elem 'c') true
            (set! out (str-insert out 0 elem))
        )))
    (assert-eq out "dz")
))
```

### Floats

TODO

### Identifiers

#### `(str=>id s)`

Returns an identifier created from the string `s`.

Throws `{ :tag :err-identifier, :got s}` if it would not be a valid identifier (empty, longer than 255 characters, or containing invalid characters).

Time: O(n) where n is `(str-count s)`.

```pavo
(assert-eq (str=>id "foo") $foo)
(assert-eq (str=>id "nil") $nil)
(assert-eq (str=>id "42") $42)
(assert-throw (str=>id "") { :tag :err-identifier, :got ""})
(assert-throw (str=>id "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") { :tag :err-identifier, :got ""})
(assert-throw (str=>id ":a") { :tag :err-identifier, :got ":a"})
(assert-throw (str=>id "ß") { :tag :err-identifier, :got "ß"})
```

#### `(str=>id? s)`

Returns whether the string `s` would be a valid identifier, i.e. it is neither empty nor longer than 255 characters, contains only valid identifier characters, and is not a different literal.

```pavo
(assert-eq (str=>id? "foo") true)
(assert-eq (str=>id? "nil") false)
(assert-eq (str=>id? "42") false)
(assert-eq (str=>id? "-_") true)
(assert-eq (str=>id? "-42") false)
(assert-eq (str=>id? "") false)
(assert-eq (str=>id? "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") false)
(assert-eq (str=>id? "ß") false)
(assert-eq (str=>id? ":a") false)
```

#### `(id->str id)`

Returns the string that corresponds to the given identfier `id.

```pavo
(assert-eq (id->str $foo) "foo")
```

### Keywords

#### `(str=>kw s)`

Returns the keyword `<:s>` created from the string `s`.

Throws `{ :tag :err-kw, :got s}` if it would not be a valid keyword (empty, longer than 255 characters, or containing invalid characters).

Time: O(n) where n is `(str-count s)`.

```pavo
(assert-eq (str=>kw "foo") :foo)
(assert-eq (str=>kw "nil") :nil)
(assert-eq (str=>kw "42") :42)
(assert-throw (str=>kw "") { :tag :err-kw, :got ""})
(assert-throw (str=>kw "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") { :tag :err-kw, :got ""})
(assert-throw (str=>kw ":a") { :tag :err-kw, :got ":a"})
(assert-throw (str=>kw "ß") { :tag :err-kw, :got "ß"})
```

#### `(str=>kw? s)`

Returns whether the string `s` prefixed by a colon would be a valid keyword, i.e. it is neither empty nor longer than 255 characters and only contains valid identifier characters.

```pavo
(assert-eq (str=>kw? "foo") true)
(assert-eq (str=>kw? "nil") true)
(assert-eq (str=>kw? "42") true)
(assert-eq (str=>kw? "-_") true)
(assert-eq (str=>kw? "-42") true)
(assert-eq (str=>kw? "") false)
(assert-eq (str=>kw? "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") false)
(assert-eq (str=>kw? "ß") false)
(assert-eq (str=>kw? ":a") false)
```

#### `(kw->str kw)`

Returns the string that corresponds to the given keyword `kw`, without the leading colon.

```pavo
(assert-eq (kw->str :foo) "foo")
```

### Arrays

#### `(arr-count arr)`

Returns the number of elements in the array `arr`.

Time: O(1).

```pavo
(assert-eq (arr-count []) 0)
(assert-eq (arr-count [nil]) 1)
(assert-eq (arr-count [0, 1, 2]) 3)
```

#### `(arr-get arr index)`

Returns the element at the int `index` in the array `arr`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-get [true] 0) true)
(assert-throw (arr-get [] 0) { :tag :err-lookup, :got 0})
```

#### `(arr-insert arr index new)`

Inserts the value `new` into the array `arr` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-collection-full }` if the resulting array would contain 2^63 or more elements.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-insert [0 1] 0 42) [42 0 1])
(assert-eq (arr-insert [0 1] 1 42) [0 42 1])
(assert-eq (arr-insert [0 1] 2 42) [0 1 42])
(assert-throw (arr-insert [0 1] 3 42) { :tag :err-lookup, :got 3})
```

#### `(arr-remove arr index)`

Returns the array obtained by removing the element at the index int `index` from the array `arr`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-remove [0 1] 0) [1])
(assert-eq (arr-remove [0 1] 1) [0])
(assert-throw (arr-remove [0 1] 3) { :tag :err-lookup, :got 3})
```

#### `(arr-update arr index new)`

Returns the array obtained by replacing the element at the index int `index` in the array `arr` with the value `new`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-update [0 1] 0 42) [42 1])
(assert-eq (arr-update [0 1] 1 42) [0 42])
(assert-throw (arr-update [0 1] 2 42) { :tag :err-lookup, :got 2})
```

#### `(arr-slice arr start end)`

Returns an array containing a subsequence of the elements of the array `arr`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-slice [true false] 1 1) [])
(assert-eq (arr-slice [true false] 0 1) [true])
(assert-eq (arr-slice [true false] 1 2) [false])
(assert-eq (arr-slice [true false] 0 2) [true false])
(assert-throw (arr-slice [] 0 1) { :tag :err-lookup, :got 1})
(assert-throw (arr-slice [] 2 3) { :tag :err-lookup, :got 2})
(assert-throw (arr-slice [0 1 2 3] 2 1) { :tag :err-lookup, :got 1})
```

#### `(arr-splice old index new)`

Inserts the elements of the array `new` into the array `old`, starting at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds (of the `old` array).
Throws `{ :tag :err-collection-full }` if the resulting array would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(arr-count old)` and m is `(arr-count new)`.

```pavo
(assert-eq (arr-splice [0 1] [10 11] 0) [10 11 0 1])
(assert-eq (arr-splice [0 1] [10 11] 1) [0 10 11 1])
(assert-eq (arr-splice [0 1] [10 11] 2) [0 1 10 11])
(assert-throw (arr-splice [0 1] [10 11] 3) { :tag :err-lookup, :got 3})
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

### Applications

#### `(app-count app)`

Returns the number of elements in the application `app`.

Time: O(1).

```pavo
(assert-eq (app-count $()) 0)
(assert-eq (app-count $(nil)) 1)
(assert-eq (app-count $(0, 1, 2)) 3)
```

#### `(app-get app index)`

Returns the element at the int `index` in the application `app`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-get $(true) 0) true)
(assert-throw (app-get $() 0) { :tag :err-lookup, :got 0})
```

#### `(app-insert app index new)`

Inserts the value `new` into the application `app` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-collection-full }` if the resulting application would contain 2^63 or more elements.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-insert $(0 1) 0 42) $(42 0 1))
(assert-eq (app-insert $(0 1) 1 42) $(0 42 1))
(assert-eq (app-insert $(0 1) 2 42) $(0 1 42))
(assert-throw (app-insert $(0 1) 3 42) { :tag :err-lookup, :got 3})
```

#### `(app-remove app index)`

Returns the application obtained by removing the element at the index int `index` from the application `app`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-remove $(0 1) 0) $(1))
(assert-eq (app-remove $(0 1) 1) $(0))
(assert-throw (app-remove $(0 1) 3) { :tag :err-lookup, :got 3})
```

#### `(app-update app index new)`

Returns the application obtained by replacing the element at the index int `index` in the application `app` with the value `new`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-update $(0 1) 0 42) $(42 1))
(assert-eq (app-update $(0 1) 1 42) $(0 42))
(assert-throw (app-update $(0 1) 2 42) { :tag :err-lookup, :got 2})
```

#### `(app-slice app start end)`

Returns an application containing a subsequence of the elements of the application `app`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-slice $(true false) 1 1) $())
(assert-eq (app-slice $(true false) 0 1) $(true))
(assert-eq (app-slice $(true false) 1 2) $(false))
(assert-eq (app-slice $(true false) 0 2) $(true false))
(assert-throw (app-slice $() 0 1) { :tag :err-lookup, :got 1})
(assert-throw (app-slice $() 2 3) { :tag :err-lookup, :got 2})
(assert-throw (app-slice $(0 1 2 3) 2 1) { :tag :err-lookup, :got 1})
```

#### `(app-splice old index new)`

Inserts the elements of the application `new` into the application `old`, starting at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds (of the `old` application).
Throws `{ :tag :err-collection-full }` if the resulting application would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(app-count old)` and m is `(app-count new)`.

```pavo
(assert-eq (app-splice $(0 1) $(10 11) 0) $(10 11 0 1))
(assert-eq (app-splice $(0 1) $(10 11) 1) $(0 10 11 1))
(assert-eq (app-splice $(0 1) $(10 11) 2) $(0 1 10 11))
(assert-throw (app-splice $(0 1) $(10 11) 3) { :tag :err-lookup, :got 3})
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

#### `(set-min set)`

Returns the minimal element in the set `set`.

Throws `{ :tag :err-collection-empty }` if `set` is the empty set.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-min @{ 4 3 }) 3)
(assert-throw (set-min @{}) { :tag :err-collection-empty })
```

#### `(set-max set)`

Returns the maximal element in the set `set`.

Throws `{ :tag :err-collection-empty }` if `set` is the empty set.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-max @{ 4 3 }) 4)
(assert-throw (set-max @{}) { :tag :err-collection-empty })
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

#### `(map-get map key)`

Returns the value associated with the key `key` in the map `map`.

Throws `{ :tag :err-lookup, :got key}` if the map contains no entry with this key.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-get {0 42} 0) 42)
(assert-throw (map-get {} 0) { :tag :err-lookup, :got 0})
```

#### `(map-contains? map key)`

Returns `true` if the map `map` contains an entry with key `key`, `false` otherwise.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-contains? { nil 0 } nil) true)
(assert-eq (map-contains? { 42 0 } 43) false)
(assert-eq (map-contains? {} nil) false)
```

#### `(map-min map)`

Returns the value of the entry with the minimal key in the map `map`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min @{0 42, 1 41}) 42)
(assert-throw (map-min {}) { :tag :err-collection-empty })
```

#### `(map-min-key map)`

Returns the minimal key in the map `map`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min-key @{0 42, 1 41}) 0)
(assert-throw (map-min-key {}) { :tag :err-collection-empty })
```

#### `(map-min-entry map)`

Returns the entry with the minimal key in the map `map`, as an array `[key value]`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min-entry @{0 42, 1 41}) [0 42])
(assert-throw (map-min-entry {}) { :tag :err-collection-empty })
```

#### `(map-max map)`

Returns the value of the entry with the maximal key in the map `map`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max @{0 42, 1 41}) 41)
(assert-throw (map-max {}) { :tag :err-collection-empty })
```

#### `(map-max-key map)`

Returns the maximal key in the map `map`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max-key @{0 42, 1 41}) 1)
(assert-throw (map-max-key {}) { :tag :err-collection-empty })
```

#### `(map-max-entry map)`

Returns the entry with the maximal key in the map `map`, as an array `[key value]`.

Throws `{ :tag :err-collection-empty }` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max-entry @{0 42, 1 41}) [1 41])
(assert-throw (map-max-entry {}) { :tag :err-collection-empty })
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

#### `(read s)`

If the string `s` is a pavo expression, returns the value denoted by that expression.

Throws `{ :tag :err-not-expression }` if the string is not a valid pavo expression.

Time: O(n), where n is `(string-count <prefix>)`, where `<prefix>` is the longest prefix of `s` that is a pavo expression.

```pavo
(assert-eq (read "42") 42)
(assert-eq (read "(a) ") $(a))
(assert-throw (read "(a) b") { :tag :err-not-expression })
```

#### `(read-prefix)`

If a prefix of the string `s` is a pavo expression, returns `{ :expression <expr>, :suffix <suffix>}` where `<expr>` is the value denoted by that expression and `<suffix>` is the remainder of the string.

Throws `{ :tag :err-not-expression }` if no prefix of the string is a valid pavo expression.

Time: O(n), where n is `(string-count <prefix>)`, where `<prefix>` is the longest prefix of `s` that is a pavo expression.

```pavo
(assert-eq (read-prefix "42") {:expression 42, :suffix ""})
(assert-eq (read-prefix "(a) ") {:expression $(a), :suffix " "})
(assert-throw (read-str "(a) b") {:expression $(a), :suffix " b"})
```

#### `(write v)`

Returns a string `s` such that `(read s)` equals the value `v`. The precise definition is given at the end of this document.

Throws `{ :tag :err-not-writable }` if no such string exists. This is the case if `v` is or contains a function or a symbol.
Throws `{ :tag :err-collection-full }` if the resulting string would contain 2^63 or more elements.

Time: Linear in the length of the returned string. Yeah, that's not a proper definition...

```pavo
(assert-eq (write 42) "42")
(assert-eq (write $(a )) "(a)")
(assert-throw (write (symbol)) { :tag :err-not-writable })
```

TODO expand, check, evaluate, etc.

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

Immediately and abnormally terminates the execution of the program. Semantically you can think of this as going into an infinite loop, but telling the outside world about it to save resources and give feedback to the programmer. In the pavo semantics, passing a value `v` does nothing whatsoever, but the runtime should somehow pass this value to the outside world for additional context.

Note that this is different from a program terminating through an uncaught throw and you should almost always throw instead of deliberately calling `diverge` (just as there are very few situations where you'd deliberately go into an effect-free infinite loop).

## Appendix: Precise Definition of `(write v)`

This section defines the return value of `(write v)` for any expression `v`, defined through structural induction (examples/tests are below).

### Base Cases

- `(= v nil)`: `"nil"`
- `(= v true)`: `"true"`
- `(= v false)`: `"false"`
- `(= (typeof v) :int)`:
  - `(>= v 0)`: The decimal representation of the integer (without sign).
  - `(< v 0)`: The minus sign `-` followed by the decimal representation of the absolute value of the integer.
- `(= (typeof v) :char)`:
  - `(= v '\\')`: `"'\\'"`
  - `(= v '\'')`: `"'\''"`
  - `(= v '\n')`: `"'\n'"`
  - `(= v '\t')`: `"'\t'"`
  - otherwise: `'` followed by the character followed by `'`
- `(= (typeof v) :string)`: `"` followed by the characters as defined next followed by `"`.
  - characters: for each each char `c` in the string:
    - `(= c '\\')`: `\\`
    - `(= c '\")`: `\"`
    - `(= c '\n')`: `\n`
    - `(= c '\t')`: `\t`
    - otherwise: the character itself
- `(= (typeof v) :bytes)`: `@[` followed by the bytes as defined next followed by `]`
  - bytes: for each byte `b` the decimal representation, all but the last byte followed by a space
- `(= (typeof v) :keyword)`: the keyword itself
- `(= (typeof v) :identifier)`: the identifier itself
- `(= (typeof v) :symbol)`: throw `{ :tag :err-not-writable }`
- `(= (typeof v) :function)`: throw `{ :tag :err-not-writable }`

### Induction Steps

Collections serialize their components and separate them by a single space (there's no whitespace at the front or the back of the collection).

### Examples

```pavo
(assert-eq (write nil) "nil")
(assert-eq (write true) "true")
(assert-eq (write false) "false")

(assert-eq (write 0) "0")
(assert-eq (write 1) "1")
(assert-eq (write -1) "-1")
(assert-eq (write -0) "0")

(assert-eq (write 'a') "'a'")
(assert-eq (write '"') "'"'")
(assert-eq (write '🌃') "'🌃'")
(assert-eq (write '\t') "'\\t'")
(assert-eq (write '\n') "'\\n'")
(assert-eq (write '\\') "'\\\\'")
(assert-eq (write '\'') "'\\''")

(assert-eq (write "a") "\"a\"")
(assert-eq (write "'") "\"'\"")
(assert-eq (write "🌃") "\"🌃\"")
(assert-eq (write "") "\"\"")
(assert-eq (write "ab") "\"ab\"")
(assert-eq (write "\t") "\"\\t\"")
(assert-eq (write "\n") "\"\\n\"")
(assert-eq (write "\\") "\"\\\\\"")
(assert-eq (write "\"") "\"\\"\"")

(assert-eq (write @[ ]) "@[]")
(assert-eq (write @[ 0x11 ]) "@[17]")
(assert-eq (write @[1, 2]) "@[1 2]")

(assert-eq (write :foo) ":foo")

(assert-eq (write $foo) "foo")

(assert-throw (write (symbol)) {:tag :err-not-writable})
(assert-throw (write write) {:tag :err-not-writable})

(assert-eq (write [ ]) "[]")
(assert-eq (write [ 2]) "[2]")
(assert-eq (write [ 2, 4 ]) "[2 4]")

(assert-eq (write $()) "()")
(assert-eq (write $(2)) "(2)")
(assert-eq (write $(2, 4)) "(2 4)")

(assert-eq (write @{}) "@{}")
(assert-eq (write @{1}) "@{1}")
(assert-eq (write @{1 1}) "@{1}")
(assert-eq (write @{2 , 1  3}) "@{1 2 3}")

(assert-eq (write {}) "{}")
(assert-eq (write {1 nil}) "{1 nil}")
(assert-eq (write {1 nil 1 nil}) "{1 nil}")
(assert-eq (write {2 nil , 1 nil  3 nil}) "{1 nil 2 nil 3 nil}")
```

TODO floats... those are fun!
