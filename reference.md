# The Pavo Language Reference

This document serves as the reference description of the pavo programming language. Reading it is *not* the recommended way to *learn* the language, since it aims to be exhaustive rather than pedagocical, and it primarily deals with the *what* and *how* of pavo, not the *why*. Still, care has been taken to write it such that all aspects of the language are introduced before they are being referred to.

Pavo is a [homoiconic](https://en.wikipedia.org/wiki/Homoiconicity), [dynamically typed](https://en.wikipedia.org/wiki/Type_system#Dynamic_type_checking_and_runtime_type_information) but otherwise rather static [lisp](https://en.wikipedia.org/wiki/Lisp_(programming_language)) in the tradition of [scheme](https://en.wikipedia.org/wiki/Scheme_(programming_language)) and [clojure](https://clojure.org/), with [deterministic semantics](https://en.wikipedia.org/wiki/Deterministic_algorithm). Running a program consists of the following steps:

1. The source code is *parsed* into a value.
2. Macro *expansion* is performed on that value.
3. A number of *static checks* guarantees that the obtained value is a valid pavo program.
4. The program value is *evaluated* into the final result.

## Values

Values are the entities that the pavo programming language manipulates. Programming is about telling the machine how to derive new values from old ones. While hardware typically only knows about zeros and ones, pavo presents a more high-level interface to the programmer. The set of pavo values consists of the following things:

- *nil*: the [unit type](https://en.wikipedia.org/wiki/Unit_type)
- *bools*: [truth values](https://en.wikipedia.org/wiki/Boolean_data_type) `true` and `false`
- *ints*: [signed 64 bit integers](https://en.wikipedia.org/wiki/Integer_(computer_science))
- *floats*: [64 bit IEEE 754 floating point numbers](https://en.wikipedia.org/wiki/IEEE_754), excluding negative zero, not-a-number and the infinities
- *characters*: [unicode scalar values](http://www.unicode.org/glossary/#unicode_scalar_value)
- *strings*: [sequences](https://en.wikipedia.org/wiki/Sequence) of characters whose utf-8 encoding does not exceed 2^63 - 1 bytes
- *bytes*: sequences of unsigned bytes whose length does not exceed 2^63 - 1 bytes
- *arrays*: sequences of up to 2^63 - 1 values
- *sets*: [sets](https://en.wikipedia.org/wiki/Set_(mathematics)) of up to 2^63 - 1 values
- *maps*: [maps](https://en.wikipedia.org/wiki/Map_(mathematics)) (relations that are total functions) from values to values, containing up to 2^63 - 1 entry pairs
- *keywords*: a set of values that denote themselves and that can be created from and turned into strings
- *identifiers*: another set of values that denote theselves and that can be created from and turned into strings
- *symbols*: a set of values whose core property is that a symbol is equal to nothing but itself
- *functions*: a value that tells the computer how to perform a computation
- *cells*: storage locations that contain exactly one value that may change across time
- *opaques*: values with no observable properties, but usually they come with functions that can operate on them

Most of these become clearer through the defintion of their corresponding expressions and the operations that are available to manipulate them.

## Syntax

Pavo input must be valid utf8.

The [syntax](https://en.wikipedia.org/wiki/Syntax_(programming_languages)) of pavo consists of only two categories: Whitespace and expressions.

Whitespace serves to separate expressions and to aid readability, but has no effect on the semantics of a program. The following tokens are considered whitespace:

- unicode code point 0x20 space
- unicode code point 0x09 horizontal tab (\\t)
- unicode code point 0x0A line feed (\\n)
- unicode code point 0x0D carriage return (\\r)
- `,` (comma)
- a sequence of characters beginning with `#` up until a line feed or the end of the source code, called a comment

Expressions are parsed into values. Some values have exactly one corresponding expression, some have multiple, and some have none at all. A pavo file consists of any amount of whitespace followed by exactly one expression, followed by any amount of whitespace.

The expressions are:

### Nil

`nil`, parses into the `nil` value

### Bool

`true` and `false`, the literals for the `bool` values

### Identifier

Identifiers, a sequence of at least one and at most 255 of the following characters: `!*+-_?~<>=/\&|` or ascii alphanumerics. A sequence of more than 255 such characters is a parse error. Additionally, that sequence may not match the syntax of any other expression (such as `nil`, `true`, `false`, valid or overflowing integers, valid or overflowing floats).

```pavo
!
=P
-_-
!*+-_?~<>=/\&|abcdefghijklmnopqrsstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ
abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefg
# too long: abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh
```

### Keyword

A keyword consists of a colon (`:`) followed by at least one and at most 255 of the following characters: `!*+-_?~<>=/\&|` or ascii alphanumerics. A sequence of more than 255 such characters is a parse error.

```pavo
:!
:nil # while not an identifier, this is ok for a keyword
:!*+-_?~<>=/\&|abcdefghijklmnopqrsstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ
:abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefg
# too long: abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh
```

### Int

Integer literals, either decimal or hexadecimal. They consist of an optional sign (`+` or `-`), followed by an optional hex indicator `0x`, followed by at one or more digits (`0-9`, also `a-fA-F` if the hex indicator was present). The digits must form a number in the range from `-(2^63)` to `(2^63) - 1`, otherwise it is a parse error (e.g. `-9999999999999999999999` is *not* a valid expression even though it matches the syntax of an identifier).

```pavo
0
-0
+0
00
01
-9223372036854775808
# too small: -9223372036854775809 (must *not* parse as an identifier)
9223372036854775807
# too large: 9223372036854775808
(assert-eq 0xa 10)
(assert-eq -0xF -15)
```

### Float

Float literals denote pavo floats. A literal is parsed as if it were a true rational number, and is then [rounded](https://en.wikipedia.org/wiki/IEEE_754#Rounding_rules) to the nearest IEEE 754 64 bit float ([round to nearest, ties to even](https://en.wikipedia.org/wiki/Rounding#Round_half_to_even)), except that negative zero becomes positive zero. A float literal that rounds to an infinity is a parse error (e.g. `-999E999` is *not* a valid expression even though it matches the syntax of an identifier).

A float literal consists of an an optional sign (`+` or `-`), followed by one or more decimal digits, followed by a dot (`.`), followed by one or more decimal digits. This sequence may then be optionally followed by an exponent indicator (`e` or `E`), an optional sign (`+` or `-`) and one or more decimal digits.

The exponent indicator expresses [scientific notation](https://en.wikipedia.org/wiki/Scientific_notation), the leading floating point number is multiplied with ten to the power of the exponent. E.g. `5E-3` is the rational number `5 * (10 ^ -3) = 0.005`.

```pavo
0.0
-0.0
+0.0
00.00
(assert-eq -0.3e2 -30.0)
(assert-eq 0.3E+2 30.0)
(assert-eq 10.0e-2 0.1)
(assert-eq 0.0e99999999 0.0)
(assert-eq 9007199254740992.0 9007199254740993.0) # both round to the same float
# too small: -999E999 (must *not* parse as an identifier)
# too large: 999E999
```

### Character

Character literals denote [unicode scalar values](http://www.unicode.org/glossary/#unicode_scalar_value). For all characters other than `'` (`0x27`) and `\` (`0x5c`), a `'` followed by the character followed by another `'` denotes that character. In addition, there are escape sequences:

- `\'` for the char `'` (`0x27`)
- `\\` for the char `\` (`0x5c`)
- `\t` for the char `horizontal tab` (`0x09`)
- `\n` for the char `new line` (`0x0a`)
- `\{DIGITS}`, where `DIGITS` is the ASCII hexadecimal (`0-9a-fA-F`) representation of any unicode scalar value (one to six digits).

Literals or escape sequences that do not correspond to a Unicode scalar value are a parse error.

```pavo
'a'
'⚗'
'\{10FFFF}'
(assert-eq '\{61}' 'a')
(assert-eq '\{000061}' 'a')
(assert-eq '\{2697}' '⚗')
(assert-eq '"' '\{22}')
(assert-eq '\'' '\{27}')
(assert-eq '\\' '\{5c}')
(assert-eq '\t' '\{09}')
(assert-eq '\n' '\{0a}')
# surrogates are not scalar values: '\{D800}' to '\{DBFF}' and '\{DC00}' to '\{DFFF}' are parse errors
# as are escape sequences that are larger than the highest code point: '\{110000}' is a parse error
# escape sequences can not be too short or long: '\{}' and '\{1234567}' are parse errors
# ''' and '\' don't work, they must be escaped
# '\r' is not a valid escape sequence, so it's a parse error
```

### String

String literals denote sequences of up to 2^63 - 1 [unicode scalar values](http://www.unicode.org/glossary/#unicode_scalar_value). The syntax for a string literal consists of a `"` (`0x22`), followed by any number of character encodings (see next paragraph), followed by another `"` (`0x22`).

ach character can either be encoded literally or through an escape sequence. The literal encoding can be used for all scalar values other than `"` (`0x22`) and `\` (`0x5c`) and consists of the utf-8 encoding of the scalar value. Alternatively, any of the following escape sequences can be used:

- `\"` for the character `"` (`0x22`)
- `\\` for the character `\` (`0x5c`)
- `\t` for the character `horizontal tab` (`0x09`)
- `\n` for the character `new line` (`0x0a`)
- `\{DIGITS}`, where `DIGITS` is the ASCII hexadecimal (`0-9a-fA-F`) representation of any unicode scalar value (one to six digits).

Strings that contain a literal or an escape sequence that does not correspond to a unicode scalar value are a parse error. In particular, unicode code points that are not scalar values are not allowed, even when they form valid surrogate pairs.

```pavo
""
"a"
"abc"
"⚗"
"⚗\{10FFFF}\{10FFFF} \\foo"
"\{10FFFF}"
(assert-eq "\{61}" "a")
(assert-eq "\{000061}" "a")
(assert-eq "\{2697}" "⚗")
(assert-eq "'" '\{27}')
(assert-eq "\"" "\{22}")
(assert-eq "\\" "\{5c}")
(assert-eq "\t" "\{09}")
(assert-eq "\n" "\{0a}")
# surrogates are not scalar values: "\{D800}" to "\{DBFF}" and "\{DC00}" to "\{DFFF}" are parse errors
# as are escape sequences that are larger than the highest code point: "\{110000}" is a parse error
# escape sequences can not be too short or long: "\{}" and "\{1234567}"" are parse errors
# """ and "\" don't work, they must be escaped
# "\r" is not a valid escape sequence, so it's a parse error
```

If the size of the resulting string in utf-8 bytes exceeds 2^63-1, it is a parse error. Also you are doing some really weird stuff and should probably use a different tool for the job than pavo.

### Bytes

Bytes denote a string of arbitrary bytes. A byte literal consists of `@[`, followed by zero or more whitespace tokens, followed by any number of byte tokens (see next paragraph) that are separated by at least one whitespace token, followed by zero or more whitespace tokens, followed by `]`.

A byte token is either a sequence of one to three decimal digits of numeric value below 256, or `0x` followed by one or two hexadecimal digits (`0-9a-fA-F`).

```pavo
@[]
@[0]
@[0,0]
@[0xF]
@[   ,, 0xfE   ]
@[0, 001, 255]
@[1 0x1]
# @[1111] is too long, so a parse error, as is @[0001]
# @[256] is too large, so a parse error
# @[0x] is too short, @[0xddd] is too long
# @[10x1] is a parse error and distinct from @[1 0x1]
```

If the number of bytes exceeds 2^63-1, it is a parse error. Also you are doing some really weird stuff and should probably use a different tool for the job than pavo.

### Array

An array literal consists of `[`, followed by zero or more whitespace tokens, followed by any number of expressions that are separated by at least one whitespace token, followed by zero or more whitespace tokens, followed by `]`. It parses to the array value containing the values to which the inner expression parsed in sequence.

```pavo
[]
[0]
[0,1]
[ 0, 1  ,,2 ]
[[0],1,]
[1 :a]
[[] []]
# [1a], [1:a] and [[][]] are parse errors, the inner expressions must be separated by whitespace
```

If the number of inner expressions exceeds 2^63-1, it is a parse error. Also you are doing some really weird stuff and should probably use a different tool for the job than pavo.

### Application

An application literal consists of `(`, followed by zero or more whitespace tokens, followed by any number of expressions that are separated by at least one whitespace token, followed by zero or more whitespace tokens, followed by `)`. It parses to the application value containing the values to which the inner expression parsed in sequence.

```pavo
()
(0)
(0,1)
( 0, 1  ,,2 )
((0),1,)
(1 :a)
(() ())
# (1a), (1:a) and (()()) are parse errors, the inner expressions must be separated by whitespace
```

If the number of inner expressions exceeds 2^63-1, it is a parse error. Also you are doing some really weird stuff and should probably use a different tool for the job than pavo.

### Set

A set literal consists of `@{`, followed by zero or more whitespace tokens, followed by any number of expressions that are separated by at least one whitespace token, followed by zero or more whitespace tokens, followed by `}`. It parses to the set value containing the values to which the inner expression parsed (duplicates are thrown away).

```pavo
@{}
@{0}
@{0,1}
@{ 0, 1  ,,2 }
@{@{0},1,}
@{1 :a}
@{@{} @{}}
(assert-eq @{0 1} @{1 0})
(assert-eq @{0} @{0 0})
(assert-eq @{0} @{0 0x0})
# @{1a}, @{1:a} and @{@{}@{}} are parse errors, the inner expressions must be separated by whitespace
```

If the number of inner expressions exceeds 2^63-1 (before eliminating duplicates), it is a parse error. Also you are doing some really weird stuff and should probably use a different tool for the job than pavo.

### Map

A map literal consists of `{`, followed by zero or more whitespace tokens, followed by any number of entries (two values separated by at least one whitespace token) that are separated by at least one whitespace token, followed by zero or more whitespace tokens, followed by `}`. It parses to the map value containing the values to which the inner entries parsed. The first value of an entry is the key, and the second value the associated value. In case of duplicate keys, the rightmost entry is used.

```pavo
{}
{0 0}
{ 0,1 ,2 3 }
(assert-eq {0 1 2 3} {2 3 0 1})
(assert-eq {0 1 0 2 1 3 0 4} {0 4 1 3})
# {1a}, {1:a} and {{}{}} are parse errors, the inner expressions must be separated by whitespace
# {1} and {1 2 3} are parse errors, the number of inner expressions must be even
```

If the number of inner entries exceeds 2^63-1 (before eliminating entries with duplicate keys), it is a parse error. Also you are doing some really weird stuff and should probably use a different tool for the job than pavo.

### Syntactic Sugar

The previous expressions have all been *literals*. There are five more expressions that serve as shorthands for commonly used literals:

- A dollar sign `$` followed by an expression `exp` is parsed to the same value as `(quote exp)`
- A backtick `\`` followed by an expression `exp` is parsed to the same value as `(quasiquote exp)`
- A semicolon `;` followed by an expression `exp` is parsed to the same value as `(:unquote exp)`
- A percent sign `%` followed by an expression `exp` is parsed to the same value as `(:unquote-splice exp)`
- An at sign `@` followed by an identifier `id` is parsed to the same value as `(:fresh-name id)`

```pavo
(assert-eq (sf-quote $a) (sf-quote (quote a)))
(assert-eq (sf-quote `a) (sf-quote (quasiquote a)))
(assert-eq (sf-quote ;a) (sf-quote (:unquote a)))
(assert-eq (sf-quote %a) (sf-quote (:unquote-splice a)))
(assert-eq (sf-quote @a) (sf-quote (:fresh-name a)))
(assert-eq (sf-quote $$a) (sf-quote (quote (quote a))))
# $ by itself is a parse error (same for the other shorthands)
# $ 0 is a parse error, no whitespace allowed (same for the other shorthands)
# @0, @:a, @nil, @true, @false and @0a are parse errors, @ can only precede an identifier
```

## Static Checks

Before evaluating any pavo value in an environment, some static checks are performed first. This can be done in batch: Even though the evaluation of a value is defined in terms of the values it contains, only a single run of static checks on the initial value needs to be performed up-front. Unlike evaluation, the static checks are decidable.

There are two classes of static checks: Those enforcing the syntactic well-formedness of special forms, and those enforcing that all binding usages are correct. If a value to be evaluated violates any of these constraints, it can not be evaluated (a pavo compiler would signal a compile error, an interpreter would refuse to interpret the program).

### Special Form Syntax

Special forms are application literals whose first item is one of the following identifiers: `sf-quote`, `sf-do`, `sf-if`, `sf-set!`, `sf-throw`, `sf-try`, and `sf-lambda`. Checking special form syntax of a value proceeds recursively.

If the value is an identifier, symbol, nil, bool, int, float, char, string, bytes, keyword, function, cell or opaque, the check finishes successfully. To check an array, set, or map in an environment `E`, all inner values are checked. When checking an application, if it is a special form, it must satisfy the criteria outlined below. Then, if it is not a `sf-quote` form, all inner values are checked.

```pavo
(sf-quote (sf-if)) # this is ok - the sf-if is malformed, but that's ok because it is quoted
```

#### `sf-quote`

An application literal whose first item is `sf-quote` must have exactly two items.

```pavo
(sf-quote :quoted)
# (sf-quote) and (sf-quote foo bar) are static errors
```

#### `sf-do`

Any application literal whose first item is `sf-do` is well-formed.

#### `sf-if`

An application literal whose first item is `sf-quote` must have exactly two items.

```pavo
(sf-if :cond :then :else)
# (sf-if), (sf-if :cond), (sf-if :cond :then) and (sf-if :cond :then :else :wut?) are static errors
```

#### `sf-set!`

An application literal whose first item is `sf-set!` must have exactly three items, and the middle one must be a name (an identifier or symbol).

```pavo
(sf-lambda [(:mut a)] (sf-set! a 42))
# (sf-set! a 42) is syntactically valid, but a static error due to binding problems
# (sf-set! 42 43) is a static error because the second item must be a name
# (sf-set!), (sf-set! a) and (sf-set! a 42 foo) are static errors
```

#### `sf-throw`

An application literal whose first item is `sf-throw` must have exactly two items.

```pavo
(sf-throw :thrown)
# (sf-throw) and (sf-throw foo bar) are static errors
```

#### `sf-try`

An application literal whose first item is `(sf-try)` must have exactly four items, the third of which is either a name or a two-element application containing the keyword `:mut` followed by a name.


```pavo
(sf-try 0 a 1)
(sf-try 0 (:mut a) 1)
# (sf-try 0 1 2) is a static error because the third item must be a name or application
# (sf-try 0 (:mut 1) 2)
# (sf-try 0 (:foo b) 2)
# (sf-try 0 (:mut a))
# (sf-try), (sf-try 0), (sf-try 0 a), (sf-try) and (sf-try 0 a 1 2) are static errors
```

#### `sf-lambda`

An application literal whose first item is `(sf-lambda)` must have exactly three items, the second of which is either a name, a two-element application containing the keyword `:mut` followed by a name, or an array that contains any number of either names or such two-element applications.

```pavo
(sf-lambda a 0)
(sf-lambda (:mut a) 0)
(sf-lambda [] 0)
(sf-lambda [a] 0)
(sf-lambda [(:mut a)] 0)
(sf-lambda [a b] 0)
(sf-lambda [(:mut a) b] 0)
(sf-lambda [a (:mut b)] 0)
(sf-lambda [(:mut a) (:mut b)] 0)
(sf-lambda [a a] 0)
# (sf-lambda 0 1) is a static error because the second item must be a name, array or :mut
# (sf-lambda (:mut 0) 1) is a static error because the third item must be a name
# (sf-lambda [0] 1) is a static error because the array may not contain arbitrary values
# (sf-lambda [(:mut)] 0) is a static error because each :mut must correspond to an identifier
# (sf-lambda [(:mut a b)] 0)
# (sf-lambda [(a :mut)] 0) is a static error because each :mut must precede its identifier
# (sf-lambda), (sf-lambda a), (sf-lambda a 0 1), (sf-lambda :mut), (sf-lambda :mut a), (sf-lambda :mut a 0 1), (sf-lambda []) and (sf-lambda [] 0 1) are static errors
```

### Binding Correctness

Binding rules govern the usage of names (identifiers and symbols). The static checking of a value occurs in the context of a *check-environment*. A check-environment is a [partial function](https://en.wikipedia.org/wiki/Partial_function) (mathematically, not a pavo function) from names to booleans. A name that is mapped to false is called an *immutable binding*, a name that is mapped to true is called a *mutable binding*, and a name that is not mapped to anything is called a *free name*. By default, the initial check-environment used for checking a value contains exactly the values listed in section `Toplevel Values`, all of them mapped to false.

Checking bindings for a value proceeds recursively. If the value is a name (identifier or symbol), and that name is free in the check-environment, that is a static error. Bound names, nil, bools, ints, floats, chars, strings, bytes, keywords, functions, cells and opaques are always ok. To check an array, set, or map in a check-environment `E`, all inner values are checked in the check-environment `E`. The interesting case is checking an application in a check-environment `E`. The exact behavior depends on the application:

- `(sf-quote <quoted-exp>)`: always results in a successful check.
- `(sf-set! <target-name> <rvalue-exp>)`: Is a static error if the `<target-name>` is not a mutable binding, otherwise `<rvalue-exp>` is checked in the check-environment `E`.
- `(sf-try <try-exp> <binder-name> <caught-exp>)`: Check `<try-exp>` in the check-environment `E`. If successful, check `<caught-exp>` in the check-environment that behaves like `E` except that it maps `<binder-name>` to false.
- `(sf-try <try-exp> (:mut <binder-name>) <caught-exp>)`: Check `<try-exp>` in the check-environment `E`. If successful, check `<caught-exp>` in the check-environment that behaves like `E` except that it maps `<binder-name>` to true.
- `(sf-lambda <binder-name> <body-exp>)`: Check `<body-exp>` in the check-environment that behaves like `E` except that it maps `<binder-name>` to false.
- `(sf-lambda (:mut <binder-name>) <body-exp>)`: Check `<body-exp>` in the check-environment that behaves like `E` except that it maps `<binder-name>` to true.
- `(sf-lambda <args-array> <body-exp>)`: Check `<body-exp>` in the check-environment that behaves like `E` except that all names directly contained in the `<args-array>` map to false, and those inside an application with the `:mut` keyword map to `true`. For duplicate names, the mutability of the rightmost one is used.
- Otherwise, all inner values are checked in the environment `E`.

```pavo
(sf-quote a)
(sf-try 0 a a)
(sf-try 0 (:mut a) (sf-set! a 42))
(sf-lambda a a)
(sf-lambda (:mut a) (sf-set! a 42))
(sf-lambda a (sf-lambda (:mut a) (sf-set! a 0)))
(sf-lambda [a (:mut a)] (sf-set! a 42))
# some-id, [some-id] and (sf-set! some-id 0) are static errors because the name is not bound
# (sf-set! int-max-val 42), (sf-try 0 a (sf-set! a 42)) and (sf-lambda a (sf-set! a 42)) are static errors because the name is bound immutably
# (sf-lambda [(:mut a) a] (sf-set! a 42)) is a static error because the name is bound immutably
```

## Evaluation

Evaluation takes an input value and an environment of mutable and immutable name-to-value bindings, and yields either the successfully computed resulting value, or an error that was thrown.

The following values evaluate to themselves: nil, bools, ints, floats, chars, strings, bytes, keywords, functions, cells and opaques.

Evaluation of an array consists of evaluating the contained values in iteration order. If the evaluation of an inner value throws an error, the evaluation of the array stops by throwing that same error. If no inner evaluation errors, the overall evaluation yields the array containing the evaluation results in the same order.

```pavo
(assert-throw [(sf-throw :b) (sf-throw :a)] :b)
```

Evaluation of a set consists of evaluating the contained values in iteration order. If the evaluation of an inner value throws an error, the evaluation of the set stops by throwing that same error. If no inner evaluation errors, the overall evaluation yields the set containing the evaluation results.

Note that the iteration order of a set might not be the same as the order in which the items occur in the source code (i.e. the syntax):

```pavo
(assert-throw @{(sf-throw :b) (sf-throw :a)} :a)
```

Evaluation of a map consists of evaluating the contained entries in iteration order. For each entry, the key is evaluated first, then the value. If an inner evaluation throws an error, the evaluation of the map stops by throwing that same error. If no inner evaluation errors, the overall evaluation yields the map containing the evaluated entries.

Note that the iteration order of a map might not be the same as the order in which the items occur in the source code (i.e. the syntax):

```pavo
(assert-throw {:b (sf-throw 1), :a (sf-throw 0)} 0)
(assert-throw {(sf-throw :b) 42, (sf-throw :a) 42} :a)
```

Names (identifiers and symbols) evaluate to the value to which they are bound in the current environment. Unbound names never get evaluated, the static checks prevent evaluation in such a case.

Applications are where the magic happens. If the application contains zero items, the evaluation stops immediately by throwing `{:tag :err-lookup :got 0}`. If the first item of an application is the identifier of a special form, application proceeds as described in the section on that particular special form. Otherwise, the contained values are evaluated in iteration order. If the evaluation of an inner value throws an error, the evaluation of the application stops by throwing that same error. If no inner evaluation errors, the type of the evaluation result of the first item is checked. If it is not a function, the evaluation stops by throwing `{:tag :err-type, :expected :function, :got <the_actual_type_as_a_keyword>}`. Otherwise, the function is applied to the remaining results.

```pavo
(assert-throw ((sf-throw :b) (sf-throw :a)) :b)
(assert-throw () {:tag :err-lookup :got 0})
(assert-throw (42) {:tag :err-type, :expected :function, :got :int})
```

### Special Forms

Special forms form the backbone of pavo's evaluation, by providing crucial ways of deviating from the regular evaluation rules. Special form syntax is statically checked, so when evaluating a special form, it always matches one of the cases described below.

#### `(sf-quote x)`

Evaluates to the literal value denoted by `x`, without evaluating `x`.

```pavo
(assert-eq (sf-quote 42) 42)
(sf-quote foo) # Does not throw since the identifier is never evaluated
(assert-eq (typeof (sf-quote foo) :identifier))
(sf-quote ()) # Does not throw since the application is never evaluated
(assert-eq (typeof (sf-quote ()) :application))
(assert-not (= (sf-quote (int-add 1 1)) 2))
```

#### `(sf-do exp*)`

Evaluates the expressions in sequence, evaluating to the value of the last expression. If there are zero expressions, evaluates to `nil`.

```pavo
(assert-eq (sf-do) nil)
(assert-eq (sf-do 1) 1)
(assert-eq (sf-do 1 2 3) 3)
```

#### `(sf-if condition then else)`

Evaluates the `condition`. If it evaluated to `nil` or `false`, evaluates to the value of `else`, otherwise to the value of `then`. At most one of `then` and `else` is getting evaluated (or none if evaluating `condition` throws).

```pavo
(assert-eq (sf-if true :then :else) :then)
(assert-eq (sf-if 0 :then :else) :then)
(assert-eq (sf-if [] :then :else) :then)
(assert-eq (sf-if (sf-quote ()) :then :else) :then)
(assert-eq (sf-if nil :then :else) :else)
(assert-eq (sf-if false :then :else) :else)
```

#### `(sf-set! id exp)`

Evaluates `exp` and rebinds identifier `id` to the value. `id` must refer to a mutable binding in scope. The form itself evaluates to `nil`. If evaluating `exp` throws, the identifier is not rebound.

```pavo
(assert-eq ((sf-lambda [(:mut a)] (sf-do (sf-set! a 42) a)) 17) 42)
(assert-eq ((sf-lambda [(:mut a)] (sf-set! a 42)) 17) nil)
```

#### `(sf-throw x)`

Evaluates `x` and throws the result.

```pavo
(assert-throw (sf-throw 0) 0)
(assert-throw (sf-do
    0
    (sf-throw 1)
    (sf-throw 2)
    3
) 1)
(assert-throw (sf-if
    (sf-throw 0)
    (sf-throw 1)
    (sf-throw 2)
) 0)
```

#### `(sf-try try-exp id caught-exp)` `(sf-try try-exp (:mut id) caught-exp)`

Evaluates the `try-exp`. If it throws, the thrown value is bound to the `id` and then the `caught-exp` is evaluated. If the keyword `:mut` is supplied, the binding is mutable.

```pavo
(assert-eq (sf-try 0 foo 1) 0)
(assert-eq (sf-try (sf-throw 0) foo 1) 1)
(assert-eq (sf-try (sf-throw 0) (:mut foo) (sf-set! foo 1)) nil)
(assert-eq (sf-try (sf-throw 0) foo foo) 0)
(assert-throw (sf-try (sf-throw 0) foo (sf-throw 1)) 1)
```

#### `(sf-lambda id body)` `(sf-lambda (:mut id) body)` `(sf-lambda [<args>] body)`

Evaluates to a function. Associated with that function is the current environment, i.e. the same set of variable bindings that are in scope at the program point where the `sf-lambda` form occurs. When applying the function, the environment is modified according to the arguments (see below), and then the `body` expression is evaluated in that environment. The bindings introduced through application to arguments shadow any bindings of the same identifier that have been in lexical scope at the point of the function definition.

In the `(sf-lambda id body)` and `(sf-lambda (:mut id) body)` versions, when applying the function to some argument values, the identifier `id` is bound to an array containing those values before evaluating the `body`. If the keyword `:mut` is supplied, the binding is mutable.

```pavo
(assert-eq (typeof (sf-lambda foo nil)) :function))
(assert-eq ((sf-lambda foo foo) 0 1 2) [0 1 2])
(assert-eq ((sf-lambda (:mut foo) (sf-do (sf-set! foo 42) foo)) 0 1 2) 42)
```

In the `(sf-lambda [<args>] body)` version, when applying the function to some arguments, if the number of arguments does not match the number of identifiers in the function definition, `{ :tag :err-num-args, :expected <defined>, :got <supplied>}` is thrown, where `<defined>` is the number of identifiers in the `[<args>]` definition, and `<supplied>` is the number of arguments to which the function was applied. If the number of arguments matches, then each argument identifier is bound to the corresponding supplied value before evaluating the `body`. For identifiers that are prefixed with the `:mut` keyword, the binding is mutable. In case of duplicate identifiers, the rightmost one wins.

```pavo
(assert-eq (typeof (sf-lambda [] nil)) :function)
(assert-eq ((sf-lambda [] 42)) 42)
(assert-throw ((sf-lambda [] 42) :an-argument) {:tag :err-num-args, :expected 0, :got 1})
(assert-eq ((sf-lambda [a b] (int-add a b)) 1 2) 3)
(assert-eq ((sf-lambda [a (:mut b)] (sf-do (sf-set! b 3) (int-add a b))) 1 2) 4)
(assert-eq ((sf-lambda [a a] a) 0 1) 1)
```

Pavo guarantees tail-call optimization, (mutually) recursive function calls in tail position only take up a bounded amount of stack space. The tail positions are exactly the following:

- the body of a function
- the last expression of an `sf-do` form that is in tail position
- the `then` and `else` expressions of an `sf-if` form that is in tail position
- the `caught-exp` of an `sf-try` form that is in tail position

## Macro Expansion

Macro expansion turns a value into a different value, usually before it is checked and evaluated.

TODO

## Toplevel Macros

TODO

## Toplevel Values

These are all the values that are bound to an identifier in the default toplevel environment. All of these bindings are immutable.

The given time complexities on functions are the minimum that a pavo implementation must provide. An implementation is free to guarantee *better* complexity bounds than those required. In particular, any amortized complexity bound can be implemented as non-amortized. The converse is not true: If a complexity requirement is unamortized, then implementations are not allowed to provide only amortized bounds.

Whenever a function is described to "throw a type error", it throws a map `{ :tag :err-type, :expected <expected>, :got <got>}` where `<expected>` and `<got>` are the keywords denoting the respective types (see `(typeof x)`). Type errors are also trown when an argument is described as having a certain type, but an argument of a different type is supplied. For example "Do foo to the int `n`" throws a type error with `:expected :int` if `n` is not an int.

Whenever an argument is referred to as a "positive int", but an int less than zero is supplied, the function throws `{ :tag :err-negative, :got <got>}`, where `<got>` is the supplied, negative int.

If a function is invoked with an incorrect number of args, it throws a map `{ :tag :err-num-args, expected: <expected>, :got <got>}`, where `<expected>` is the number of arguments the function expected to take, and `got` is the number of arguments that were supplied.

The precedence of errors is as follows: First, the number of arguments is checked, then the supplied arguments are checked in sequence. Checking an argument means first checking the type, and then any additional properties (such as non-negativity).

```pavo
(assert-throw (bool-not) { :tag :err-num-args, :expected 1, :got 0 })
(assert-throw (bool-not 42 43) { :tag :err-num-args, :expected 1, :got 2 })
(assert-throw (bool-not 42) { :tag :err-type, :expected :bool, :got :int })
(assert-throw (int-pow-wrap :nope "nope") { :tag :err-type, :expected :int, :got :keyword})
(assert-throw (int-pow-wrap 2 :nope) { :tag :err-type, :expected :int, :got :keyword})
(assert-throw (int-pow-wrap 2 -2) { :tag :err-negative, :got -2})
```

### Bool

Bools are binary [truth values](https://en.wikipedia.org/wiki/Truth_value), either `true` or `false`.

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

Ints are [signed 64 bit integers](https://en.wikipedia.org/wiki/Integer_(computer_science)) represented in [two's complement](https://en.wikipedia.org/wiki/Two's_complement), that is numbers between `-2^63` and and `2^63 - 1` inclusive. Because of their finite width and the inherent asymmetry of two's complement representation, the functions operating on integers often have cornercases. The "default" functions (`int-add` and friends) throw an error when reaching the boundaries of the numeric representation. Others (`int-add-wrap` etc. and `int-add-sat` etc.) allow the programmer to embrace the limits of the finite representation. Not caring about those limits at all however usually leads to bad surprises.

Most of the function (documentation) has been taken/adapted from the [rust i64 docs](https://doc.rust-lang.org/std/primitive.i64.html). A helpful discussion of various design choices for the behavior of the modulus and division operations is [Boute, Raymond T. "The Euclidean definition of the functions div and mod."](https://biblio.ugent.be/publication/314490/file/452146.pdf).

#### `int-max-val`

The largest integer, `2^63 - 1`.

```pavo
(assert-eq int-max-val 9223372036854775807)
(assert-throw (int-add int-max-val 1) { :tag :err-wrap-int })
```

#### `int-min-val`

The smallest integer, `- 2^63`.

```pavo
(assert-eq int-min-val -9223372036854775808)
(assert-throw (int-sub int-min-val 1) { :tag :err-wrap-int })
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
(assert-eq (int-leading-ones -4611686018427387904) 2)
```

#### `(int-leading-zeros n)`

Returns the number of leading zeros in the binary representation of the int `n`.

```pavo
(assert-eq (int-leading-zeros 13) 60)
```

#### `(int-trailing-ones n)`

Returns the number of trailing ones in the binary representation of the int `n`.

```pavo
(assert-eq (int-trailing-ones 3) 2)
```

#### `(int-trailing-zeros n)`

Returns the number of trailing zeros in the binary representation of the int `n`.

```pavo
(assert-eq (int-trailing-zeros 4) 2)
```

#### `(int-rotate-left n by)`

Shifts the bits of the int `n` to the left by the amount `by`, wrapping the truncated bits to the end of the resulting int.

```pavo
(assert-eq (int-rotate-left 0xaa00000000006e1 12) 0x6e10aa)
```

#### `(int-rotate-right n by)`

Shifts the bits of the int `n` to the right by the positive int `by`, wrapping the truncated bits to the beginning of the resulting int.

```pavo
(assert-eq (int-rotate-right 0x6e10aa 12) 0xaa00000000006e1)
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
(assert-throw (int-add int-max-val 1) { :tag :err-wrap-int })
```

#### `(int-sub n m)`

Subtracts the int `m` from the int `n`.

Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-sub 1 2) -1)
(assert-eq (int-sub 1 -2) 3)
(assert-throw (int-sub int-min-val 1) { :tag :err-wrap-int })
```

#### `(int-mul n m)`

Multiplies the int `n` with the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-mul 2 3) 6)
(assert-eq (int-mul 2 -3) -6)
(assert-throw (int-mul int-max-val 2) { :tag :err-wrap-int })
```

#### `(int-div n m)`

Divides the int `n` by the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow. Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the quotient of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-div 8 3) 2)
(assert-eq (int-div -8 3) -3)
(assert-throw (int-div int-min-val -1) { :tag :err-wrap-int })
(assert-throw (int-div 1 0) { :tag :err-zero })
```

#### `(int-div-trunc n m)`

Divides the int `n` by the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow. Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the quotient of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-div-trunc 8 3) 2)
(assert-eq (int-div-trunc -8 3) -2)
(assert-throw (int-div-trunc int-min-val -1) { :tag :err-wrap-int })
(assert-throw (int-div-trunc 1 0) { :tag :err-zero })
```

#### `(int-mod n m)`

Computes the int `n` modulo the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow. Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the remainder of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-mod 8 3) 2)
(assert-eq (int-mod -8 3) 1)
(assert-throw (int-mod int-min-val -1) { :tag :err-wrap-int })
(assert-throw (int-mod 1 0) { :tag :err-zero })
```

#### `(int-mod-trunc n m)`

Computes the int `n` modulo the int `m`.

Throws `{ :tag :err-wrap-int }` in case of an overflow. Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the remainder of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-mod-trunc 8 3) 2)
(assert-eq (int-mod-trunc -8 3) -2)
(assert-throw (int-mod-trunc int-min-val -1) { :tag :err-wrap-int })
(assert-throw (int-mod-trunc 1 0) { :tag :err-zero })
```

#### `(int-neg n)`

Negates the int `n`.Throws `{ :tag :err-wrap-int }` in case of an overflow.

```pavo
(assert-eq (int-neg 42) -42)
(assert-eq (int-neg -42) 42)
(assert-eq (int-neg 0) 0)
(assert-throw (int-neg int-min-val) { :tag :err-wrap-int })
```

#### `(int-shl n m)`

Performs a [logical left shift](https://en.wikipedia.org/wiki/Logical_shift) of the int `n` by the positive int `m` many bits. This always results in `0` if `m` is greater than `63`.

```pavo
(assert-eq (int-shl 5 1) 10)
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
(assert-throw (int-abs int-min-val) { :tag :err-wrap-int })
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
(assert-eq (int-add-sat int-max-val 1) int-max-val)
(assert-eq (int-add-sat int-min-val -1) int-min-val)
```

#### `(int-sub-sat n m)`

Subtracts the int `n` from the int `m`, saturating at the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-sub-sat 1 2) -1)
(assert-eq (int-sub-sat 1 -2) 3)
(assert-eq (int-sub-sat int-min-val 1) int-min-val)
(assert-eq (int-sub-sat int-max-val -1) int-max-val)
```

#### `(int-mul-sat n m)`

Multiplies the int `n` with the int `m`, saturating at the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-mul-sat 2 3) 6)
(assert-eq (int-mul-sat 2 -3) -6)
(assert-eq (int-mul-sat int-max-val 2) int-max-val)
(assert-eq (int-mul-sat int-min-val 2) int-min-val)
```

#### `(int-pow-sat n m)`

Computes the int `n` to the power of the positive int `m`, saturating at the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-pow-sat 2 3) 8)
(assert-eq (int-pow-sat 2 0) 1)
(assert-eq (int-pow-sat 0 999) 0)
(assert-eq (int-pow-sat 1 999) 1)
(assert-eq (int-pow-sat -1 999) -1)
(assert-eq (int-pow-sat 99 99) int-max-val)
(assert-eq (int-pow-sat -99 99) int-min-val)
```

#### `(int-add-wrap n m)`

Adds the int `n` to the int `m`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-add-wrap 1 2) 3)
(assert-eq (int-add-wrap int-max-val 1) int-min-val)
(assert-eq (int-add-wrap int-min-val -1) int-max-val)
```

#### `(int-sub-wrap n m)`

Subtracts the int `n` from the int `m`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-sub-wrap 1 2) -1)
(assert-eq (int-sub-wrap int-min-val 1) int-max-val)
(assert-eq (int-sub-wrap int-max-val -1) int-min-val)
```

