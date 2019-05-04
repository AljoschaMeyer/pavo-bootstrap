# The Pavo Language Reference

## Syntax

## Values

## Evaluation

## Macro Expansion

## Toplevel Macros

## Toplevel Values

These are all the values that are bound to an identifier in the toplevel environment in which code is executed by default. All of these bindings are immutable.

In the example code blocks, all "statements" evaluate to `nil`, none throws. If you put all examples into a `sf-do` special form, it would evaluate to `nil`.

Whenever a function is described to "throw a type error", it throws a map `{ :tag :err-type, :expected <expected>, :got <got>}` where `<expected>` and `<got>` are the keywords denoting the respective types (see `(typeof x)`). Type errors are also trown when an argument is described as having a certain type, but an argument of a different type is supplied. For example "Do foo to the int `n`" throws a type error with `:expected :int` if `n` is not an int.

Whenever an argument is referred to as a "positive int", but an int less than zero is supplied, the function throws `{ :tag :err-negative, :got <got>}`, where `<got>` is the supplied, negative int.

TODO Specify errors that are thrown on incorrect number of args
TODO specify in which order all errors apply.

### Nil

#### `(nil? x)`

Returns `true` if `x` is `nil`, `false` otherwise.

Equivalent to `(= (typeof x) :nil)`.

```pavo
(assert (nil? nil))
(assert-not (nil? false))
(assert-not (nil? 0))
```

### Bool

#### `(bool? x)`

Returns `true` if `x` is a bool, `false` otherwise.

Equivalent to `(= (typeof x) :bool)`.

```pavo
(assert (bool? true))
(assert (bool? false))
(assert-not (bool? nil))
(assert-not (bool? "true"))
```

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

There are no functions whose behavior depends on the endianess of the executing machine, since this would be nondeterministic behavior with respect to the pavo semantics. Implementations are highly encouraged to supply the following functions to the entry of a pavo program:

- `(int-from-be n)`: Converts the int `n` from big endian to the target's endianness.
- `(int-from-le n)`: Converts the int `n` from little endian to the target's endianness.
- `(int-to-be n)`: Converts the int `n` to big endian from the target's endianness.
- `(int-to-le n)`: Converts the int `n` to little endian from the target's endianness.

Additionally, they should provide the value `endianess`: Either `:be` or `le` depending on the target's endianess.

TODO move the section on endianess somewhere else, since this also effects some bytes methods

#### `(int? x)`

Returns `true` if `x` is an int, `false` otherwise.

Equivalent to `(= (typeof x) :int)`.

```pavo
(assert (int? 0))
(assert-not (int? 0.0))
```

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

### Arrays

### `(arr-count arr)`

Returns the number of elements in the array `arr`.

```pavo
(assert-eq (arr-count []) 0)
(assert-eq (arr-count [nil]) 1)
(assert-eq (arr-count [0, 1, 2]) 3)
```

### `(arr-empty? arr)`

Returns whether the count of elements in the array `arr` is zero.

Equivalent to `(= (arr-count arr) 0)`.

```pavo
(assert (arr-empty? []))
(assert (arr-empty? [nil]))
```

### `(arr-get arr index)` `(arr-get arr index default)`

Returns the element at the int `index` in the array `arr`. If `default` is not supplied, throws `{ :tag :err-index, :got index}` if the index is out of bounds. If `default` is supplied, returns `default` if the index is out of bounds.

```pavo
(assert-eq (arr-get [true] 0) true)
(assert-throw (arr-get [] 0) { :tag :err-index, :got 0})
(assert-eq (arr-get [] 0 nil) nil)
```

### `(arr-front arr)` `(arr-front arr default)`

Returns the first element in the array `arr`. If `default` is not supplied, throws `{ :tag :err-index, :got 0}` if the array is empty. If `default` is supplied, returns `default` if the array is empty.

Equivalent to `(arr-get arr 0)` and `(arr-get arr 0 default)`.

```pavo
(assert-eq (arr-front [true false]) true)
(assert-throw (arr-front []) { :tag :err-index, :got 0})
(assert-eq (arr-front [] nil) nil)
```

### `(arr-back arr)` `(arr-back arr default)`

Returns the last element in the array `arr`. If `default` is not supplied, throws `{ :tag :err-index, :got 0}` if the array is empty. If `default` is supplied, returns `default` if the array is empty.