#### `(int-mul-wrap n m)`

Muliplies the int `n` with the int `m`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-mul-wrap 2 3) 6)
(assert-eq (int-mul-wrap int-max-val 2) -2)
(assert-eq (int-mul-wrap int-max-val -2) 2)
(assert-eq (int-mul-wrap int-min-val 2) 0)
(assert-eq (int-mul-wrap int-min-val -2) 0)
```

#### `(int-div-wrap n m)`

Divides the int `n` by the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the quotient of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-div-wrap 8 3) 2)
(assert-eq (int-div-wrap -8 3) -3)
(assert-eq (int-div-wrap int-min-val -1) int-min-val)
(assert-throw (int-div-wrap 1 0) { :tag :err-zero })
```

#### `(int-div-trunc-wrap n m)`

Divides the int `n` by the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the quotient of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-div-trunc-wrap 8 3) 2)
(assert-eq (int-div-trunc-wrap -8 3) -2)
(assert-eq (int-div-trunc-wrap int-min-val -1) int-min-val)
(assert-throw (int-div-trunc-wrap 1 0) { :tag :err-zero })
```

#### `(int-mod-wrap n m)`

Computes the int `n` modulo the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the remainder of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-mod-wrap 8 3) 2)
(assert-eq (int-mod-wrap -8 3) 1)
(assert-eq (int-mod-wrap int-min-val -1) 0)
(assert-throw (int-mod-wrap 1 0) { :tag :err-zero })
```

#### `(int-mod-trunc-wrap n m)`

Computes the int `n` modulo the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{ :tag :err-zero }` if `m` is `0`.

This computes the remainder of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-mod-trunc-wrap 8 3) 2)
(assert-eq (int-mod-trunc-wrap -8 3) -2)
(assert-eq (int-mod-trunc-wrap int-min-val -1) 0)
(assert-throw (int-mod-trunc-wrap 1 0) { :tag :err-zero })
```

#### `(int-neg-wrap n)`

Negates the int `n`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-neg-wrap 42) -42)
(assert-eq (int-neg-wrap -42) 42)
(assert-eq (int-neg-wrap 0) 0)
(assert-eq (int-neg-wrap int-min-val) int-min-val)
```

#### `(int-abs-wrap n)`

Returns the absolute value of the int `n`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-abs-wrap 42) 42)
(assert-eq (int-abs-wrap -42) 42)
(assert-eq (int-abs-wrap 0) 0)
(assert-eq (int-abs-wrap int-min-val) int-min-val)
```

#### `(int-pow-wrap n m)`

Computes the int `n` to the power of the positive int `m`, wrapping around the numeric bounds instead of overflowing.

```pavo
(assert-eq (int-pow-wrap 2 3) 8)
(assert-eq (int-pow-wrap 2 0) 1)
(assert-eq (int-pow-wrap 0 999) 0)
(assert-eq (int-pow-wrap 1 999) 1)
(assert-eq (int-pow-wrap -1 999) -1)
(assert-eq (int-pow-wrap 99 99) -7394533151961528133)
(assert-throw (int-pow-wrap 2 -1) {:tag :err-negative :got -1 })
```