Equivalent to `(arr-get arr (- (arr-count arr) 1))` and `(arr-get arr (- (arr-count arr) 1) default)`.

```pavo
(assert-eq (arr-back [true false]) false)
(assert-throw (arr-back []) { :tag :err-index, :got 0})
(assert-eq (arr-back [] nil) nil)
```

### `(arr-slice arr start end)` `(arr-slice arr start end default)`

Returns an array containing a subsequence of the elements of the array `arr`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-index, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-index, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-index, :got end}` if `end` is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

```pavo
(assert-eq (arr-slice [true false] 1 1) [])
(assert-eq (arr-slice [true false] 0 1) [true])
(assert-eq (arr-slice [true false] 1 2) [false])
(assert-eq (arr-slice [true false] 0 2) [true false])
(assert-throw (arr-slice [] 0 1) { :tag :err-index, :got 1})
(assert-throw (arr-slice [] 2 3) { :tag :err-index, :got 2})
(assert-throw (arr-slice [0 1 2 3] 2 1) { :tag :err-index, :got 1})
(assert-eq (arr-slice [] 0 1 nil) nil)
```

### `(arr-slice-sat arr start end)`

Returns an array containing a subsequence of the elements of the array `arr`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive). Clamps `start` and `end` to valid indexes. Returns the empty array if `start` is greater than `end`.

```pavo
(assert-eq (arr-slice-sat [true false] 1 1) [])
(assert-eq (arr-slice-sat [true false] 0 1) [true])
(assert-eq (arr-slice-sat [true false] 1 2) [false])
(assert-eq (arr-slice-sat [true false] 0 2) [true false])
(assert-eq (arr-slice-sat [true false] -42 42) [true false])
(assert-eq (arr-slice-sat [] 2 3) [])
(assert-eq (arr-slice-sat [0 1 2 3] 2 1) [])
```

### `(arr-split arr index)` `(arr-split arr index default)`

Returns an array of two elements, first an array containing all the elements of the array `arr` up until the index int `index`, and second an array containing all remaining elements.

Throws `{ :tag :err-index, :got index}` if the index is out of bounds.

If `default` is supplied, returns `default` instead of throwing.

```pavo
(assert-eq (arr-split [true false] 0) [[] [true false]])
(assert-eq (arr-split [true false] 1) [[true] [false]])
(assert-eq (arr-split [true false] 2) [[true false] []])
(assert-throw (arr-split [true false] 3) { :tag :err-index, :got 3})
(assert-eq (arr-split [true false] 3 nil) nil)
```

### `(arr-split-sat arr index)`

Returns an array of two elements, first an array containing all the elements of the array `arr` up until the index int `index`, and second an array containing all remaining elements. Clamps `index` to a valid index.


```pavo
(assert-eq (arr-split [true false] 0) [[] [true false]])
(assert-eq (arr-split [true false] 1) [[true] [false]])
(assert-eq (arr-split [true false] 2) [[true false] []])
(assert-eq (arr-split [true false] -42) [[] [true false]])
(assert-throw (arr-split [true false] 3) [[true false] []])
```

### `(arr-take arr n)` `(arr-take arr n fallback)`

Returns an array containing the first `n` elements of the array `arr`.

Throws `{ :tag :err-index, :got index}` if the `n` is greater than `(arr-count arr)`.

If `default` is supplied, returns `default` instead of throwing.

```pavo
(assert-eq (arr-take [true false] 0) [])
(assert-eq (arr-take [true false] 1) [true])
(assert-eq (arr-take [true false] 2) [true false])
(assert-throw (arr-take [true false] 3) { :tag :err-index, :got 3})
(assert-eq (arr-take [true false] 3 nil) nil)
```

### `(arr-take-sat arr n)`

Returns an array containing the first `n` elements of the array `arr`. Clamps `n` to a valid int.

```pavo
(assert-eq (arr-take-sat [true false] 0) [])
(assert-eq (arr-take-sat [true false] 1) [true])
(assert-eq (arr-take-sat [true false] 2) [true false])
(assert-eq (arr-take-sat [true false] -42) [])
(assert-eq (arr-take-sat [true false] 3) [true false])
```

### `(arr-take-back arr n)` `(arr-take-back arr n fallback)`

Returns an array containing the last `n` elements of the array `arr`.

Throws `{ :tag :err-index, :got index}` if the `n` is greater than `(arr-count arr)`.

If `default` is supplied, returns `default` instead of throwing.

```pavo
(assert-eq (arr-take-back [true false] 0) [])
(assert-eq (arr-take-back [true false] 1) [false])
(assert-eq (arr-take-back [true false] 2) [true false])
(assert-throw (arr-take-back [true false] 3) { :tag :err-index, :got 3})
(assert-eq (arr-take-back [true false] 3 nil) nil)
```

### `(arr-take-back-sat arr n)`

Returns an array containing the last `n` elements of the array `arr`. Clamps `n` to a valid int.

```pavo
(assert-eq (arr-take-back-sat [true false] 0) [])
(assert-eq (arr-take-back-sat [true false] 1) [false])
(assert-eq (arr-take-back-sat [true false] 2) [true false])
(assert-eq (arr-take-back-sat [true false] -42) [])
(assert-eq (arr-take-back-sat [true false] 3) [true false])
```

### `(arr-take-while arr pred)`

Returns the longest prefix of elements in the array `arr` for which `(pred elem)` evaluates truthy. Iterates over the elements from left to right.

```pavo
(assert-eq (arr-take-while [0 1 nil false 4] int?) [0 1])
(assert-eq (arr-take-while [nil false 2] int?) [])
(assert-eq (arr-take-while [] int?) [])
(assert-eq (arr-take-while [0 1] int?) [0 1])
```

### `(arr-take-while-back arr pred)`

Returns the longest suffix of elements in the array `arr` for which `(pred elem)` evaluates truthy. Iterates over the elements from right to left.

```pavo
(assert-eq (arr-take-while-back [4 false nil 1 0] int?) [1 0])
(assert-eq (arr-take-while-back [4 false nil] int?) [])
(assert-eq (arr-take-while-back [] int?) [])
(assert-eq (arr-take-while-back [1 0] int?) [1 0])
```

### `(arr-skip arr n)` `(arr-skip arr n fallback)`

Returns an array containing all but the first `n` elements of the array `arr`.

Throws `{ :tag :err-index, :got index}` if the `n` is greater than `(arr-count arr)`.

If `default` is supplied, returns `default` instead of throwing.

```pavo
(assert-eq (arr-skip [true false] 0) [true false])
(assert-eq (arr-skip [true false] 1) [false])
(assert-eq (arr-skip [true false] 2) [])
(assert-throw (arr-skip [true false] 3) { :tag :err-index, :got 3})
(assert-eq (arr-skip [true false] 3 nil) nil)
```

### `(arr-skip-sat arr n)`

Returns an array containing all but the first `n` elements of the array `arr`. Clamps `n` to a valid int.

```pavo
(assert-eq (arr-skip-sat [true false] 0) [true false])
(assert-eq (arr-skip-sat [true false] 1) [true])
(assert-eq (arr-skip-sat [true false] 2) [])
(assert-eq (arr-skip-sat [true false] -42) [true false])
(assert-eq (arr-skip-sat [true false] 3) [])
```

### `(arr-skip-back arr n)` `(arr-skip-back arr n fallback)`

Returns an array containing all but the last `n` elements of the array `arr`.

Throws `{ :tag :err-index, :got index}` if the `n` is greater than `(arr-count arr)`.

If `default` is supplied, returns `default` instead of throwing.

```pavo
(assert-eq (arr-skip-back [true false] 0) [true false])
(assert-eq (arr-skip-back [true false] 1) [true])
(assert-eq (arr-skip-back [true false] 2) [])
(assert-throw (arr-skip-back [true false] 3) { :tag :err-index, :got 3})
(assert-eq (arr-skip-back [true false] 3 nil) nil)
```

### `(arr-skip-back-sat arr n)`

Returns an array containing all but the last `n` elements of the array `arr`. Clamps `n` to a valid int.

```pavo
(assert-eq (arr-skip-back-sat [true false] 0) [true false])
(assert-eq (arr-skip-back-sat [true false] 1) [true])
(assert-eq (arr-skip-back-sat [true false] 2) [])
(assert-eq (arr-skip-back-sat [true false] -42) [true false])
(assert-eq (arr-skip-back-sat [true false] 3) [])
```

### `(arr-skip-while arr pred)`

Returns the array obtained by removing the longest prefix of elements in the array `arr` for which `(pred elem)` evaluates truthy. Iterates over the elements from left to right.

```pavo
(assert-eq (arr-skip-while [0 1 nil false 4] int?) [nil false 4])
(assert-eq (arr-skip-while [nil false 2] int?) [nil false 2])
(assert-eq (arr-skip-while [] int?) [])
(assert-eq (arr-skip-while [0 1] int?) [])
```

### `(arr-skip-while-back arr pred)`

Returns the array obtained by removing the longest suffix of elements in the array `arr` for which `(pred elem)` evaluates truthy. Iterates over the elements from right to left.

```pavo
(assert-eq (arr-skip-while-back [4 false nil 1 0] int?) [4 false nil])
(assert-eq (arr-skip-while-back [2 false nil] int?) [2 false nil])
(assert-eq (arr-skip-while-back [] int?) [])
(assert-eq (arr-skip-while-back [1 0] int?) [])
```

---

init, tail
append, prepend, concat
insert, insert-all, update
all?, any?, none?
prefix?, suffix?
chain/flatmap
filter/retain, reject
(for-)each, map
fold, fold-back, fold-while, fold-back-while
unfold, unfold-back
scan, scan-back
repeat, unit, empty
reverse

<!-- ### `(arr-tail arr)` `(arr-tail arr default)`

Returns an array containing all but the first element of the array `arr`.

Throws `{ :tag :err-index, :got 1}` if the array is empty.

If `default` is supplied, returns `default` instead of throwing.

Equivalent to `(arr-drop arr 1)` and `(arr-drop arr 1 default)`.

```pavo
(assert-eq (arr-tail [true false 42]) [false 42])
(assert-throw (arr-tail []) { :tag :err-index, :got 1})
(assert-eq (arr-tail [] nil) nil)
```

### `(arr-init arr)` `(arr-init arr default)`

Returns an array containing all but the last element of the array `arr`.

Throws `{ :tag :err-index, :got 1}` if the array is empty.

If `default` is supplied, returns `default` instead of throwing.

Equivalent to `(arr-drop-back arr 1)` and `(arr-drop-back arr 1 default)`.

```pavo
(assert-eq (arr-init [true false 42]) [true false])
(assert-throw (arr-init []) { :tag :err-index, :got 1})
(assert-eq (arr-init [] nil) nil)
``` -->

from-set, from-map-keys, from-map-vals, from-map-entries, from-map-zipped

### Assertions

#### `(assert x)`

Returns `nil` if `x` is truthy (neither `nil` nor `false`), throws `{ :tag :err-assert }` otherwise.

Equivalent to `(sf-if x nil (sf-throw {:tag :err-assert }))`.

```pavo
(assert-not (assert 42))
(assert-throw (assert false) { :tag :err-assert })
(assert-throw (assert nil) { :tag :err-assert })
```

#### `(assert-not x)`

Returns `nil` if `x` is falsey (either `nil` or `false`), throws `{ :tag :err-assert }` otherwise.

Equivalent to `(sf-if x (sf-throw {:tag :err-assert }) nil)`.

```pavo
(assert-not (assert-not nil))
(assert-not (assert-not false))
(assert-throw (assert-not 42) { :tag :err-assert, :got 42})
```

#### `(assert-eq x y)`

Returns `nil` if `x`is equal to `y`, throws `{ :tag :err-assert }` otherwise.

Equivalent to `(assert (= x y))`.

```pavo
(assert-not (assert-eq nil nil))
(assert-throw (assert-eq nil false) { :tag :err-assert })
```

#### `(assert-type type x)`

Throws a type error unless `x` has the type `type`. Use this function if you want to report type errors like the toplevel functions do.

Equivalent to `(sf-if (= (typeof x) type) nil (sf-throw {:tag :err-type, :expected type, :got (typeof x)}))`.

```pavo
(assert-not (assert-type 42 :int))
(assert-throw (assert-type 42 :float) { :tag :err-type, :expected :float, :got :int})
(assert-throw (assert-type 42 43) { :tag :err-type, :expected 43, :got :int}) # Please don't do this.
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