#### `(int-signum n)`

Returns `-1` if the int `n` is less than `0`, `0` if `n` is equal to `0`, `1` if `n` is greater than `0`.

```pavo
(assert-eq (int-signum -42) -1)
(assert-eq (int-signum 0) 0)
(assert-eq (int-signum 42) 1)
```

### Bytes

Bytes are sequences of unsigned 8 bit integers, that is integers between `0` and `255` inclusive. They serve as a more efficient alternative to arrays containing ints.

Whenever a function takes a "byte" as an argument but is given a non-int argument, a type error is thrown. If it is an int but it is not between zero and 255, an `err-not-byte` is thrown.

```pavo
(assert-throw (bytes-insert @[] 0 :256) { :tag :err-type, :expected :int, :got :keyword})
(assert-throw (bytes-insert @[] 0 256) { :tag :err-not-byte, :got 256})
```

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
(assert-throw (bytes-get @[] 0) { :tag :err-lookup, :got 0})
```

#### `(bytes-insert b index new)`

Inserts the byte `new` into the bytes `b` at the index int `index`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.
Throws `{ :tag :err-not-byte, :got new}` if `new` is not a byte (an int between 0 and 255 inclusive).
Throws `{ :tag :err-collection-full }` if the resulting bytes would contain 2^63 or more elements.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-insert @[0 1] 0 42) @[42 0 1])
(assert-eq (bytes-insert @[0 1] 1 42) @[0 42 1])
(assert-eq (bytes-insert @[0 1] 2 42) @[0 1 42])
(assert-throw (bytes-insert @[0 1] 3 42) { :tag :err-lookup, :got 3})
(assert-throw (bytes-insert @[] 0 256) { :tag :err-not-byte, :got 256})
    (assert-throw (bytes-insert @[] 0 :256) { :tag :err-type, :expected :int, :got :keyword})
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
(assert-throw (bytes-update @[0] 0 256) { :tag :err-not-byte, :got 256})
```

#### `(bytes-slice b start end)`

Returns a subsequence of the bytes `b`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{ :tag :err-lookup, :got end}` if `start` is greater than `end`.
Throws `{ :tag :err-lookup, :got start}` if `start` is out of bounds.
Throws `{ :tag :err-lookup, :got end}` if `end` is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-slice @[42 43] 1 1) @[])
(assert-eq (bytes-slice @[42 43] 0 1) @[42])
(assert-eq (bytes-slice @[42 43] 1 2) @[43])
(assert-eq (bytes-slice @[42 43] 0 2) @[42 43])
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
(assert-eq (bytes-splice @[0 1] 0 @[10 11]) @[10 11 0 1])
(assert-eq (bytes-splice @[0 1] 1 @[10 11]) @[0 10 11 1])
(assert-eq (bytes-splice @[0 1] 2 @[10 11]) @[0 1 10 11])
(assert-throw (bytes-splice @[0 1] 3 @[10 11]) { :tag :err-lookup, :got 3})
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
(let (:mut product) 1 (do
    (bytes-iter @[1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (bytes-iter @[1 2 3 4] (fn [elem] (sf-if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 2)
))
(assert-throw (bytes-iter @[0 1] (fn [b] (throw b))) 0)
```

#### `(bytes-iter-back b fun)`

Starting from the back of the bytes `b`, applies the function `fun` to the elements of `b` in reverse order until either `fun` returns a truthy value or the end of the bytes is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(bytes-count b)`.

```pavo
(let (:mut product) 1 (do
    (bytes-iter-back @[1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (bytes-iter-back @[1 2 3 4] (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 4)
))
(assert-throw (bytes-iter-back @[0 1] (fn [b] (throw b))) 1)
```

### Chars

A char is a [unicode scalar value](http://www.unicode.org/glossary/#unicode_scalar_value).

#### `char-max-val`

The largest char (numerically the largest unicode scalar value).

```pavo
(assert-eq char-max-val '\{10ffff}')
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

Strings are conceptually sequences of chars. Unless otherwise noted, operations in general (and indexing in particular) are defined in terms of chars. There are however a few functions that deal with raw bytes of the utf-8 encoding of a string, and the maximum size of strings is defined in term of its utf8 encoding (at most `2^63 - 1` bytes).

#### `(str-count s)`

Returns the number of chars in the string `s`.

Time: O(1).

```pavo
(assert-eq (str-count "") 0)
(assert-eq (str-count "a") 1)
(assert-eq (str-count "⚗") 1)
(assert-eq (str-count "abc") 3)
```

#### `(str-count-utf8 s)`

Returns the number of bytes in the utf8 encoding of the string `s`.

Time: O(1).

```pavo
(assert-eq (str-count-utf8 "") 0)
(assert-eq (str-count-utf8 "a") 1)
(assert-eq (str-count-utf8 "⚗") 3)
(assert-eq (str-count-utf8 "abc") 3)
```

#### `(str-get s index)`

Returns the char at the int `index` in the string `s`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-get "a" 0) 'a')
(assert-eq (str-get "⚗b" 1) 'b')
(assert-throw (str-get "" 0) { :tag :err-lookup, :got 0})
```

#### `(str-get-utf8 s index)`

Returns the utf8 byte at the int `index` (in bytes) in the string `s`.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

Time: O(log n), where n is `(str-count-utf8 s)`.

```pavo
(assert-eq (str-get-utf8 "a" 0) 'a')
(assert-eq (str-get-utf8 "⚗" 0) 226)
(assert-eq (str-get-utf8 "⚗" 1) 154)
(assert-eq (str-get-utf8 "⚗" 2) 151)
(assert-throw (str-get-utf8 "" 0) { :tag :err-lookup, :got 0})
```

#### `(str-index-char->utf8 str index)`

Finds the character at the int `index` in the string `s`, and returns at which byte index it begins.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

```pavo
(assert-eq (str-index-char->utf8 "a" 0) 0)
(assert-eq (str-index-char->utf8 "ab" 1) 1)
(assert-eq (str-index-char->utf8 "⚗b" 1) 3)
(assert-throw (str-index-char->utf8 "" 0) { :tag :err-lookup, :got 0})
```

#### `(str-index-utf8->char str index)`

Finds the utf8 byte at the int `index` (in bytes) in the string `s`, and returns at which position (in characters) the character to which it belongs begins.

Throws `{ :tag :err-lookup, :got index}` if the index is out of bounds.

```pavo
(assert-eq (str-index-utf8->char "a" 0) 0)
(assert-eq (str-index-utf8->char "ab" 1) 1)
(assert-eq (str-index-utf8->char "⚗b" 1) 0)
(assert-eq (str-index-utf8->char "⚗b" 2) 0)
(assert-eq (str-index-utf8->char "⚗b" 3) 1)
(assert-throw (str-index-char->utf8 "" 0) { :tag :err-lookup, :got 0})
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
(assert-throw (str-remove "ab" 2) { :tag :err-lookup, :got 2})
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
(assert-eq (str-splice "ab" 0 "cd") "cdab")
(assert-eq (str-splice "ab" 1 "cd") "acdb")
(assert-eq (str-splice "ab" 2 "cd") "abcd")
(assert-throw (str-splice "ab" 3 "cd") { :tag :err-lookup, :got 3})
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
(let (:mut out) "z" (do
    (str-iter "abcd" (fn [elem] (set! out (str-insert out 0 elem))))
    (assert-eq out "dcbaz")
))
(let (:mut out) "z" (do
    (str-iter "abcd" (fn [elem] (if
            (= elem 'c') true
            (set! out (str-insert out 0 elem))
        )))
    (assert-eq out "baz")
))
(assert-throw (str-iter "ab" (fn [c] (throw c))) 'a')
```

#### `(str-iter-back s fun)`

Starting from the back of the string `s`, applies the function `fun` to the chars of `s` in reverse order until either `fun` returns a truthy value or the end of the string is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(str-count s)`.

```pavo
(let (:mut out) "z" (do
    (str-iter-back "abcd" (fn [elem] (set! out (str-insert out 0 elem))))
    (assert-eq out "abcdz")
))
(let (:mut out) "z" (do
    (str-iter-back "abcd" (fn [elem] (if
            (= elem 'c') true
            (set! out (str-insert out 0 elem))
        )))
    (assert-eq out "dz")
))
(assert-throw (str-iter-back "ab" (fn [c] (throw c))) 'b')
```

#### `(str-iter-utf8 s fun)`

Starting from the beginning of the string `s`, applies the function `fun` to the utf8 bytes of `s` in sequence until either `fun` returns a truthy value or the end of the string is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(str-count-utf8 s)`.

```pavo
(let (:mut product) 1 (do
    (str-iter-utf8 "abc" (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 941094)
))
(let (:mut product) 1 (do
    (str-iter-utf8 "abc" (fn [elem] (sf-if
            (= elem 98) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 97)
))
(assert-throw (str-iter-utf8 "abc" (fn [b] (throw b))) 97)
```

#### `(str-iter-utf8-back s fun)`

Starting from the back of the string `s`, applies the function `fun` to the utf8 bytes of `s` in reverse order until either `fun` returns a truthy value or the end of the string is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(str-count-utf8 s)`.

```pavo
(let (:mut product) 1 (do
    (str-iter-utf8-back "abc" (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 941094)
))
(let (:mut product) 1 (do
    (str-iter-utf8-back "abc" (fn [elem] (sf-if
            (= elem 98) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 99)
))
(assert-throw (str-iter-utf8-back "abc" (fn [b] (throw b))) 99)
```

### Floats

Floats are IEEE 754 64 bit floating point numbers, except that there are no NaNs, no infinities, and no negative zero. All operations are carried out as per the standard, that is the results are determined as if the numbers had infinite precision, and are then rounded to the nearest representable number. The rounding mode is always [round to nearest, ties to even](https://en.wikipedia.org/wiki/Rounding#Round_half_to_even). If an operation would produce positive infinity under regular IEEE 754 semantcis (regardless whether by definition or through rounding), the corresponding pavo function throws `:inf` instead. Analogously `:-inf` is thrown instead of returning negative infinity, and `:nan` is thrown instead of returning any NaN. Any operation that would produce negative zero produces positive zero instead.

The examples in this section are not very exhaustive, since all operations are well-defined in the standard.

#### `float-max-val`

The largest float.

```pavo
(assert-eq float-max-val 1.7976931348623157e308)
(assert-throw (float-mul float-max-val 2.0) :inf)
```

#### `float-min-val`

The smallest float.

```pavo
(assert-eq float-min-val -1.7976931348623157e308)
(assert-throw (float-mul float-min-val 2.0) :-inf)
```

#### `(float-add x y)`

Adds the float `x` to the float `y`.

```pavo
(assert-eq (float-add 1.0 2.0) 3.0)
(assert-eq (float-add 1.0 -2.0) -1.0)
(assert-eq (float-add 0.1 0.2) 0.30000000000000004)
```

#### `(float-sub x y)`

Subtracts the float `y` from the float `x`.

```pavo
(assert-eq (float-sub 1.0 2.0) -1.0)
(assert-eq (float-sub 1.0 -2.0) 3.0)
```

#### `(float-mul x y)`

Multiplies the float `x` with the int `y`.

```pavo
(assert-eq (float-mul 2.0 3.0) 6.0)
(assert-eq (float-mul 2.0 -3.0) -6.0)
```

#### `(float-div x y)`

Divides the float `x` by the float `y`.

```pavo
(assert-eq (float-div 8.0 3.0) 2.6666666666666665)
(assert-throw (float-div 1.0 0.0) :inf)
(assert-throw (float-div 1.0 -0.0) :inf) # non :-inf, since -0.0 is positive zero
(assert-throw (float-div 0.0 0.0) :nan)
```

#### `(float-mul-add x y z)`

Computes `x * y + z` on the floats `x`, `y` and `z` with perfect precision and then rounds the result to the nearest float. This is more accurate than (and thus not equivalent to) `(float-add (float-mul x y) z)`. Also known as fused multiply-add.

```pavo
(assert-eq (float-mul-add 1.2 3.4 5.6) 9.68)
```

#### `(float-neg x)`

Negates the float `x`. Zero remains unchanged.

```pavo
(assert-eq (float-neg 1.2) -1.2)
(assert-eq (float-neg -1.2) 1.2)
(assert-eq (float-neg 0.0) 0.0)
```

#### `(float-floor x)`

Returns the largest integral float less than or equal to the float `x`.

```pavo
(assert-eq (float-floor 1.9) 1.0)
(assert-eq (float-floor 1.0) 1.0)
(assert-eq (float-floor -1.1) -2.0)
```

#### `(float-ceil x)`

Returns the smallest integral float greater than or equal to the float `x`.

```pavo
(assert-eq (float-ceil 1.1) 2.0)
(assert-eq (float-ceil 1.0) 1.0)
(assert-eq (float-ceil -1.9) -1.0)
```

#### `(float-round x)`

Rounds the float `x` towards the nearest integral float, rounding towards the even one in case of a tie.

```pavo
(assert-eq (float-round 1.0) 1.0)
(assert-eq (float-round 1.49) 1.0)
(assert-eq (float-round 1.51) 2.0)
(assert-eq (float-round 1.5) 2.0)
(assert-eq (float-round 2.5) 2.0)
```

#### `(float-trunc x)`

Returns the integer part of the float `x` as a float.

```pavo
(assert-eq (float-trunc 1.0) 1.0)
(assert-eq (float-trunc 1.49) 1.0)
(assert-eq (float-trunc 1.51) 1.0)
(assert-eq (float-trunc -1.51) -1.0)
```

#### `(float-fract x)`

Returns the fractional part of the float `x` (negative for negative `x`).

```pavo
(assert-eq (float-fract 1.0) 0.0)
(assert-eq (float-fract 1.49) 0.49)
(assert-eq (float-fract 1.51) 0.51)
(assert-eq (float-fract -1.51) -0.51)
```

#### `(float-abs x)`

Returns the absolute value of the float `x`.

```pavo
(assert-eq (float-abs 1.2) 1.2)
(assert-eq (float-abs -1.2) 1.2)
(assert-eq (float-abs 0.0) 0.0)
(assert-eq (float-abs -0.0) 0.0)
```

#### `(float-signum x)`

Returns `1.0` if the float `x` is greater than zero, `0.0` if it is equal to zero, `-1.0` if it is less than zero.

```pavo
(assert-eq (float-signum 99.2) 1.0)
(assert-eq (float-signum -99.2) -1.0)
(assert-eq (float-signum 0.0) 0.0)
(assert-eq (float-signum -0.0) 0.0)
```

#### `(float-pow x y)`

Raises the float `x` to the power of the float `y`.

```pavo
(assert-eq (float-pow 1.2 3.4) 1.858729691979481)
```

#### `(float-sqrt x)`

Computes the square root of the float `x`.

```pavo
(assert-eq (float-sqrt 1.2) 1.0954451150103321)
(assert-throw (float-sqrt -1.0) :nan)
```

#### `(float-exp x)`

Returns [e](https://en.wikipedia.org/wiki/E_(mathematical_constant)) to the power of the float `x`.

```pavo
(assert-eq (float-exp 1.2) 3.3201169227365472)
```

#### `(float-exp2 x)`

Returns 2.0 to the power of the float `x`.

```pavo
(assert-eq (float-exp2 1.2) 2.2973967099940698)
```

#### `(float-ln x)`

Returns the [natural logarithm](https://en.wikipedia.org/wiki/Natural_logarithm) of the float `x`.

```pavo
(assert-eq (float-ln 1.2) 0.1823215567939546)
```

#### `(float-log2 x)`

Returns the [binary logarithm](https://en.wikipedia.org/wiki/Binary_logarithm) of the float `x`.

```pavo
(assert-eq (float-log2 1.2) 0.2630344058337938)
```

#### `(float-log10 x)`

Returns the base 10 logarithm of the float `x`.

```pavo
(assert-eq (float-log10 1.2) 0.07918124604762482)
```

#### `(float-hypot x y)`

Calculates the length of the hypotenuse of a right-angle triangle given legs of the float lengths `x` and `y`.

```pavo
(assert-eq (float-hypot 1.2 3.4) 3.605551275463989)
(assert-eq (float-hypot 1.2 -3.4) 3.605551275463989)
(assert-eq (float-hypot -1.2 3.4) 3.605551275463989)
(assert-eq (float-hypot -1.2 -3.4) 3.605551275463989)
```

#### `(float-sin x)`

Computes the sine of the float `x` (in radians).

```pavo
(assert-eq (float-sin 1.2) 0.9320390859672263)
```

#### `(float-cos x)`

Computes the cosine of the float `x` (in radians).

```pavo
(assert-eq (float-cos 1.2) 0.3623577544766736)
```

#### `(float-tan x)`

Computes the tangent of the float `x` (in radians).

```pavo
(assert-eq (float-tan 1.2) 2.5721516221263188)
```

#### `(float-asin x)`

Computes the arcsine of the float `x`, in radians in the range `[-pi/2, pi/2]`. Throws `:nan` if `x` is outside the range `[-1, 1]`.

```pavo
(assert-eq (float-asin 0.8) 0.9272952180016123)
(assert-throw (float-asin 1.2) :nan)
```

#### `(float-acos x)`

Computes the arccosine of the float `x`, in radians in the range `[-pi/2, pi/2]`. Throws `:nan` if `x` is outside the range `[-1, 1]`.

```pavo
(assert-eq (float-acos 0.8) 0.6435011087932843)
(assert-throw (float-acos 1.2) :nan)
```

#### `(float-atan x)`

Computes the arctangent of the float `x`, in radians in the range `[-pi/2, pi/2]`.

```pavo
(assert-eq (float-atan 1.2) 0.8760580505981934)
```

#### `(float-atan2 x y)`

Computes [atan2](https://en.wikipedia.org/wiki/Atan2) of the float `x` and the float `y`.

```pavo
(assert-eq (float-atan2 1.2 3.4) 0.3392926144540447)
```

#### `(float-exp-m1 x)`

Returns [e](https://en.wikipedia.org/wiki/E_(mathematical_constant)) to the power of the float `x`, minus one, i.e. `(e^x) - 1`.

```pavo
(assert-eq (float-exp-m1 1.2) 2.3201169227365472)
```

#### `(float-ln-1p x)`

Returns the [natural logarithm](https://en.wikipedia.org/wiki/Natural_logarithm) of one plus the float `x`, i.e. `ln(1 + x)`

```pavo
(assert-eq (float-ln-1p 1.2) 0.7884573603642702)
```

#### `(float-sinh x)`

Computes the [hyperbolic sine](https://en.wikipedia.org/wiki/Hyperbolic_function) of the float `x`.

```pavo
(assert-eq (float-sinh 1.2) 1.5094613554121725)
```

#### `(float-cosh x)`

Computes the [hyperbolic cosine](https://en.wikipedia.org/wiki/Hyperbolic_function) of the float `x`.

```pavo
(assert-eq (float-cosh 1.2) 1.8106555673243747)
```

#### `(float-tanh x)`

Computes the [hyperbolic tangent](https://en.wikipedia.org/wiki/Hyperbolic_function) of the float `x`.

```pavo
(assert-eq (float-tanh 1.2) 0.8336546070121552)
```

#### `(float-asinh x)`

Computes the [inverse hyperbolic sine](https://en.wikipedia.org/wiki/Inverse_hyperbolic_functions) of the float `x`.

```pavo
(assert-eq (float-asinh 1.2) 1.015973134179692)
```

#### `(float-acosh x)`

Computes the [inverse hyperbolic cosine](https://en.wikipedia.org/wiki/Inverse_hyperbolic_functions) of the float `x`.

```pavo
(assert-eq (float-acosh 1.2) 0.6223625037147785)
```

#### `(float-atanh x)`

Computes the [inverse hyperbolic tangent](https://en.wikipedia.org/wiki/Inverse_hyperbolic_functions) of the float `x`.

```pavo
(assert-eq (float-atanh 0.8) 1.0986122886681098)
(assert-throw (float-atanh 1.2) :nan)
```

#### `(float-normal? x)`

Returns `true` if the float `x` is neither zero nor [subnormal](https://en.wikipedia.org/wiki/Denormal_number), false otherwise.

```pavo
(assert-eq (float-normal? 1.0) true)
(assert-eq (float-normal? 1.0e-308) false) # subnormal
(assert-eq (float-normal? 0.0) false)
```

#### `(float->degrees x)`

Converts the float `x` from radians to degrees.

```pavo
(assert-eq (float->degrees 1.2) 68.75493541569878)
```

#### `(float->radians x)`

Converts the float `x` from degrees to radians.

```pavo
(assert-eq (float->radians 1.2) 0.020943951023931952)
```

#### `(float->int x)`

Converts the float `x` to an int, rounding towards zero.

```pavo
(assert-eq (float->int 0.0) 0)
(assert-eq (float->int 1.0) 1)
(assert-eq (float->int -1.0) -1)
(assert-eq (float->int 1.9) 1)
(assert-eq (float->int -1.9) -1)
(assert-eq (float->int float-max-val) int-max-val)
(assert-eq (float->int float-min-val) int-min-val)
```

#### `(int->float n)`

Converts the int `n` to a float, using the usual rounding rules if it can not be represented exactly ([round to nearest, ties to even](https://en.wikipedia.org/wiki/Rounding#Round_half_to_even)).

```pavo
(assert-eq (int->float 0) 0.0)
(assert-eq (int->float 1) 1.0)
(assert-eq (int->float -1) -1.0)
(assert-eq (int->float 9007199254740993) 9007199254740992.0)
(assert-eq (int->float -9007199254740993) -9007199254740992.0)
```

#### `(float->bits x)`

Returns the int with the same bit pattern as the bit pattern of the float `x`. The sign bit of the float is the most significant bit of the resulting int, the least significant bit of the float's mantissa is the least significant bit of the resulting int.

```pavo
(assert-eq (float->bits 1.2) 4608083138725491507)
(assert-eq (float->bits -1.2) -4615288898129284301)
(assert-eq (float->bits 0.0) 0)
(assert-eq (float->bits -0.0) 0)
```

#### `(bits=>float n)`

Returns the float with the same bit pattern as the bit pattern of the int `n`. The most significant bit of `n` is the sign bit of the float, the least significant bit of `n` is the least significant bit of the float's mantissa.

Throws `:nan`, `:inf` or `:-inf` if the bit pattern represents on of these in IEEE 754.

```pavo
(assert-eq (bits=>float 42) 2.08e-322)
(assert-throw (bits=>float -42) :nan)
(assert-throw (bits=>float 9218868437227405312) :inf)
(assert-throw (bits=>float -4503599627370496) :-inf)
```

#### `(bits=>float? n)`

Returns whether applying `bits=>float` to the float `n` would wrk (`true`) or throw an error (`false`).

```pavo
(assert-eq (bits=>float? 42) true)
(assert-eq (bits=>float? -42) false)
(assert-eq (bits=>float? 9218868437227405312) false)
(assert-eq (bits=>float? -4503599627370496) false)
```

### Identifiers

TODO bla

#### `(str=>id s)`

Returns an identifier created from the string `s`.

Throws `{ :tag :err-identifier, :got s}` if it would not be a valid identifier (empty, longer than 255 characters, or containing invalid characters). TODO other errors: nil, bools, numbers

Time: O(n) where n is `(str-count s)`.

```pavo
(assert-eq (str=>id "foo") $foo)
(assert-eq (str=>id "nil") $nil) XXX
(assert-eq (str=>id "42") $42) XXX
(assert-throw (str=>id "") { :tag :err-identifier, :got ""})
(assert-throw (str=>id "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") { :tag :err-identifier, :got "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789"})
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

TODO bla

#### `(str=>kw s)`

Returns the keyword `<:s>` created from the string `s`.

Throws `{ :tag :err-kw, :got s}` if it would not be a valid keyword (empty, longer than 255 characters, or containing invalid characters).

Time: O(n) where n is `(str-count s)`.

```pavo
(assert-eq (str=>kw "foo") :foo)
(assert-eq (str=>kw "nil") :nil)
(assert-eq (str=>kw "42") :42)
(assert-throw (str=>kw "") { :tag :err-kw, :got ""})
(assert-throw (str=>kw "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") { :tag :err-kw, :got "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789"})
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
(let (:mut product) 1 (do
    (arr-iter [1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (arr-iter [1 2 3 4] (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 2)
))
(assert-throw (arr-iter [0 1] (fn [n] (throw n))) 0)
```

#### `(arr-iter-back arr fun)`

Starting from the back of the array `arr`, applies the function `fun` to the elements of `arr` in reverse order until either `fun` returns a truthy value or the end of the array is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(arr-count arr)`.

```pavo
(let (:mut product) 1 (do
    (arr-iter-back [1 2 3 4] (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (arr-iter-back [1 2 3 4] (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 4)
))
(assert-throw (arr-iter-back [0 1] (fn [n] (throw n))) 1)
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
(let (:mut product) 1 (do
    (app-iter $(1 2 3 4) (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (app-iter $(1 2 3 4) (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 2)
))
(assert-throw (app-iter $(0 1) (fn [n] (throw n))) 0)
```

#### `(app-iter-back app fun)`

Starting from the back of the application `app`, applies the function `fun` to the elements of `app` in reverse order until either `fun` returns a truthy value or the end of the application is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(app-count app)`.

```pavo
(let (:mut product) 1 (do
    (app-iter-back $(1 2 3 4) (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (app-iter-back $(1 2 3 4) (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 4)
))
(assert-throw (app-iter-back $(0 1) (fn [n] (throw n))) 1)
```

#### `(app-apply app)`

Applies the first value in the application to the remaining values.

TODO wrap errors or not?
TODO examples

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
(let (:mut product) 1 (do
    (set-iter @{4 2 3 1} (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (set-iter @{4 2 3 1} (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 2)
))
(assert-throw (set-iter @{0 1} (fn [n] (throw n))) 0)
```

#### `(set-iter-back set fun)`

Starting from the maximal element of the set `set`, applies the function `fun` to the elements of `set` in descending order until either `fun` returns a truthy value or the end of the set is reached. Returns `nil`. Propagates any value thrown by `fun`.

Time: Iteration takes amortized O(n), where n is `(set-count set)`.

```pavo
(let (:mut product) 1 (do
    (set-iter-back @{4 2 3 1} (fn [elem] (set! product (int-mul product elem))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (set-iter-back @{4 2 3 1} (fn [elem] (if
            (= elem 3) true
            (set! product (int-mul product elem))
        )))
    (assert-eq product 4)
))
(assert-throw (set-iter-back @{0 1} (fn [n] (throw n))) 1)
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

#### `(map-iter map fun)`

Starting from the entry with the minimal key in the map `map`, applies the function `fun` to the entries of `map` in ascending order until either `fun` returns a truthy value or the end of the set is reached. Returns `nil`. Propagates any value thrown by `fun`.

Fun is passed the key first and the value second.

Time: Iteration takes amortized O(n), where n is `(map-count map)`.

```pavo
(let (:mut product) 1 (do
    (map-iter {4 2, 3 1} (fn [key value] (set! product (int-mul product (int-mul key value)))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (map-iter {4 2, 3 1} (fn [key value] (if
            (= key 3) true
            (set! product (int-mul product (int-mul key value)))
        )))
    (assert-eq product 3)
))
(assert-throw (map-iter {0 1, 2 3} (fn [n m] (throw (int-mul n m)))) 0)
```

#### `(map-iter-back map fun)`

Starting from the entry with the maximal key in the map `map`, applies the function `fun` to the entries of `map` in descending order until either `fun` returns a truthy value or the end of the set is reached. Returns `nil`. Propagates any value thrown by `fun`.

Fun is passed the key first and the value second.

Time: Iteration takes amortized O(n), where n is `(map-count map)`.

```pavo
(let (:mut product) 1 (do
    (map-iter-back {4 2, 3 1} (fn [key value] (set! product (int-mul product (int-mul key value)))))
    (assert-eq product 24)
))
(let (:mut product) 1 (do
    (map-iter-back {4 2, 3 1} (fn [key value] (if
            (= key 3) true
            (set! product (int-mul product (int-mul key value)))
        )))
    (assert-eq product 8)
))
(assert-throw (map-iter-back {0 1, 2 3} (fn [n m] (throw (int-mul n m)))) 6)
```

### Symbols

#### `(symbol)`

Returns a fresh symbol that is only equal to itself.

```pavo
(assert (let x (symbol) (= x x)))
(assert-not (= (symbol) (symbol)))
```

### Cells

TODO max number, ordering

#### `(cell x)`

Returns a new cell that contains the value `x`. The cell is only equal to itself.

```pavo
(assert (let x (cell 42) (= x x)))
(assert-not (= (cell 42) (cell 42)))
```

#### `(cell-get cl)`

Returns the value contained in the cell.

```pavo
(assert-eq (cell-get (cell 42)) 42)
```

#### `(cell-set cl x)`

Updates the cell `cl` to now contain the value `x`.

```pavo
(assert-eq ((sf-lambda [x] (sf-do (cell-set x 43) (cell-get x))) (cell 42)) 43)
```

### Opaques

#### `(opaque)`

Returns an array containing two functions: One that takes a value and returns an opaque value, and one that inverts it (but throws `{:tag :err-opaque}` if its input has not been produced by the first function).

TODO

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

TODO expand, check, eval, exval (expand then eval)

### Miscellaneous

#### `(typeof x)`

Returns a keyword indicating the type of `x`: `:nil`, `:bool`, `:int`, `:float`, `:char`, `:string`, `:bytes`, `:keyword`, `:identifier`, `:symbol`, `:function`, `:array`, `:application`, `:map`, `:set`, `:cell` or `:opaque`.

```pavo
(assert-eq (typeof nil) :nil)
(assert-eq (typeof true) :bool)
(assert-eq (typeof 42) :int)
(assert-eq (typeof 0.0) :float)
(assert-eq (typeof 'a') :char)
(assert-eq (typeof "foo") :string)
(assert-eq (typeof @[]) :bytes)
(assert-eq (typeof :kw) :keyword)
(assert-eq (typeof $id) :identifier)
(assert-eq (typeof (symbol)) :symbol)
(assert-eq (typeof typeof) :function)
(assert-eq (typeof (cell 42)) :cell)
(assert-eq (typeof []) :array)
(assert-eq (typeof $()) :application)
(assert-eq (typeof {}) :map)
(assert-eq (typeof @{}) :set)
```

TODO opaque

#### `(not x)`

Returns `true` if `x` is `nil` or `false`, and `false` otherwise.

Equivalent to `(if x false true)`.

```pavo
(assert (not nil))
(assert (not false))
(assert-not (not true)
(assert-not (not 0))
(assert-not (not falsey?))
```

#### `(diverge v)`

Immediately and abnormally terminates the execution of the program. Semantically you can think of this as going into an infinite loop, but telling the outside world about it to save resources and give feedback to the programmer. In the pavo semantics, passing the value `v` does nothing whatsoever, but the runtime should somehow pass this value to the outside world for additional context.

Note that this is different from a program terminating through an uncaught throw and you should almost always throw instead of deliberately calling `diverge` (just as there are very few situations where you'd deliberately go into an effect-free infinite loop).

## Appendix: Precise Definition of `(write v)`

TODO rewrite this, explain with recursion rather than induction...

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
- `(= (typeof v) :cell)`: throw `{ :tag :err-not-writable }`
- `(= (typeof v) :opaque)`: throw `{ :tag :err-not-writable }`

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
# TODO example with an opaque value here

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

---

TODO trace

TODO require (dynamic linking, *not* loading)

TODO syntax sugar

---

- functions that compute the builtin macros?

Macros:

- `set!`
- `quote`
- `throw`
- `if`
- `fn`
- `letfn`
- `let`
- `try`
- `do`
- `quasiquote`
- `->`, `->>`
- `and`, `or`
- `while`

- `dotimes` ?
- `case`, `loop` ?

letfn:

Defines some functions via `<fns>`, then evaluates to `exp` in an environment in which these functions are immutably bound to some names. `<fns>` is a map from identifiers to applications that contain the `[<args>]` and `body` like a regular `sf-lambda` definition. When evaluating one of those function bodies, all the functions defined by the `sf-letfn` form are immutably bound to their respective identifier.

```pavo
(assert-eq (sf-letfn {
    even? ([n] (
        sf-if
            (= n 0)
            true
            (odd? (- n 1))
        )),
    odd? ([n] (
        sf-if
            (= n 0)
            false
            (even? (- n 1))
        ))
} [(even? 10000) (odd? 10000)]) [true false])
```
