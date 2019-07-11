# The Pavo Language Reference

This document serves as the reference description of the pavo programming language. Reading it is *not* the recommended way to *learn* the language, since it aims to be exhaustive rather than pedagocical, and it primarily deals with the *what* and *how* of pavo, not the *why*. Still, care has been taken to write it such that all aspects of the language are introduced before they are being referred to.

Pavo is a [homoiconic](https://en.wikipedia.org/wiki/Homoiconicity), [dynamically typed](https://en.wikipedia.org/wiki/Type_system#Dynamic_type_checking_and_runtime_type_information) but otherwise rather static [lisp](https://en.wikipedia.org/wiki/Lisp_(programming_language)) in the tradition of [scheme](https://en.wikipedia.org/wiki/Scheme_(programming_language)) and [clojure](https://clojure.org/), with [deterministic semantics](https://en.wikipedia.org/wiki/Deterministic_algorithm). Running a program consists of the following steps:

1. The source code is *parsed* into a value.
2. Macro *expansion* is performed on that value.
3. A number of *static checks* guarantees that the obtained value is a valid pavo program.
4. The program value is *evaluated* into the final result.

**Status of this Document**: Apart from some macros, everything should be precisely specified. It's not particularly well-written, but it should be sufficiently precise (except for those macros...).

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

Identifiers, a sequence of at least one and at most 255 of the following characters: `!*+-_?.%<>=/\&|` or ascii alphanumerics. A sequence of more than 255 such characters is a parse error. Additionally, that sequence may not match the syntax of any other expression (such as `nil`, `true`, `false`, valid or overflowing integers, valid or overflowing floats).

```pavo
!
=P
-_-
!*+-_?.%<>=/\&|abcdefghijklmnopqrsstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ
abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefg
# too long: abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh
```

### Keyword

A keyword consists of a colon (`:`) followed by at least one and at most 255 of the following characters: `!*+-_?.%<>=/\&|` or ascii alphanumerics. A sequence of more than 255 such characters is a parse error.

```pavo
:!
:nil # while not an identifier, this is ok for a keyword
:!*+-_?.%<>=/\&|abcdefghijklmnopqrsstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ
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

In addition to regular string literals, there are *raw string literals* that don't have escape sequences. They begin with 1-8 `@` signs followed by `"` and extend until the first further `"` that is followed by as many `@` signs.

```pavo
(assert-eq @"no escape for inner " or \ needed"@ "no escape for inner \" or \\ needed")
(assert-eq @"\n"@ "\\n")
(assert-eq @"\{1234}"@ "\\{1234}")
(assert-eq @@@@""@@@@ "")
(assert-eq @@@@@@@@""@@@@@@@@ "")
(assert-eq @@@"@"@@"""@@@ "@\"@@\"\"")
# @@@@@@@@@"nope"@@@@@@@@@ too many @ signs
# @@@@@@@"nope"@@@@@@@@@ trailing @s are an error
```

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
- A tilde `~` followed by an expression `exp` is parsed to the same value as `(:unquote exp)`
- `@~` followed by an expression `exp` is parsed to the same value as `(:unquote-splice exp)`
- An at sign `@` followed by an identifier `id` is parsed to the same value as `(:fresh-name id)`

```pavo
(assert-eq (sf-quote $a) (sf-quote (quote a)))
(assert-eq (sf-quote `a) (sf-quote (quasiquote a)))
(assert-eq (sf-quote ~a) (sf-quote (:unquote a)))
(assert-eq (sf-quote @~a) (sf-quote (:unquote-splice a)))
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

An application literal whose first item is `sf-do` must have exactly two items, the second one must be an array.

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

An application literal whose first item is `(sf-lambda)` must have exactly three items, the second of which is an array that contains any number of either names or two-element applications containing the keyword `:mut` followed by a name.

```pavo
(sf-lambda [] 0)
(sf-lambda [a] 0)
(sf-lambda [(:mut a)] 0)
(sf-lambda [a b] 0)
(sf-lambda [(:mut a) b] 0)
(sf-lambda [a (:mut b)] 0)
(sf-lambda [(:mut a) (:mut b)] 0)
(sf-lambda [a a] 0)
# (sf-lambda 0 1) is a static error because the second item must an array
# (sf-lambda [0] 1) is a static error because the array may not contain arbitrary values
# (sf-lambda [(:mut)] 0) is a static error because each :mut must correspond to an identifier
# (sf-lambda [(:mut a b)] 0)
# (sf-lambda [(a :mut)] 0) is a static error because each :mut must precede its identifier
# (sf-lambda []) and (sf-lambda [] 0 1) are static errors
```

### Binding Correctness

Binding rules govern the usage of names (identifiers and symbols). The static checking of a value occurs in the context of a *check-environment*. A check-environment is a [partial function](https://en.wikipedia.org/wiki/Partial_function) (mathematically, not a pavo function) from names to booleans. A name that is mapped to false is called an *immutable binding*, a name that is mapped to true is called a *mutable binding*, and a name that is not mapped to anything is called a *free name*. By default, the initial check-environment used for checking a value contains exactly the values listed in section `Toplevel Values`, all of them mapped to false.

Checking bindings for a value proceeds recursively. If the value is a name (identifier or symbol), and that name is free in the check-environment, that is a static error. Bound names, nil, bools, ints, floats, chars, strings, bytes, keywords, functions, cells and opaques are always ok. To check an array, set, or map in a check-environment `E`, all inner values are checked in the check-environment `E`. The interesting case is checking an application in a check-environment `E`. The exact behavior depends on the application:

- `(sf-quote <quoted-exp>)`: always results in a successful check.
- `(sf-set! <target-name> <rvalue-exp>)`: Is a static error if the `<target-name>` is not a mutable binding, otherwise `<rvalue-exp>` is checked in the check-environment `E`.
- `(sf-try <try-exp> <binder-name> <caught-exp>)`: Check `<try-exp>` in the check-environment `E`. If successful, check `<caught-exp>` in the check-environment that behaves like `E` except that it maps `<binder-name>` to false.
- `(sf-try <try-exp> (:mut <binder-name>) <caught-exp>)`: Check `<try-exp>` in the check-environment `E`. If successful, check `<caught-exp>` in the check-environment that behaves like `E` except that it maps `<binder-name>` to true.
- `(sf-lambda <args-array> <body-exp>)`: Check `<body-exp>` in the check-environment that behaves like `E` except that all names directly contained in the `<args-array>` map to false, and those inside an application with the `:mut` keyword map to `true`. For duplicate names, the mutability of the rightmost one is used.
- Otherwise, all inner values are checked in the check-environment `E`.

```pavo
(sf-quote a)
(sf-try 0 a a)
(sf-try 0 (:mut a) (sf-set! a 42))
(sf-lambda [a] a)
(sf-lambda [(:mut a)] (sf-set! a 42))
(sf-lambda [a] (sf-lambda [(:mut a)] (sf-set! a 0)))
# some-id, [some-id] and (sf-set! some-id 0) are static errors because the name is not bound
# (sf-set! int-max-val 42), (sf-try 0 a (sf-set! a 42)) and (sf-lambda [a] (sf-set! a 42)) are static errors because the name is bound immutably
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

Names (identifiers and symbols) evaluate to the value to which they are bound in the current environment. It can not happen that an unbound name needs to be evaluated, the static checks prevent such situations from arising.

Applications are where the magic happens. If the application contains zero items, the evaluation stops immediately by throwing `{:tag :err-lookup}`. If the first item of an application is the identifier of a special form, application proceeds as described in the section on that particular special form. Otherwise, the contained values are evaluated in iteration order. If the evaluation of an inner value throws an error, the evaluation of the application stops by throwing that same error. If no inner evaluation errors, the type of the evaluation result of the first item is checked. If it is not a function, the evaluation stops by throwing `{:tag :err-type}`. Otherwise, the function is applied to the remaining results.

If a function is invoked with an incorrect number of args, it throws a map `{:tag :err-num-args}`.

```pavo
(assert-throw ((sf-throw :b) (sf-throw :a)) :b)
(assert-throw () {:tag :err-lookup})
(assert-throw (42) {:tag :err-type})
(assert-throw (int-add 1) {:tag :err-num-args})
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

#### `(sf-do [exprs...])`

Evaluates the expressions in the array in sequence, evaluating to the value of the last expression. If there are zero expressions, evaluates to `nil`.

```pavo
(assert-eq (sf-do []) nil)
(assert-eq (sf-do [1]) 1)
(assert-eq (sf-do [1 2 3]) 3)
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
(assert-eq ((sf-lambda [(:mut a)] (sf-do [(sf-set! a 42) a])) 17) 42)
(assert-eq ((sf-lambda [(:mut a)] (sf-set! a 42)) 17) nil)
```

#### `(sf-throw x)`

Evaluates `x` and throws the result.

```pavo
(assert-throw (sf-throw 0) 0)
(assert-throw (sf-do [
    0
    (sf-throw 1)
    (sf-throw 2)
    3
]) 1)
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

#### `(sf-lambda [args...] body)`

Evaluates to a function. Associated with that function is the current environment, i.e. the same set of variable bindings that are in scope at the program point where the `sf-lambda` form occurs. When applying the function, the environment is modified according to the arguments (see below), and then the `body` expression is evaluated in that environment. The bindings introduced through application to arguments shadow any bindings of the same identifier that have been in lexical scope at the point of the function definition.

When applying the function to some arguments, if the number of arguments does not match the number of args in the function definition, `{:tag :err-num-args}` is thrown. If the number of arguments matches, then each argument identifier is bound to the corresponding supplied value before evaluating the `body`. For identifiers that are prefixed with the `:mut` keyword, the binding is mutable. In case of duplicate identifiers, the rightmost one wins.

```pavo
(assert-eq (typeof (sf-lambda [] nil)) :function)
(assert-eq ((sf-lambda [] 42)) 42)
(assert-throw ((sf-lambda [] 42) :an-argument) {:tag :err-num-args})
(assert-eq ((sf-lambda [a b] (int-add a b)) 1 2) 3)
(assert-eq ((sf-lambda [a (:mut b)] (sf-do [(sf-set! b 3) (int-add a b)])) 1 2) 4)
(assert-eq ((sf-lambda [a a] a) 0 1) 1)
```

Pavo guarantees tail-call optimization, (mutually) recursive function calls in tail position only take up a bounded amount of stack space. The tail positions are exactly the following:

- the body of a function
- the last expression in the array of an `sf-do` form that is in tail position
- the `then` and `else` expressions of an `sf-if` form that is in tail position
- the `caught-exp` of an `sf-try` form that is in tail position

## Macro Expansion

Macro expansion turns a value into a different value, usually before it is checked and evaluated.

In addition to the value to be expanded, macro expansion needs two further pieces of information: a mapping from identifiers to values (the *macro-environment*) and a regular environment (mapping identifiers to values and mutabilities, the *definition-environment*). The basic idea is that each application that begins with an identifier in the macro-environment gets replaced with the result of applying the macro to the remaining arguments of the application. The definition-environment is the environment that is used in the definition of the macros.

The actual definition of the expansion process is inductive as usual: nil, bools, ints, floats, chars, strings, bytes, keywords, functions, cells and opaques remain unchanged. Arrays and sets are expanded by expanding the contained values in iteration order. Maps are expanded by expanding the contained entries in iteration order (key first, corresponding value second, repeat over all keys in ascending order). Apps are where the fun stuff happens.

If the first item of an application is the identifier `sf-quote`, the expansion of the application is the application itself.

If the first item of an application is an identifier other than `macro` or `sf-quote`, and if that identifier is a key in the macro-environment, the expansion of the application is obtained by applying the corresponding value in the macro environment to the items of the application (without the leading identifier). If the macro value is not a function, if the number of arguments is not correct, or if the function throws, the macro expansion errors.

If the first item of an application is the identifier `macro`, this defines a new macro. If the application does not have exactly four items, the macro expansion errors. The second item of the application must be either an identifier or a map whose entries may have arbitrary keys but the values must again be either identifiers or such a map. This item is called the *pattern*. The third item of the application can be any value. This value is evaluated in the definition-environment, the resulting value is called the *definition*. If it throws, the macro expansion errors. Otherwise, the expanded form of the application is the expanded form of the fourth item, using the same definition-environment, but a new macro-environment obtained as follows:

- start with the old macro-environment
- if the *pattern* is an identifier, update the macro-environment by mapping that identifier to the *definition*
- if the *pattern* is a map
  - if the *definition* is not a map, the macro expansion errors
  - else, for all entries in the current *pattern* (in iteration order):
    - if the *definition* does not contain the key, the macro expansion errors
    - if it does, treat the value of the *pattern*'s entry as a new pattern, and the corresponding value in the *definition* as a new definition, and recur

For all other applications, the expanded value is an application containing the expanded values of the items of the original application.

```pavo
(assert-eq (expand 42 {}) 42)
(assert-eq (expand $throw {}) $throw)
(assert-eq (expand $(sf-quote (macro)) {}) $(sf-quote (macro)))
(assert-eq (expand $(throw nil) {}) $(sf-throw nil))
(assert-eq (expand $(x y) {}) $(x y))
(assert-eq (expand $(x (throw nil)) {}) $(x (sf-throw nil)))
(assert-eq (expand (macro
    foo
    (sf-lambda [] 42)
    (foo)
    ) {}) 42)
(assert-eq (expand (macro
    { :foo foo, :bar {:baz baz}}
    { :foo (sf-lambda [] 42), :bar {:baz (sf-lambda [a] (int-add a 3))}, :zonk 42}
    [1, (foo), (baz 17)]
    ) {}) [1, 42, 20])
(assert-eq (expand (macro
    {:2 a, :1 {:9 a}}
    { :1 {:9 (sf-lambda [] :nope)}, :2 (sf-lambda [] :yup)}
    (a)
    ) {}) :yup)
(assert-throw (expand $(macro) {}) {:tag :err-expand})
(assert-throw (expand $(throw 1 2) {}) {:tag :err-expand})
(assert-throw (expand $(macro foo 42 (foo)) {}) {:tag :err-expand})
(assert-throw (expand $(macro {:foo foo} {} 42) {}) {:tag :err-expand})
```

## Toplevel Macros

There is a function named `macro-xyz` for each builtin macro `xyz` that is the implementation of the macro. For now, look up the documentation of these functions as documentation for the builtin macros.

## Toplevel Values

These are all the values that are bound to an identifier in the default toplevel environment. All of these bindings are immutable.

The given time complexities on functions are the minimum that a pavo implementation must provide. An implementation is free to guarantee *better* complexity bounds than those required. In particular, any amortized complexity bound can be implemented as non-amortized. The converse is not true: If a complexity requirement is unamortized, then implementations are not allowed to provide only amortized bounds.

Whenever a function is described to "throw a type error", it throws a map `{:tag :err-type}`. Type errors are also trown when an argument is described as having a certain type, but an argument of a different type is supplied. For example "Do foo to the int `n`" throws a type error if `n` is not an int.

Whenever an argument is referred to as a "positive int", but an int less than zero is supplied, the function throws `{:tag :err-negative}`.

If a function is invoked with an incorrect number of args, it throws a map `{:tag :err-num-args}`.

The precedence of errors is as follows: First, the number of arguments is checked, then the supplied arguments are checked in sequence. Checking an argument means first checking the type, and then any additional properties (such as non-negativity).

```pavo
(assert-throw (bool-not) {:tag :err-num-args})
(assert-throw (bool-not 42 43) {:tag :err-num-args})
(assert-throw (bool-not 42) {:tag :err-type})
(assert-throw (int-pow-wrap :nope "nope") {:tag :err-type})
(assert-throw (int-pow-wrap 2 :nope) {:tag :err-type})
(assert-throw (int-pow-wrap 2 -2) {:tag :err-negative})
```

### Bool

Bools are binary [truth values](https://en.wikipedia.org/wiki/Truth_value), either `true` or `false`.

#### `(bool-not b)`

Computes [logical negation](https://en.wikipedia.org/wiki/Negation) `¬b` on bools.

Throws a type error if the arguments is not a bool.

```pavo
(assert (bool-not false))
(assert-not (bool-not true))

(assert-throw (bool-not 0) {:tag :err-type})
```

#### `(bool-and b0 b1)`

Computes [logical conjunction](https://en.wikipedia.org/wiki/Logical_conjunction) `b0 ∧ b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert-not (bool-and false false))
(assert-not (bool-and false true))
(assert-not (bool-and true false))
(assert (bool-and true true))

(assert-throw (bool-and false 0) {:tag :err-type)
```

#### `(bool-or b0 b1)`

Computes [logical disjunction](https://en.wikipedia.org/wiki/Logical_disjunction) `b0 ∨ b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert-not (bool-or false false))
(assert (bool-or false true))
(assert (bool-or true false))
(assert (bool-or true true))

(assert-throw (bool-or true 1) {:tag :err-type})
```

#### `(bool-if b0 b1)`

Computes [logical implication](https://en.wikipedia.org/wiki/https://en.wikipedia.org/wiki/Material_conditional) `b0 → b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert (bool-if false false))
(assert (bool-if false true))
(assert-not (bool-if true false))
(assert (bool-if true true))

(assert-throw (bool-if false 1) {:tag :err-type})
```

#### `(bool-iff b0 b1)`

Computes [logical biimplication](https://en.wikipedia.org/wiki/Logical_biconditional) `b0 ↔ b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert (bool-iff false false))
(assert-not (bool-iff false true))
(assert-not (bool-iff true false))
(assert (bool-iff true true))

(assert-throw (bool-iff false 1) {:tag :err-type})
```

#### `(bool-xor b0 b1)`

Computes [logical exclusive disjunction](https://en.wikipedia.org/wiki/Exclusive_or) `b0 ⊕ b1` on bools.

Throws a type error if any of the arguments is not a bool.

```pavo
(assert-not (bool-xor false false))
(assert (bool-xor false true))
(assert (bool-xor true false))
(assert-not (bool-xor true true))

(assert-throw (bool-xor false 1) {:tag :err-type})
```

### Int

Ints are [signed 64 bit integers](https://en.wikipedia.org/wiki/Integer_(computer_science)) represented in [two's complement](https://en.wikipedia.org/wiki/Two's_complement), that is numbers between `-2^63` and and `2^63 - 1` inclusive. Because of their finite width and the inherent asymmetry of two's complement representation, the functions operating on integers often have cornercases. The "default" functions (`int-add` and friends) throw an error when reaching the boundaries of the numeric representation. Others (`int-add-wrap` etc. and `int-add-sat` etc.) allow the programmer to embrace the limits of the finite representation. Not caring about those limits at all however usually leads to bad surprises.

Most of the function (documentation) has been taken/adapted from the [rust i64 docs](https://doc.rust-lang.org/std/primitive.i64.html). A helpful discussion of various design choices for the behavior of the modulus and division operations is [Boute, Raymond T. "The Euclidean definition of the functions div and mod."](https://biblio.ugent.be/publication/314490/file/452146.pdf).

#### `int-max-val`

The largest integer, `2^63 - 1`.

```pavo
(assert-eq int-max-val 9223372036854775807)
(assert-throw (int-add int-max-val 1) {:tag :err-wrap-int})
```

#### `int-min-val`

The smallest integer, `- 2^63`.

```pavo
(assert-eq int-min-val -9223372036854775808)
(assert-throw (int-sub int-min-val 1) {:tag :err-wrap-int})
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

Throws `{:tag :err-wrap-int}` in case of an overflow.

```pavo
(assert-eq (int-add 1 2) 3)
(assert-eq (int-add 1 -2) -1)
(assert-throw (int-add int-max-val 1) {:tag :err-wrap-int})
```

#### `(int-sub n m)`

Subtracts the int `m` from the int `n`.

Throws `{:tag :err-wrap-int}` in case of an overflow.

```pavo
(assert-eq (int-sub 1 2) -1)
(assert-eq (int-sub 1 -2) 3)
(assert-throw (int-sub int-min-val 1) {:tag :err-wrap-int})
```

#### `(int-mul n m)`

Multiplies the int `n` with the int `m`.

Throws `{:tag :err-wrap-int}` in case of an overflow.

```pavo
(assert-eq (int-mul 2 3) 6)
(assert-eq (int-mul 2 -3) -6)
(assert-throw (int-mul int-max-val 2) {:tag :err-wrap-int})
```

#### `(int-div n m)`

Divides the int `n` by the int `m`.

Throws `{:tag :err-wrap-int}` in case of an overflow. Throws `{:tag :err-zero}` if `m` is `0`.

This computes the quotient of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-div 8 3) 2)
(assert-eq (int-div -8 3) -3)
(assert-throw (int-div int-min-val -1) {:tag :err-wrap-int})
(assert-throw (int-div 1 0) {:tag :err-zero})
```

#### `(int-div-trunc n m)`

Divides the int `n` by the int `m`.

Throws `{:tag :err-wrap-int}` in case of an overflow. Throws `{:tag :err-zero}` if `m` is `0`.

This computes the quotient of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-div-trunc 8 3) 2)
(assert-eq (int-div-trunc -8 3) -2)
(assert-throw (int-div-trunc int-min-val -1) {:tag :err-wrap-int})
(assert-throw (int-div-trunc 1 0) {:tag :err-zero})
```

#### `(int-mod n m)`

Computes the int `n` modulo the int `m`.

Throws `{:tag :err-wrap-int}` in case of an overflow. Throws `{:tag :err-zero}` if `m` is `0`.

This computes the remainder of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-mod 8 3) 2)
(assert-eq (int-mod -8 3) 1)
(assert-throw (int-mod int-min-val -1) {:tag :err-wrap-int})
(assert-throw (int-mod 1 0) {:tag :err-zero})
```

#### `(int-mod-trunc n m)`

Computes the int `n` modulo the int `m`.

Throws `{:tag :err-wrap-int}` in case of an overflow. Throws `{:tag :err-zero}` if `m` is `0`.

This computes the remainder of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-mod-trunc 8 3) 2)
(assert-eq (int-mod-trunc -8 3) -2)
(assert-throw (int-mod-trunc int-min-val -1) {:tag :err-wrap-int})
(assert-throw (int-mod-trunc 1 0) {:tag :err-zero})
```

#### `(int-neg n)`

Negates the int `n`.Throws `{:tag :err-wrap-int}` in case of an overflow.

```pavo
(assert-eq (int-neg 42) -42)
(assert-eq (int-neg -42) 42)
(assert-eq (int-neg 0) 0)
(assert-throw (int-neg int-min-val) {:tag :err-wrap-int})
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

Throws `{:tag :err-wrap-int}` in case of an overflow.

```pavo
(assert-eq (int-abs 42) 42)
(assert-eq (int-abs -42) 42)
(assert-eq (int-abs 0) 0)
(assert-throw (int-abs int-min-val) {:tag :err-wrap-int})
```

#### `(int-pow n m)`

Computes the int `n` to the power of the positive int `m`.

Throws `{:tag :err-wrap-int}` in case of an overflow.

```pavo
(assert-eq (int-pow 2 3) 8)
(assert-eq (int-pow 2 0) 1)
(assert-eq (int-pow 0 999) 0)
(assert-eq (int-pow 1 999) 1)
(assert-eq (int-pow -1 999) -1)
(assert-throw (int-pow 99 99) {:tag :err-wrap-int})
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

Throws `{:tag :err-zero}` if `m` is `0`.

This computes the quotient of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-div-wrap 8 3) 2)
(assert-eq (int-div-wrap -8 3) -3)
(assert-eq (int-div-wrap int-min-val -1) int-min-val)
(assert-throw (int-div-wrap 1 0) {:tag :err-zero})
```

#### `(int-div-trunc-wrap n m)`

Divides the int `n` by the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{:tag :err-zero}` if `m` is `0`.

This computes the quotient of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-div-trunc-wrap 8 3) 2)
(assert-eq (int-div-trunc-wrap -8 3) -2)
(assert-eq (int-div-trunc-wrap int-min-val -1) int-min-val)
(assert-throw (int-div-trunc-wrap 1 0) {:tag :err-zero})
```

#### `(int-mod-wrap n m)`

Computes the int `n` modulo the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{:tag :err-zero}` if `m` is `0`.

This computes the remainder of [euclidean division](https://en.wikipedia.org/wiki/Euclidean_division).

```pavo
(assert-eq (int-mod-wrap 8 3) 2)
(assert-eq (int-mod-wrap -8 3) 1)
(assert-eq (int-mod-wrap int-min-val -1) 0)
(assert-throw (int-mod-wrap 1 0) {:tag :err-zero})
```

#### `(int-mod-trunc-wrap n m)`

Computes the int `n` modulo the int `m`, wrapping around the numeric bounds instead of overflowing.

Throws `{:tag :err-zero}` if `m` is `0`.

This computes the remainder of [truncating division](https://en.wikipedia.org/w/index.php?title=Truncated_division).

```pavo
(assert-eq (int-mod-trunc-wrap 8 3) 2)
(assert-eq (int-mod-trunc-wrap -8 3) -2)
(assert-eq (int-mod-trunc-wrap int-min-val -1) 0)
(assert-throw (int-mod-trunc-wrap 1 0) {:tag :err-zero})
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
(assert-throw (int-pow-wrap 2 -1) {:tag :err-negative})
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

Whenever a function takes a "byte" as an argument but is given a non-int argument, a type error is thrown. If it is an int but it is not between zero and 255, `{:tag :err-not-byte}` is thrown.

```pavo
(assert-throw (bytes-insert @[] 0 :256) {:tag :err-type})
(assert-throw (bytes-insert @[] 0 256) {:tag :err-not-byte})
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

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-get @[42] 0) 42)
(assert-throw (bytes-get @[] 0) {:tag :err-lookup})
```

#### `(bytes-insert b index new)`

Inserts the byte `new` into the bytes `b` at the index int `index`.

Throws `{:tag :err-lookup}` if the index is out of bounds.
Throws `{:tag :err-not-byte}` if `new` is not a byte (an int between 0 and 255 inclusive).
Throws `{:tag :err-collection-full}` if the resulting bytes would contain 2^63 or more elements.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-insert @[0 1] 0 42) @[42 0 1])
(assert-eq (bytes-insert @[0 1] 1 42) @[0 42 1])
(assert-eq (bytes-insert @[0 1] 2 42) @[0 1 42])
(assert-throw (bytes-insert @[0 1] 3 42) {:tag :err-lookup})
(assert-throw (bytes-insert @[] 0 256) {:tag :err-not-byte})
  (assert-throw (bytes-insert @[] 0 :256) {:tag :err-type})
```

#### `(bytes-remove b index)`

Returns the bytes obtained by removing the byte at the index int `index` from the bytes `b`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-remove @[0 1] 0) @[1])
(assert-eq (bytes-remove @[0 1] 1) @[0])
(assert-throw (bytes-remove @[0 1] 3) {:tag :err-lookup})
```

#### `(bytes-update b index new)`

Returns the bytes obtained by replacing the byte at the index int `index` in the bytes `b` with the byte `new`.

Throws `{:tag :err-lookup}` if the index is out of bounds.
Throws `{:tag :err-not-byte}` if `new` is not a byte (an int between 0 and 255 inclusive).

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-update @[0 1] 0 42) @[42 1])
(assert-eq (bytes-update @[0 1] 1 42) @[0 42])
(assert-throw (bytes-update @[0 1] 2 42) {:tag :err-lookup})
(assert-throw (bytes-update @[0] 0 256) {:tag :err-not-byte})
```

#### `(bytes-split b index)`

Splits the bytes `b` at the index int `index`, returning an array containing two bytes: The first from 0 (inclusive) to `index` (exclusive), the second from `index` (inclusive) to the end.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-split @[0 1 2] 0) [@[] @[0 1 2]])
(assert-eq (bytes-split @[0 1 2] 1) [@[0] @[1 2]])
(assert-eq (bytes-split @[0 1 2] 2) [@[0 1] @[2]])
(assert-eq (bytes-split @[0 1 2] 3) [@[0 1 2] @[]])
(assert-throw (bytes-split @[0 1 2] 4) {:tag :err-lookup})
```

#### `(bytes-slice b start end)`

Returns a subsequence of the bytes `b`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{:tag :err-lookup}` if `start` is greater than `end`.
Throws `{:tag :err-lookup}` if `start` is out of bounds.
Throws `{:tag :err-lookup}` if `end` is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes-slice @[42 43] 1 1) @[])
(assert-eq (bytes-slice @[42 43] 0 1) @[42])
(assert-eq (bytes-slice @[42 43] 1 2) @[43])
(assert-eq (bytes-slice @[42 43] 0 2) @[42 43])
(assert-throw (bytes-slice @[] 0 1) {:tag :err-lookup})
(assert-throw (bytes-slice @[] 2 3) {:tag :err-lookup})
(assert-throw (bytes-slice @[0 1 2 3] 2 1) {:tag :err-lookup})
```

#### `(bytes-splice old index new)`

Inserts the elements of the bytes `new` into the bytes `old`, starting at the index int `index`.

Throws `{:tag :err-lookup}` if the index is out of bounds (of the `old` bytes).
Throws `{:tag :err-collection-full}` if the resulting bytes would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(bytes-count old)` and m is `(bytes-count new)`.

```pavo
(assert-eq (bytes-splice @[0 1] 0 @[10 11]) @[10 11 0 1])
(assert-eq (bytes-splice @[0 1] 1 @[10 11]) @[0 10 11 1])
(assert-eq (bytes-splice @[0 1] 2 @[10 11]) @[0 1 10 11])
(assert-throw (bytes-splice @[0 1] 3 @[10 11]) {:tag :err-lookup})
```

#### `(bytes-concat left right)`

Returns a bytes that contains all elements of the bytes `left` followed by all elements of the bytes `right`.

Throws `{:tag :err-collection-full}` if the resulting bytes would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(bytes-count left)` and m is `(bytes-count right)`.

```pavo
(assert-eq (bytes-concat @[0 1] @[2 3]) @[0 1 2 3])
(assert-eq (bytes-concat @[] @[0 1]) @[0 1])
(assert-eq (bytes-concat @[0 1] @[]) @[0 1])
```

### `(bytes-cursor b index)`

Returns a new bytes cursor (see below), positioned right before the given int `index` of the bytes `b`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(bytes-count b)`.

```pavo
(assert-throw (bytes-cursor @[0 1 2] -1) {:tag :err-lookup})
(assert-eq (cursor-bytes-next! (bytes-cursor @[0 1 2] 0)) 0)
(assert-eq (cursor-bytes-next! (bytes-cursor @[0 1 2] 1)) 1)
(assert-eq (cursor-bytes-next! (bytes-cursor @[0 1 2] 2)) 2)
(assert-throw (cursor-bytes-next! (bytes-cursor @[0 1 2] 3)) :cursor-end)
(assert-throw (bytes-cursor @[0 1 2] 4) {:tag :err-lookup})
```

### Bytes Cursor

A bytes cursor (created via `bytes-cursor`) is an opaque value that allows to efficiently step through the items in a bytes. It maintains a current position as state: either "in between" two elements, or at the front or back.

#### `cursor-bytes-type`

The type symbol of bytes cursors.

```pavo
(assert-eq cursor-bytes-type (typeof (bytes-cursor @[] 0)))
```

#### `(cursor-bytes-next! cursor)`

Advances the cursor by one element and returns the element over which it passed. If the starting position was at the back of the bytes, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of elements in the underlying bytes. Moving from the front to the back of the bytes is guaranteed to take amortized O(n).

```pavo
(let cursor (bytes-cursor @[0 1 2] 0) (do [
    (assert-eq (cursor-bytes-next! cursor) 0)
    (assert-eq (cursor-bytes-next! cursor) 1)
    (assert-eq (cursor-bytes-next! cursor) 2)
    (assert-throw (cursor-bytes-next! cursor) :cursor-end)
    (assert-throw (cursor-bytes-next! cursor) :cursor-end)
]))
```

#### `(cursor-bytes-prev! cursor)`

Retreats the cursor by one element and returns the element over which it passed. If the starting position was at the front of the bytes, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of elements in the underlying bytes. Moving from the back to the front of the bytes is guaranteed to take amortized O(n).

```pavo
(let cursor (bytes-cursor @[0 1 2] 3) (do [
    (assert-eq (cursor-bytes-prev! cursor) 2)
    (assert-eq (cursor-bytes-prev! cursor) 1)
    (assert-eq (cursor-bytes-prev! cursor) 0)
    (assert-throw (cursor-bytes-prev! cursor) :cursor-end)
    (assert-throw (cursor-bytes-prev! cursor) :cursor-end)
]))
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

Throws `{:tag :err-not-unicode-scalar}` if `n` is `n` is not a unicode scalar value.

```pavo
(assert-eq (int=>char 0x41) 'A')
(assert-throw (int=>char 0x110000) {:tag :err-not-unicode-scalar})
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

#### `(str->bytes s)`

Returns the utf-8 encoding of the string `s` as a bytes.

Time: O(n), where n is `(str-count-utf8 s)`.

```pavo
(assert-eq (str->bytes "") @[])
(assert-eq (str->bytes "abc") @[97 98 99])
(assert-eq (str->bytes "⚗") @[226 154 151])
```

#### `(bytes=>str b)`

If the bytes `b` are valid utf-8, returns the string they encode. Otherwise, throws `{:tag :err-utf8}`.

Time: O(n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes=>str @[]) "")
(assert-eq (bytes=>str @[97 98 99]) "abc")
(assert-eq (bytes=>str @[226 154 151]) "⚗")
(assert-throw (bytes=>str @[255]) {:tag :err-utf8})
```

#### `(bytes=>str? b)`

Returns `true` if the bytes `b` are valid utf-8, `false` otherwise.

Time: O(n), where n is `(bytes-count b)`.

```pavo
(assert-eq (bytes=>str? @[]) true)
(assert-eq (bytes=>str? @[97 98 99]) true)
(assert-eq (bytes=>str? @[226 154 151]) true)
(assert-eq (bytes=>str? @[255]) false)
```

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

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-get "a" 0) 'a')
(assert-eq (str-get "⚗b" 1) 'b')
(assert-throw (str-get "" 0) {:tag :err-lookup})
```

#### `(str-get-utf8 s index)`

Returns the utf8 byte at the int `index` (in bytes) in the string `s`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(str-count-utf8 s)`.

```pavo
(assert-eq (str-get-utf8 "a" 0) 97)
(assert-eq (str-get-utf8 "⚗" 0) 226)
(assert-eq (str-get-utf8 "⚗" 1) 154)
(assert-eq (str-get-utf8 "⚗" 2) 151)
(assert-throw (str-get-utf8 "" 0) { :tag :err-lookup})
```

#### `(str-index-char->utf8 str index)`

Finds the character at the int `index` in the string `s`, and returns at which byte index it begins.

Throws `{:tag :err-lookup}` if the index is out of bounds.

```pavo
(assert-eq (str-index-char->utf8 "a" 0) 0)
(assert-eq (str-index-char->utf8 "ab" 1) 1)
(assert-eq (str-index-char->utf8 "⚗b" 1) 3)
(assert-throw (str-index-char->utf8 "" 0) {:tag :err-lookup})
```

#### `(str-index-utf8->char str index)`

Finds the utf8 byte at the int `index` (in bytes) in the string `s`, and returns at which position (in characters) the character to which it belongs begins.

Throws `{:tag :err-lookup}` if the index is out of bounds.

```pavo
(assert-eq (str-index-utf8->char "a" 0) 0)
(assert-eq (str-index-utf8->char "ab" 1) 1)
(assert-eq (str-index-utf8->char "⚗b" 1) 0)
(assert-eq (str-index-utf8->char "⚗b" 2) 0)
(assert-eq (str-index-utf8->char "⚗b" 3) 1)
(assert-throw (str-index-char->utf8 "" 0) {:tag :err-lookup})
```

#### `(str-insert s index c)`

Inserts the char `c` into the string `s` at the index int `index`.

Throws `{:tag :err-lookup}` if the index is out of bounds.
Throws `{:tag :err-collection-full}` if the resulting string would contain 2^63 or more elements.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-insert "ab" 0 'z') "zab")
(assert-eq (str-insert "ab" 1 'z') "azb")
(assert-eq (str-insert "ab" 2 'z') "abz")
(assert-throw (str-insert "ab" 3 'z') {:tag :err-lookup})
```

#### `(str-remove s index)`

Returns the string obtained by removing the char at the index int `index` from the string `s`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-remove "ab" 0) "b")
(assert-eq (str-remove "ab" 1) "a")
(assert-throw (str-remove "ab" 2) {:tag :err-lookup})
```

#### `(str-update s index c)`

Returns the string obtained by replacing the char at the index int `index` in the string `s` with the char `c`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-update "ab" 0 'z') "zb")
(assert-eq (str-update "ab" 1 'z') "az")
(assert-throw (str-update "ab" 2 'z') {:tag :err-lookup})
```

#### `(str-split s index)`

Splits the string `s` at the index int `index`, returning an array containing two strings: The first from 0 (inclusive) to `index` (exclusive), the second from `index` (inclusive) to the end.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-split "a⚗c" 0) ["" "a⚗c"])
(assert-eq (str-split "a⚗c" 1) ["a" "⚗c"])
(assert-eq (str-split "a⚗c" 2) ["a⚗" "c"])
(assert-eq (str-split "a⚗c" 3) ["a⚗c" ""])
(assert-throw (str-split "a⚗c" 4) {:tag :err-lookup})
```

#### `(str-slice s start end)`

Returns a substring of the string `b`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{:tag :err-lookup}` if `start` is greater than `end`.
Throws `{:tag :err-lookup}` if `start` is out of bounds.
Throws `{:tag :err-lookup}` if `end` is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-eq (str-slice "ab" 1 1) "")
(assert-eq (str-slice "ab" 0 1) "a")
(assert-eq (str-slice "ab" 1 2) "b")
(assert-eq (str-slice "ab" 0 2) "ab")
(assert-throw (str-slice "" 0 1) {:tag :err-lookup})
(assert-throw (str-slice "" 2 3) {:tag :err-lookup})
(assert-throw (str-slice "abcd" 2 1) {:tag :err-lookup})
```

#### `(str-splice old index new)`

Inserts the string `new` into the string `old`, starting at the index int `index`.

Throws `{:tag :err-lookup}` if the index is out of bounds (of the `old` bytes).
Throws `{:tag :err-collection-full}` if the resulting string would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(str-count old)` and m is `(str-count new)`.

```pavo
(assert-eq (str-splice "ab" 0 "cd") "cdab")
(assert-eq (str-splice "ab" 1 "cd") "acdb")
(assert-eq (str-splice "ab" 2 "cd") "abcd")
(assert-throw (str-splice "ab" 3 "cd") {:tag :err-lookup})
```

#### `(str-concat left right)`

Returns a string that contains all chars of the string `left` followed by all chars of the string `right`.

Throws `{:tag :err-collection-full}` if the resulting string would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(str-count left)` and m is `(str-count right)`.

```pavo
(assert-eq (str-concat "ab" "cd") "abcd")
(assert-eq (str-concat "" "cd") "cd")
(assert-eq (str-concat "ab" "") "ab")
```

### `(str-cursor s index)`

Returns a new string cursor (see below), positioned right before the given int `index` of the string `s`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(str-count s)`.

```pavo
(assert-throw (str-cursor "a⚗c" -1) {:tag :err-lookup})
(assert-eq (cursor-str-next! (str-cursor "a⚗c" 0)) 'a')
(assert-eq (cursor-str-next! (str-cursor "a⚗c" 1)) '⚗')
(assert-eq (cursor-str-next! (str-cursor "a⚗c" 2)) 'c')
(assert-throw (cursor-str-next! (str-cursor "a⚗c" 3)) :cursor-end)
(assert-throw (str-cursor "a⚗c" 4) {:tag :err-lookup})
```

### `(str-cursor-utf8 s index)`

Returns a new string utf8 cursor (see further below), positioned right before utf8 byte at the given int `index` (in bytes) of the string `s`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(str-count-utf8 s)`.

```pavo
(assert-throw (str-cursor-utf8 "a⚗c" -1) {:tag :err-lookup})
(assert-eq (cursor-str-utf8-next! (str-cursor-utf8 "a⚗c" 0)) 97)
(assert-eq (cursor-str-utf8-next! (str-cursor-utf8 "a⚗c" 1)) 226)
(assert-eq (cursor-str-utf8-next! (str-cursor-utf8 "a⚗c" 2)) 154)
(assert-eq (cursor-str-utf8-next! (str-cursor-utf8 "a⚗c" 3)) 151)
(assert-eq (cursor-str-utf8-next! (str-cursor-utf8 "a⚗c" 4)) 99)
(assert-throw (cursor-str-utf8-next! (str-cursor-utf8 "a⚗c" 5)) :cursor-end)
(assert-throw (str-cursor-utf8 "a⚗c" 6) {:tag :err-lookup, :got 6})
```

### String Cursor

A string cursor (created via `str-cursor`) is an opaque value that allows to efficiently step through the character of a string. It maintains a current position as state: either "in between" two character, or at the front or back.

#### `cursor-str-type`

The type symbol of string cursors.

```pavo
(assert-eq cursor-str-type (typeof (str-cursor "" 0)))
```

#### `(cursor-str-next! cursor)`

Advances the cursor by one character and returns the character over which it passed. If the starting position was at the back of the string, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of characters in the underlying string. Moving from the front to the back of the string is guaranteed to take amortized O(n).

```pavo
(let cursor (str-cursor "a⚗c" 0) (do [
    (assert-eq (cursor-str-next! cursor) 'a')
    (assert-eq (cursor-str-next! cursor) '⚗')
    (assert-eq (cursor-str-next! cursor) 'c')
    (assert-throw (cursor-str-next! cursor) :cursor-end)
    (assert-throw (cursor-str-next! cursor) :cursor-end)
]))
```

#### `(cursor-str-prev! cursor)`

Retreats the cursor by one character and returns the character over which it passed. If the starting position was at the front of the string, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of characters in the underlying string. Moving from the back to the front of the string is guaranteed to take amortized O(n).

```pavo
(let cursor (str-cursor "a⚗c" 3) (do [
    (assert-eq (cursor-str-prev! cursor) 'c')
    (assert-eq (cursor-str-prev! cursor) '⚗')
    (assert-eq (cursor-str-prev! cursor) 'a')
    (assert-throw (cursor-str-prev! cursor) :cursor-end)
    (assert-throw (cursor-str-prev! cursor) :cursor-end)
]))
```

### String Utf8 Cursor

A string utf8 cursor (created via `str-cursor-utf8`) is an opaque value that allows to efficiently step through the bytes of a string's utf8 encoding. It maintains a current position as state: either "in between" two bytes, or at the front or back.

#### `cursor-str-utf8-type`

The type symbol of string utf8 cursors.

```pavo
(assert-eq cursor-str-utf8-type (typeof (str-cursor-utf8 "" 0)))
```

#### `(cursor-str-utf8-next! cursor)`

Advances the cursor by one byte and returns the byte over which it passed. If the starting position was at the back of the string, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of bytes in the underlying string. Moving from the front to the back of the string is guaranteed to take amortized O(n).

```pavo
(let cursor (str-cursor-utf8 "a⚗c" 0) (do [
    (assert-eq (cursor-str-utf8-next! cursor) 97)
    (assert-eq (cursor-str-utf8-next! cursor) 226)
    (assert-eq (cursor-str-utf8-next! cursor) 154)
    (assert-eq (cursor-str-utf8-next! cursor) 151)
    (assert-eq (cursor-str-utf8-next! cursor) 99)
    (assert-throw (cursor-str-utf8-next! cursor) :cursor-end)
    (assert-throw (cursor-str-utf8-next! cursor) :cursor-end)
]))
```

#### `(cursor-str-utf8-prev! cursor)`

Retreats the cursor by one byte and returns the byte over which it passed. If the starting position was at the front of the string, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of bytes in the underlying string. Moving from the back to the front of the string is guaranteed to take amortized O(n).

```pavo
(let cursor (str-cursor-utf8 "a⚗c" 5) (do [
    (assert-eq (cursor-str-utf8-prev! cursor) 99)
    (assert-eq (cursor-str-utf8-prev! cursor) 151)
    (assert-eq (cursor-str-utf8-prev! cursor) 154)
    (assert-eq (cursor-str-utf8-prev! cursor) 226)
    (assert-eq (cursor-str-utf8-prev! cursor) 97)
    (assert-throw (cursor-str-utf8-prev! cursor) :cursor-end)
    (assert-throw (cursor-str-utf8-prev! cursor) :cursor-end)
]))
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

Calculates the length of the hypotenuse of a right-angle triangular given legs of the float lengths `x` and `y`.

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

#### `(float-integral? x)`

Returns `true` if the float `x` is a mathematical integer, false otherwise.

```pavo
(assert-eq (float-integral? 1.0) true)
(assert-eq (float-integral? 0.0) true)
(assert-eq (float-integral? -42.0) true)
(assert-eq (float-integral? 1.1) false)
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

Returns whether applying `bits=>float` to the float `n` would work (`true`) or throw an error (`false`).

```pavo
(assert-eq (bits=>float? 42) true)
(assert-eq (bits=>float? -42) false)
(assert-eq (bits=>float? 9218868437227405312) false)
(assert-eq (bits=>float? -4503599627370496) false)
```

### Identifiers

Identifiers are intended to be used for syntax. If you find yourself using these functions a lot (possibly at all), ask yourself whether it is really necessary. These function primarily exist so that identifiers can be serialized and deserialized without horribly misusing `read` and `write` to do so.

#### `(str=>id s)`

Returns an identifier created from the string `s`.

Throws `{:tag :err-identifier}` if it would not be a valid identifier (empty, longer than 255 characters, containing invalid characters, or would be a nil, bool, int or float literal).

Time: O(n) where n is `(str-count s)`.

```pavo
(assert-eq (str=>id "foo") $foo)
(assert-throw (str=>id "nil") {:tag :err-identifier})
(assert-throw (str=>id "true") {:tag :err-identifier})
(assert-throw (str=>id "false") {:tag :err-identifier})
(assert-throw (str=>id "42") {:tag :err-identifier})
(assert-throw (str=>id "1.2") {:tag :err-identifier})
(assert-throw (str=>id "") {:tag :err-identifier})
(assert-throw (str=>id "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") {:tag :err-identifier})
(assert-throw (str=>id ":a") {:tag :err-identifier})
(assert-throw (str=>id "ß") {:tag :err-identifier})
```

#### `(str=>id? s)`

Returns whether the string `s` would be a valid identifier, i.e. it is neither empty nor longer than 255 characters, contains only valid identifier characters, and is not a nil, bool, int or float literal.

```pavo
(assert-eq (str=>id? "foo") true)
(assert-eq (str=>id? "nil") false)
(assert-eq (str=>id? "true") false)
(assert-eq (str=>id? "false") false)
(assert-eq (str=>id? "42") false)
(assert-eq (str=>id? "-_") true)
(assert-eq (str=>id? "-42") false)
(assert-eq (str=>id? "1.2") false)
(assert-eq (str=>id? "") false)
(assert-eq (str=>id? "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") false)
(assert-eq (str=>id? "ß") false)
(assert-eq (str=>id? ":a") false)
```

#### `(id->str id)`

Returns the string that corresponds to the given identfier `id`.

```pavo
(assert-eq (id->str $foo) "foo")
```

### Keywords

Just like identifiers, keywords are primarily to be compared, not to be dynamically computed. These function primarily exist so that keywords can be serialized and deserialized without horribly misusing `read` and `write` to do so.

#### `(str=>kw s)`

Returns the keyword `<:s>` created from the string `s`.

Throws `{:tag :err-kw}` if it would not be a valid keyword (empty, longer than 255 characters, or containing invalid characters).

Time: O(n) where n is `(str-count s)`.

```pavo
(assert-eq (str=>kw "foo") :foo)
(assert-eq (str=>kw "nil") :nil)
(assert-eq (str=>kw "42") :42)
(assert-throw (str=>kw "") {:tag :err-kw})
(assert-throw (str=>kw "01234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789") {:tag :err-kw})
(assert-throw (str=>kw ":a") {:tag :err-kw})
(assert-throw (str=>kw "ß") {:tag :err-kw})
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

#### `(arr->app arr)`

Returns an application that contains the same items in the same order as the array `arr`.

Time: O(1)

```pavo
(assert-eq (arr->app []) $())
(assert-eq (arr->app [0 1 2]) $(0 1 2))
```

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

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-get [true] 0) true)
(assert-throw (arr-get [] 0) {:tag :err-lookup})
```

#### `(arr-insert arr index new)`

Inserts the value `new` into the array `arr` at the index int `index`.

Throws `{:tag :err-lookup}` if the index is out of bounds.
Throws `{:tag :err-collection-full}` if the resulting array would contain 2^63 or more elements.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-insert [0 1] 0 42) [42 0 1])
(assert-eq (arr-insert [0 1] 1 42) [0 42 1])
(assert-eq (arr-insert [0 1] 2 42) [0 1 42])
(assert-throw (arr-insert [0 1] 3 42) {:tag :err-lookup})
```

#### `(arr-remove arr index)`

Returns the array obtained by removing the element at the index int `index` from the array `arr`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-remove [0 1] 0) [1])
(assert-eq (arr-remove [0 1] 1) [0])
(assert-throw (arr-remove [0 1] 3) {:tag :err-lookup})
```

#### `(arr-update arr index new)`

Returns the array obtained by replacing the element at the index int `index` in the array `arr` with the value `new`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-update [0 1] 0 42) [42 1])
(assert-eq (arr-update [0 1] 1 42) [0 42])
(assert-throw (arr-update [0 1] 2 42) {:tag :err-lookup})
```

#### `(arr-split arr index)`

Splits the array `arr` at the index int `index`, returning an array containing two arrays: The first from 0 (inclusive) to `index` (exclusive), the second from `index` (inclusive) to the end.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-split [0 1 2] 0) [[] [0 1 2]])
(assert-eq (arr-split [0 1 2] 1) [[0] [1 2]])
(assert-eq (arr-split [0 1 2] 2) [[0 1] [2]])
(assert-eq (arr-split [0 1 2] 3) [[0 1 2] []])
(assert-throw (arr-split [0 1 2] 4) {:tag :err-lookup})
```

#### `(arr-slice arr start end)`

Returns an array containing a subsequence of the elements of the array `arr`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{:tag :err-lookup}` if `start` is greater than `end`.
Throws `{:tag :err-lookup}` if `start` is out of bounds.
Throws `{:tag :err-lookup}` if `end` is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-eq (arr-slice [true false] 1 1) [])
(assert-eq (arr-slice [true false] 0 1) [true])
(assert-eq (arr-slice [true false] 1 2) [false])
(assert-eq (arr-slice [true false] 0 2) [true false])
(assert-throw (arr-slice [] 0 1) {:tag :err-lookup})
(assert-throw (arr-slice [] 2 3) {:tag :err-lookup})
(assert-throw (arr-slice [0 1 2 3] 2 1) {:tag :err-lookup})
```

#### `(arr-splice old index new)`

Inserts the elements of the array `new` into the array `old`, starting at the index int `index`.

Throws `{:tag :err-lookup}` if the index is out of bounds (of the `old` array).
Throws `{:tag :err-collection-full}` if the resulting array would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(arr-count old)` and m is `(arr-count new)`.

```pavo
(assert-eq (arr-splice [0 1] 0 [10 11]) [10 11 0 1])
(assert-eq (arr-splice [0 1] 1 [10 11]) [0 10 11 1])
(assert-eq (arr-splice [0 1] 2 [10 11]) [0 1 10 11])
(assert-throw (arr-splice [0 1] 3 [10 11]) {:tag :err-lookup})
```

#### `(arr-concat left right)`

Returns an array that contains all elements of the array `left` followed by all elements of the array `right`.

Throws `{:tag :err-collection-full}` if the resulting array would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(arr-count left)` and m is `(arr-count right)`.

```pavo
(assert-eq (arr-concat [0 1] [2 3]) [0 1 2 3])
(assert-eq (arr-concat [] [0 1]) [0 1])
(assert-eq (arr-concat [0 1] []) [0 1])
```

### `(arr-cursor arr index)`

Returns a new array cursor (see below), positioned right before the given int `index` of the array `arr`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(arr-count arr)`.

```pavo
(assert-throw (arr-cursor [0 1 2] -1) {:tag :err-lookup})
(assert-eq (cursor-arr-next! (arr-cursor [0 1 2] 0)) 0)
(assert-eq (cursor-arr-next! (arr-cursor [0 1 2] 1)) 1)
(assert-eq (cursor-arr-next! (arr-cursor [0 1 2] 2)) 2)
(assert-throw (cursor-arr-next! (arr-cursor [0 1 2] 3)) :cursor-end)
(assert-throw (arr-cursor [0 1 2] 4) {:tag :err-lookup})
```

### Array Cursor

An array cursor (created via `arr-cursor`) is an opaque value that allows to efficiently step through the items in an array. It maintains a current position as state: either "in between" two elements, or at the front or back.

#### `cursor-arr-type`

The type symbol of array cursors.

```pavo
(assert-eq cursor-arr-type (typeof (arr-cursor [] 0)))
```

#### `(cursor-arr-next! cursor)`

Advances the cursor by one element and returns the element over which it passed. If the starting position was at the back of the array, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of elements in the underlying array. Moving from the front to the back of the array is guaranteed to take amortized O(n).

```pavo
(let cursor (arr-cursor [0 1 2] 0) (do [
    (assert-eq (cursor-arr-next! cursor) 0)
    (assert-eq (cursor-arr-next! cursor) 1)
    (assert-eq (cursor-arr-next! cursor) 2)
    (assert-throw (cursor-arr-next! cursor) :cursor-end)
    (assert-throw (cursor-arr-next! cursor) :cursor-end)
]))
```

#### `(cursor-arr-prev! cursor)`

Retreats the cursor by one element and returns the element over which it passed. If the starting position was at the front of the array, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of elements in the underlying array. Moving from the back to the front of the array is guaranteed to take amortized O(n).

```pavo
(let cursor (arr-cursor [0 1 2] 3) (do [
    (assert-eq (cursor-arr-prev! cursor) 2)
    (assert-eq (cursor-arr-prev! cursor) 1)
    (assert-eq (cursor-arr-prev! cursor) 0)
    (assert-throw (cursor-arr-prev! cursor) :cursor-end)
    (assert-throw (cursor-arr-prev! cursor) :cursor-end)
]))
```

### Applications

#### `(app-apply app)`

Applies the first value in the application `app` to the remaining values.

```pavo
(assert-eq (app-apply `(~int-add 1 2)) 3)
(assert-throw (app-apply `(~int-add 1)) {:tag :err-num-args})
(assert-throw (app-apply $()) {:tag :err-lookup})
(assert-throw (app-apply $(42)) {:tag :err-type})
```

#### `(app->arr app)`

Returns an array that contains the same items in the same order as the application `app`.

Time: O(1)

```pavo
(assert-eq (app->arr $()) [])
(assert-eq (app->arr $(0 1 2)) [0 1 2])
```

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

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-get $(true) 0) true)
(assert-throw (app-get $() 0) {:tag :err-lookup})
```

#### `(app-insert app index new)`

Inserts the value `new` into the application `app` at the index int `index`.

Throws `{:tag :err-lookup}` if the index is out of bounds.
Throws `{:tag :err-collection-full}` if the resulting application would contain 2^63 or more elements.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-insert $(0 1) 0 42) $(42 0 1))
(assert-eq (app-insert $(0 1) 1 42) $(0 42 1))
(assert-eq (app-insert $(0 1) 2 42) $(0 1 42))
(assert-throw (app-insert $(0 1) 3 42) {:tag :err-lookup})
```

#### `(app-remove app index)`

Returns the application obtained by removing the element at the index int `index` from the application `app`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-remove $(0 1) 0) $(1))
(assert-eq (app-remove $(0 1) 1) $(0))
(assert-throw (app-remove $(0 1) 3) {:tag :err-lookup})
```

#### `(app-update app index new)`

Returns the application obtained by replacing the element at the index int `index` in the application `app` with the value `new`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-update $(0 1) 0 42) $(42 1))
(assert-eq (app-update $(0 1) 1 42) $(0 42))
(assert-throw (app-update $(0 1) 2 42) {:tag :err-lookup})
```

#### `(app-split app index)`

Splits the application `app` at the index int `index`, returning an array containing two applications: The first from 0 (inclusive) to `index` (exclusive), the second from `index` (inclusive) to the end.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-split $(0 1 2) 0) [$() $(0 1 2)])
(assert-eq (app-split $(0 1 2) 1) [$(0) $(1 2)])
(assert-eq (app-split $(0 1 2) 2) [$(0 1) $(2)])
(assert-eq (app-split $(0 1 2) 3) [$(0 1 2) $()])
(assert-throw (app-split $(0 1 2) 4) {:tag :err-lookup})
```

#### `(app-slice app start end)`

Returns an application containing a subsequence of the elements of the application `app`, starting at the index int `start` (inclusive) and up to the index int `end` (exclusive).

Throws `{:tag :err-lookup}` if `start` is greater than `end`.
Throws `{:tag :err-lookup}` if `start` is out of bounds.
Throws `{:tag :err-lookup}` if `end` is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-eq (app-slice $(true false) 1 1) $())
(assert-eq (app-slice $(true false) 0 1) $(true))
(assert-eq (app-slice $(true false) 1 2) $(false))
(assert-eq (app-slice $(true false) 0 2) $(true false))
(assert-throw (app-slice $() 0 1) {:tag :err-lookup})
(assert-throw (app-slice $() 2 3) {:tag :err-lookup})
(assert-throw (app-slice $(0 1 2 3) 2 1) {:tag :err-lookup})
```

#### `(app-splice old index new)`

Inserts the elements of the application `new` into the application `old`, starting at the index int `index`.

Throws `{:tag :err-lookup}` if the index is out of bounds (of the `old` application).
Throws `{:tag :err-collection-full}` if the resulting application would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(app-count old)` and m is `(app-count new)`.

```pavo
(assert-eq (app-splice $(0 1) 0 $(10 11)) $(10 11 0 1))
(assert-eq (app-splice $(0 1) 1 $(10 11)) $(0 10 11 1))
(assert-eq (app-splice $(0 1) 2 $(10 11)) $(0 1 10 11))
(assert-throw (app-splice $(0 1) 3 $(10 11)) {:tag :err-lookup})
```

#### `(app-concat left right)`

Returns an application that contains all elements of the application `left` followed by all elements of the application `right`.

Throws `{:tag :err-collection-full}` if the resulting application would contain 2^63 or more elements.

Time: O(log (n + m)), where n is `(app-count left)` and m is `(app-count right)`.

```pavo
(assert-eq (app-concat $(0 1) $(2 3)) $(0 1 2 3))
(assert-eq (app-concat $() $(0 1)) $(0 1))
(assert-eq (app-concat $(0 1) $()) $(0 1))
```

### `(app-cursor app index)`

Returns a new application cursor (see below), positioned right before the given int `index` of the application `app`.

Throws `{:tag :err-lookup}` if the index is out of bounds.

Time: O(log n), where n is `(app-count app)`.

```pavo
(assert-throw (app-cursor $(0 1 2) -1) {:tag :err-lookup})
(assert-eq (cursor-app-next! (app-cursor $(0 1 2) 0)) 0)
(assert-eq (cursor-app-next! (app-cursor $(0 1 2) 1)) 1)
(assert-eq (cursor-app-next! (app-cursor $(0 1 2) 2)) 2)
(assert-throw (cursor-app-next! (app-cursor $(0 1 2) 3)) :cursor-end)
(assert-throw (app-cursor $(0 1 2) 4) {:tag :err-lookup})
```

### Application Cursor

An application cursor (created via `app-cursor`) is an opaque value that allows to efficiently step through the items in an application. It maintains a current position as state: either "in between" two elements, or at the front or back.

#### `cursor-app-type`

The type symbol of application cursors.

```pavo
(assert-eq cursor-app-type (typeof (app-cursor $() 0)))
```

#### `(cursor-app-next! cursor)`

Advances the cursor by one element and returns the element over which it passed. If the starting position was at the back of the application, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of elements in the underlying application. Moving from the front to the back of the application is guaranteed to take amortized O(n).

```pavo
(let cursor (app-cursor $(0 1 2) 0) (do [
    (assert-eq (cursor-app-next! cursor) 0)
    (assert-eq (cursor-app-next! cursor) 1)
    (assert-eq (cursor-app-next! cursor) 2)
    (assert-throw (cursor-app-next! cursor) :cursor-end)
    (assert-throw (cursor-app-next! cursor) :cursor-end)
]))
```

#### `(cursor-app-prev! cursor)`

Retreats the cursor by one element and returns the element over which it passed. If the starting position was at the front of the application, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of elements in the underlying application. Moving from the back to the front of the application is guaranteed to take amortized O(n).

```pavo
(let cursor (app-cursor $(0 1 2) 3) (do [
    (assert-eq (cursor-app-prev! cursor) 2)
    (assert-eq (cursor-app-prev! cursor) 1)
    (assert-eq (cursor-app-prev! cursor) 0)
    (assert-throw (cursor-app-prev! cursor) :cursor-end)
    (assert-throw (cursor-app-prev! cursor) :cursor-end)
]))
```

### Sets

Sets are unordered collections of up to `2^63 - 1` unique values.

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

Throws `{:tag :err-collection-empty}` if `set` is the empty set.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-min @{ 4 3 }) 3)
(assert-throw (set-min @{}) {:tag :err-collection-empty})
```

#### `(set-max set)`

Returns the maximal element in the set `set`.

Throws `{:tag :err-collection-empty}` if `set` is the empty set.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-max @{ 4 3 }) 4)
(assert-throw (set-max @{}) {:tag :err-collection-empty})
```

#### `(set-find-< set v)`

Returns the greatest element in the set `set` that is strictly less than the value `v`.

Throws `{:tag :err-lookup}` if no such value exists.

```pavo
(assert-throw (set-find-< @{1 3} 0) {:tag :err-lookup})
(assert-throw (set-find-< @{1 3} 1) {:tag :err-lookup})
(assert-eq (set-find-< @{1 3} 2) 1)
(assert-eq (set-find-< @{1 3} 3) 1)
(assert-eq (set-find-< @{1 3} 4) 3)
```

#### `(set-find-> set v)`

Returns the least element in the set `set` that is strictly greater than the value `v`.

Throws `{:tag :err-lookup}` if no such value exists.

```pavo
(assert-eq (set-find-> @{1 3} 0) 1)
(assert-eq (set-find-> @{1 3} 1) 3)
(assert-eq (set-find-> @{1 3} 2) 3)
(assert-throw (set-find-> @{1 3} 3) {:tag :err-lookup})
(assert-throw (set-find-> @{1 3} 4) {:tag :err-lookup})
```

#### `(set-find-<= set v)`

Returns the greatest element in the set `set` that is equal or less than the value `v`.

Throws `{:tag :err-lookup}` if no such value exists.

```pavo
(assert-throw (set-find-<= @{1 3} 0) {:tag :err-lookup})
(assert-eq (set-find-<= @{1 3} 1) 1)
(assert-eq (set-find-<= @{1 3} 2) 1)
(assert-eq (set-find-<= @{1 3} 3) 3)
(assert-eq (set-find-<= @{1 3} 4) 3)
```

#### `(set-find->= set v)`

Returns the least element in the set `set` that is equal or greater than the value `v`.

Throws `{:tag :err-lookup}` if no such value exists.

```pavo
(assert-eq (set-find->= @{1 3} 0) 1)
(assert-eq (set-find->= @{1 3} 1) 1)
(assert-eq (set-find->= @{1 3} 2) 3)
(assert-eq (set-find->= @{1 3} 3) 3)
(assert-throw (set-find->= @{1 3} 4) {:tag :err-lookup})
```

#### `(set-insert set new)`

Inserts the value `new` into the set `set`.

Throws `{:tag :err-collection-full}` if the resulting set would contain 2^63 or more elements.

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

Throws `{:tag :err-collection-full}` if the resulting set would contain 2^63 or more elements.

```pavo
(assert-eq (set-union @{1 2} @{2 3}) @{1 2 3})
(assert-eq (set-union @{1 2} @{}) @{1 2})
(assert-eq (set-union @{} @{2 3}) @{2 3})
(assert-eq (set-union @{} @{}) @{})
```

Time: O(n log m), where n is the `set-count` of the smaller input and m is the `set-count` of the larger input.

#### `(set-intersection lhs rhs)`

Returns the set that contains all the elements contained in both the set `lhs` and the set `rhs`.

```pavo
(assert-eq (set-intersection @{1 2} @{2 3}) @{2})
(assert-eq (set-intersection @{1 2} @{}) @{})
(assert-eq (set-intersection @{} @{2 3}) @{})
(assert-eq (set-intersection @{} @{}) @{})
```

Time: O((n + m) log (n + m)), where n is `(set-count lhs)` and m is `(set-count rhs)`.

#### `(set-difference lhs rhs)`

Returns the set that contains all the elements contained in the set `lhs` but not contained in the set `rhs`.

```pavo
(assert-eq (set-difference @{1 2} @{2 3}) @{1})
(assert-eq (set-difference @{1 2} @{}) @{1 2})
(assert-eq (set-difference @{} @{2 3}) @{})
(assert-eq (set-difference @{} @{}) @{})
```

Time: O(m log n), where n is `(set-count lhs)` and m is `(set-count rhs)`.

#### `(set-symmetric-difference lhs rhs)`

Returns the set that contains all the elements in exactly one of the sets `lhs` and `rhs`.

```pavo
(assert-eq (set-symmetric-difference @{1 2} @{2 3}) @{1 3})
(assert-eq (set-symmetric-difference @{1 2} @{}) @{1 2})
(assert-eq (set-symmetric-difference @{} @{2 3}) @{2 3})
(assert-eq (set-symmetric-difference @{} @{}) @{})
```

Time: O((n + m) log (n + m)), where n is `(set-count lhs)` and m is `(set-count rhs)`.

#### `(set-split set v)`

Returns an array containing two sets, the first of which contains the set of all elements in the set `set` that are strictly less than the value `v`, and the second of which contains the remaining elements.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (set-split @{1 3 5} 0) [@{} @{1 3 5}])
(assert-eq (set-split @{1 3 5} 1) [@{} @{1 3 5}])
(assert-eq (set-split @{1 3 5} 2) [@{1} @{3 5}])
(assert-eq (set-split @{1 3 5} 3) [@{1} @{3 5}])
(assert-eq (set-split @{1 3 5} 4) [@{1 3} @{5}])
(assert-eq (set-split @{1 3 5} 5) [@{1 3} @{5}])
(assert-eq (set-split @{1 3 5} 6) [@{1 3 5} @{}])
```

#### `(set-slice set start end)`

Returns a set containing all elements from the set `set` that are greater or equal to `start` and strictly less than `end`.

Time: O(log n), where n is `(set-count set)`

```pavo
(assert-eq (set-slice @{1 3} 0 0) @{})
(assert-eq (set-slice @{1 3} 0 1) @{})
(assert-eq (set-slice @{1 3} 0 2) @{1})
(assert-eq (set-slice @{1 3} 1 2) @{1})
(assert-eq (set-slice @{1 3} 0 3) @{1})
(assert-eq (set-slice @{1 3} 0 4) @{1 3})
(assert-eq (set-slice @{1 3} 2 4) @{3})
(assert-eq (set-slice @{1 3} 4 0) @{})
(assert-eq (set-slice @{1 3} 99 98) @{})
```

#### `(set-cursor-min set)`

Returns a new set cursor (see below), positioned right before the minimal element of the set `set`.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (cursor-set-next! (set-cursor-min @{0 1 2})) 0)
(assert-throw (cursor-set-next! (set-cursor-min @{})) :cursor-end)
```

#### `(set-cursor-max set)`

Returns a new set cursor (see below), positioned right behind the maximal element of the set `set`.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (cursor-set-prev! (set-cursor-max @{0 1 2})) 2)
(assert-throw (cursor-set-prev! (set-cursor-max @{})) :cursor-end)
```

#### `(set-cursor-< set v)`

Returns a new set cursor (see below), positioned right behind the greatest element of the set `set` that is strictly less than the value `v`, or right before the minimal element of the set if no such value exists.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-throw (cursor-set-prev! (set-cursor-< @{0 1 3} -1)) :cursor-end)
(assert-throw (cursor-set-prev! (set-cursor-< @{0 1 3} 0)) :cursor-end)
(assert-eq (cursor-set-prev! (set-cursor-< @{0 1 3} 1)) 0)
(assert-eq (cursor-set-prev! (set-cursor-< @{0 1 3} 2)) 1)
(assert-eq (cursor-set-prev! (set-cursor-< @{0 1 3} 3)) 1)
(assert-eq (cursor-set-prev! (set-cursor-< @{0 1 3} 4)) 3)
```

#### `(set-cursor-> set v)`

Returns a new set cursor (see below), positioned right before the smallest element of the set `set` that is strictly greater than the value `v`, or right behind the maximal element of the set if no such value exists.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (cursor-set-next! (set-cursor-> @{0 1 3} -1)) 0)
(assert-eq (cursor-set-next! (set-cursor-> @{0 1 3} 0)) 1)
(assert-eq (cursor-set-next! (set-cursor-> @{0 1 3} 1)) 3)
(assert-eq (cursor-set-next! (set-cursor-> @{0 1 3} 2)) 3)
(assert-throw (cursor-set-next! (set-cursor-> @{0 1 3} 3)) :cursor-end)
(assert-throw (cursor-set-next! (set-cursor-> @{0 1 3} 4)) :cursor-end)
```

#### `(set-cursor-<= set v)`

Returns a new set cursor (see below), positioned right behind the greatest element of the set `set` that is less than or equal to the value `v`, or right before the minimal element of the set if no such value exists.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-throw (cursor-set-prev! (set-cursor-<= @{0 1 3} -1)) :cursor-end)
(assert-eq (cursor-set-prev! (set-cursor-<= @{0 1 3} 0)) 0)
(assert-eq (cursor-set-prev! (set-cursor-<= @{0 1 3} 1)) 1)
(assert-eq (cursor-set-prev! (set-cursor-<= @{0 1 3} 2)) 1)
(assert-eq (cursor-set-prev! (set-cursor-<= @{0 1 3} 3)) 3)
(assert-eq (cursor-set-prev! (set-cursor-<= @{0 1 3} 4)) 3)
```

#### `(set-cursor->= set v)`

Returns a new set cursor (see below), positioned right before the smallest element of the set `set` that is greater than or equal to the value `v`, or right behind the maximal element of the set if no such value exists.

Time: O(log n), where n is `(set-count set)`.

```pavo
(assert-eq (cursor-set-next! (set-cursor->= @{0 1 3} -1)) 0)
(assert-eq (cursor-set-next! (set-cursor->= @{0 1 3} 0)) 0)
(assert-eq (cursor-set-next! (set-cursor->= @{0 1 3} 1)) 1)
(assert-eq (cursor-set-next! (set-cursor->= @{0 1 3} 2)) 3)
(assert-eq (cursor-set-next! (set-cursor->= @{0 1 3} 3)) 3)
(assert-throw (cursor-set-next! (set-cursor->= @{0 1 3} 4)) :cursor-end)
```

### Set Cursor

A set cursor (created via `set-cursor-min`, `set-cursor-max`, `set-cursor-<`, `set-cursor->`, `set-cursor-<=` or `set-cursor->=`) is an opaque value that allows to efficiently step through the items in a set. It maintains a current position as state: either "in between" two elements, or at the front or back.

#### `cursor-set-type`

The type symbol of set cursors.

```pavo
(assert-eq cursor-set-type (typeof (set-cursor-min @{})))
```

#### `(cursor-set-next! cursor)`

Advances the set cursor by one element and returns the element over which it passed. If the starting position was at the back of the set, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of elements in the underlying set. Moving from the front to the back of the set is guaranteed to take amortized O(n).

```pavo
(let cursor (set-cursor-min @{0 1 2}) (do [
    (assert-eq (cursor-set-next! cursor) 0)
    (assert-eq (cursor-set-next! cursor) 1)
    (assert-eq (cursor-set-next! cursor) 2)
    (assert-throw (cursor-set-next! cursor) :cursor-end)
    (assert-throw (cursor-set-next! cursor) :cursor-end)
]))
```

#### `(cursor-set-prev! cursor)`

Retreats the set cursor by one element and returns the element over which it passed. If the starting position was at the front of the set, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of elements in the underlying set. Moving from the back to the front of the set is guaranteed to take amortized O(n).

```pavo
(let cursor (set-cursor-max @{0 1 2}) (do [
    (assert-eq (cursor-set-prev! cursor) 2)
    (assert-eq (cursor-set-prev! cursor) 1)
    (assert-eq (cursor-set-prev! cursor) 0)
    (assert-throw (cursor-set-prev! cursor) :cursor-end)
    (assert-throw (cursor-set-prev! cursor) :cursor-end)
]))
```

### Maps

Maps are collections of up to `2^63 - 1` key-value pairs (entries) with pairwise distinct keys.

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

Throws `{:tag :err-lookup}` if the map contains no entry with this key.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-get {0 42} 0) 42)
(assert-throw (map-get {} 0) {:tag :err-lookup})
```

#### `(map-contains? map key)`

Returns `true` if the map `map` contains an entry with key `key`, `false` otherwise.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-contains? { nil 0 } nil) true)
(assert-eq (map-contains? { 42 0 } 43) false)
(assert-eq (map-contains? {} nil) false)
```

#### `(map-find-< map v)`

Returns the greatest key in the map `map` that is strictly less than the value `v`.

Throws `{:tag :err-lookup}` if no such value exists.

```pavo
(assert-throw (map-find-< {1 nil 3 nil} 0) {:tag :err-lookup})
(assert-throw (map-find-< {1 nil 3 nil} 1) {:tag :err-lookup})
(assert-eq (map-find-< {1 nil 3 nil} 2) 1)
(assert-eq (map-find-< {1 nil 3 nil} 3) 1)
(assert-eq (map-find-< {1 nil 3 nil} 4) 3)
```

#### `(map-find-> map v)`

Returns the least key in the map `map` that is strictly greater than the value `v`.

Throws `{:tag :err-lookup}` if no such value exists.

```pavo
(assert-eq (map-find-> {1 nil 3 nil} 0) 1)
(assert-eq (map-find-> {1 nil 3 nil} 1) 3)
(assert-eq (map-find-> {1 nil 3 nil} 2) 3)
(assert-throw (map-find-> {1 nil 3 nil} 3) {:tag :err-lookup})
(assert-throw (map-find->{1 nil 3 nil} 4) {:tag :err-lookup})
```

#### `(map-find-<= map v)`

Returns the greatest key in the map `map` that is equal or less than the value `v`.

Throws `{:tag :err-lookup}` if no such value exists.

```pavo
(assert-throw (map-find-<= {1 nil 3 nil} 0) {:tag :err-lookup})
(assert-eq (map-find-<= {1 nil 3 nil} 1) 1)
(assert-eq (map-find-<= {1 nil 3 nil} 2) 1)
(assert-eq (map-find-<= {1 nil 3 nil} 3) 3)
(assert-eq (map-find-<= {1 nil 3 nil} 4) 3)
```

#### `(map-find->= map v)`

Returns the least key in the map `map` that is equal or greater than the value `v`.

Throws `{:tag :err-lookup}` if no such value exists.

```pavo
(assert-eq (map-find->= {1 nil 3 nil} 0) 1)
(assert-eq (map-find->= {1 nil 3 nil} 1) 1)
(assert-eq (map-find->= {1 nil 3 nil} 2) 3)
(assert-eq (map-find->= {1 nil 3 nil} 3) 3)
(assert-throw (map-find->= {1 nil 3 nil} 4) {:tag :err-lookup})
```

#### `(map-min map)`

Returns the value of the entry with the minimal key in the map `map`.

Throws `{:tag :err-collection-empty}` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min {0 42, 1 41}) 42)
(assert-throw (map-min {}) {:tag :err-collection-empty})
```

#### `(map-min-key map)`

Returns the minimal key in the map `map`.

Throws `{:tag :err-collection-empty}` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min-key {0 42, 1 41}) 0)
(assert-throw (map-min-key {}) {:tag :err-collection-empty})
```

#### `(map-min-entry map)`

Returns the entry with the minimal key in the map `map`, as an array `[key value]`.

Throws `{:tag :err-collection-empty}` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-min-entry {0 42, 1 41}) [0 42])
(assert-throw (map-min-entry {}) {:tag :err-collection-empty})
```

#### `(map-max map)`

Returns the value of the entry with the maximal key in the map `map`.

Throws `{:tag :err-collection-empty}` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max {0 42, 1 41}) 41)
(assert-throw (map-max {}) {:tag :err-collection-empty})
```

#### `(map-max-key map)`

Returns the maximal key in the map `map`.

Throws `{:tag :err-collection-empty}` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max-key {0 42, 1 41}) 1)
(assert-throw (map-max-key {}) {:tag :err-collection-empty})
```

#### `(map-max-entry map)`

Returns the entry with the maximal key in the map `map`, as an array `[key value]`.

Throws `{:tag :err-collection-empty}` if `map` is the empty map.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-max-entry {0 42, 1 41}) [1 41])
(assert-throw (map-max-entry {}) {:tag :err-collection-empty})
```

#### `(map-insert map key value)`

Inserts the entry `key`, `value` into the map `map`, potentially overwriting an older entry.

Throws `{:tag :err-collection-full}` if the resulting map would contain 2^63 or more entries.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-insert {} 0 42) {0 42})
(assert-eq (map-insert {0 42} 0 43) {0 43})
```

#### `(map-remove map key)`

Returns the map obtained by removing the entry (if any) with the key `key` from the map `map`.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-remove {0 42} 0) {})
(assert-eq (map-remove {} 0) {})
```

#### `(map-union lhs rhs)`

Returns the map that contains all the entries in the map `lhs` and all the entries in the map `rhs`. For entries whose keys are contained in both maps, the value from the lhs map is used.

Throws `{:tag :err-collection-full}` if the resulting map would contain 2^63 or more elements.

```pavo
(assert-eq (map-union {0 42, 1 41} {1 17, 2 40}) {0 42, 1 41, 2 40})
(assert-eq (map-union {0 42, 1 41} {}) {0 42, 1 41})
(assert-eq (map-union {} {1 41, 2 40}) {1 41, 2 40})
(assert-eq (map-union {} {}) {})
```

Time: O(n log m), where n is the `map-count` of the smaller input and m is the `map-count` of the larger input.

#### `(map-intersection lhs rhs)`

Returns the map that contains all the entries in the map `lhs` whose key is also the key of an entry in the map `rhs`.

```pavo
(assert-eq (map-intersection {0 42, 1 41} {1 17, 2 40}) {1 41})
(assert-eq (map-intersection {0 42, 1 41} {}) {})
(assert-eq (map-intersection {} {1 41, 2 40}) {})
(assert-eq (map-intersection {} {}) {})
```

Time: O((n + m) log (n + m)), where n is `(map-count lhs)` and m is `(map-count rhs)`.

#### `(map-difference lhs rhs)`

Returns the map that contains all the entries in the map `lhs` whose key is not the key of an entry in the map `rhs`.

```pavo
(assert-eq (map-difference {0 42, 1 41} {1 17, 2 40}) {0 42})
(assert-eq (map-difference {0 42, 1 41} {}) {0 42, 1 41})
(assert-eq (map-difference {} {1 41, 2 40}) {})
(assert-eq (map-difference {} {}) {})
```

Time: O(m log n), where n is `(map-count lhs)` and m is `(map-count rhs)`.

#### `(map-symmetric-difference lhs rhs)`

Returns the map that contains all the entries in the maps `lhs` and `rhs` whose key does not occur in both maps.

```pavo
(assert-eq (map-symmetric-difference {0 42, 1 41} {1 17, 2 40}) {0 42, 2 40})
(assert-eq (map-symmetric-difference {0 42, 1 41} {}) {0 42, 1 41})
(assert-eq (map-symmetric-difference {} {1 41, 2 40}) {1 41, 2 40})
(assert-eq (map-symmetric-difference {} {}) {})
```

Time: O((n + m) log (n + m)), where n is `(map-count lhs)` and m is `(map-count rhs)`.

#### `(map-split map v)`

Returns an array containing two maps, the first of which contains the map of all entries in the map `map` whose key is strictly less than the value `v`, and the second of which contains the remaining entries.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (map-split {1 :a 3 :b 5 :c} 0) [{} {1 :a 3 :b 5 :c}])
(assert-eq (map-split {1 :a 3 :b 5 :c} 1) [{} {1 :a 3 :b 5 :c}])
(assert-eq (map-split {1 :a 3 :b 5 :c} 2) [{1 :a} {3 :b 5 :c}])
(assert-eq (map-split {1 :a 3 :b 5 :c} 3) [{1 :a} {3 :b 5 :c}])
(assert-eq (map-split {1 :a 3 :b 5 :c} 4) [{1 :a 3 :b} {5 :c}])
(assert-eq (map-split {1 :a 3 :b 5 :c} 5) [{1 :a 3 :b} {5 :c}])
(assert-eq (map-split {1 :a 3 :b 5 :c} 6) [{1 :a 3 :b 5 :c} {}])
```

#### `(map-slice map start end)`

Returns a map containing all entries from the map `map` whose key is greater or equal to `start` and strictly less than `end`.

Time: O(log n), where n is `(map-count map)`

```pavo
(assert-eq (map-slice {1 :a 3 :b} 0 0) {})
(assert-eq (map-slice {1 :a 3 :b} 0 1) {})
(assert-eq (map-slice {1 :a 3 :b} 0 2) {1 :a})
(assert-eq (map-slice {1 :a 3 :b} 1 2) {1 :a})
(assert-eq (map-slice {1 :a 3 :b} 0 3) {1 :a})
(assert-eq (map-slice {1 :a 3 :b} 0 4) {1 :a 3 :b})
(assert-eq (map-slice {1 :a 3 :b} 2 4) {3 :b})
(assert-eq (map-slice {1 :a 3 :b} 4 0) {})
(assert-eq (map-slice {1 :a 3 :b} 99 98) {})
```

#### `(map-cursor-min map)`

Returns a new map cursor (see below), positioned right before the minimal entry of the map `map`.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (cursor-map-next! (map-cursor-min {0 :a 1 :b 2 :c})) [0 :a])
(assert-throw (cursor-map-next! (map-cursor-min {})) :cursor-end)
```

#### `(map-cursor-max map)`

Returns a new map cursor (see below), positioned right behind the maximal entry of the map `map`.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (cursor-map-prev! (map-cursor-max {0 :a 1 :b 2 :c})) [2 :c])
(assert-throw (cursor-map-prev! (map-cursor-max {})) :cursor-end)
```

#### `(map-cursor-< map v)`

Returns a new map cursor (see below), positioned right behind the greatest entry of the map `map` whose key is strictly less than the value `v`, or right before the minimal entry of the set if no such entry exists.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-throw (cursor-map-prev! (map-cursor-< {0 :a 1 :b 3 :d} -1)) :cursor-end)
(assert-throw (cursor-map-prev! (map-cursor-< {0 :a 1 :b 3 :d} 0)) :cursor-end)
(assert-eq (cursor-map-prev! (map-cursor-< {0 :a 1 :b 3 :d} 1)) [0 :a])
(assert-eq (cursor-map-prev! (map-cursor-< {0 :a 1 :b 3 :d} 2)) [1 :b])
(assert-eq (cursor-map-prev! (map-cursor-< {0 :a 1 :b 3 :d} 3)) [1 :b])
(assert-eq (cursor-map-prev! (map-cursor-< {0 :a 1 :b 3 :d} 4)) [3 :d])
```

#### `(map-cursor-> map v)`

Returns a new map cursor (see below), positioned right before the smallest entry of the map `map` whose key is strictly greater than the value `v`, or right behind the maximal entry of the map if no such entry exists.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (cursor-map-next! (map-cursor-> {0 :a 1 :b 3 :d} -1)) [0 :a])
(assert-eq (cursor-map-next! (map-cursor-> {0 :a 1 :b 3 :d} 0)) [1 :b])
(assert-eq (cursor-map-next! (map-cursor-> {0 :a 1 :b 3 :d} 1)) [3 :d])
(assert-eq (cursor-map-next! (map-cursor-> {0 :a 1 :b 3 :d} 2)) [3 :d])
(assert-throw (cursor-map-next! (map-cursor-> {0 :a 1 :b 3 :d} 3)) :cursor-end)
(assert-throw (cursor-map-next! (map-cursor-> {0 :a 1 :b 3 :d} 4)) :cursor-end)
```

#### `(map-cursor-<= map v)`

Returns a new map cursor (see below), positioned right behind the greatest entry of the map `map` whose key is less than or equal to the value `v`, or right before the minimal entry of the map if no such entry exists.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-throw (cursor-map-prev! (map-cursor-<= {0 :a 1 :b 3 :d} -1)) :cursor-end)
(assert-eq (cursor-map-prev! (map-cursor-<= {0 :a 1 :b 3 :d} 0)) [0 :a])
(assert-eq (cursor-map-prev! (map-cursor-<= {0 :a 1 :b 3 :d} 1)) [1 :b])
(assert-eq (cursor-map-prev! (map-cursor-<= {0 :a 1 :b 3 :d} 2)) [1 :b])
(assert-eq (cursor-map-prev! (map-cursor-<= {0 :a 1 :b 3 :d} 3)) [3 :d])
(assert-eq (cursor-map-prev! (map-cursor-<= {0 :a 1 :b 3 :d} 4)) [3 :d])
```

#### `(map-cursor->= map v)`

Returns a new map cursor (see below), positioned right before the smallest entry of the map `map` whose key is greater than or equal to the value `v`, or right behind the maximal entry of the map if no such entry exists.

Time: O(log n), where n is `(map-count map)`.

```pavo
(assert-eq (cursor-map-next! (map-cursor->= {0 :a 1 :b 3 :d} -1)) [0 :a])
    (assert-eq (cursor-map-next! (map-cursor->= {0 :a 1 :b 3 :d} 0)) [0 :a])
    (assert-eq (cursor-map-next! (map-cursor->= {0 :a 1 :b 3 :d} 1)) [1 :b])
    (assert-eq (cursor-map-next! (map-cursor->= {0 :a 1 :b 3 :d} 2)) [3 :d])
    (assert-eq (cursor-map-next! (map-cursor->= {0 :a 1 :b 3 :d} 3)) [3 :d])
    (assert-throw (cursor-map-next! (map-cursor->= {0 :a 1 :b 3 :d} 4)) :cursor-end)
```

### Map Cursor

A map cursor (created via `map-cursor-min`, `map-cursor-max`, `map-cursor-<`, `map-cursor->`, `map-cursor-<=` or `map-cursor->=`) is an opaque value that allows to efficiently step through the entries in a map. It maintains a current position as state: either "in between" two entries, or at the front or back.

#### `cursor-map-type`

The type symbol of map cursors.

```pavo
(assert-eq cursor-map-type (typeof (map-cursor-min {})))
```

#### `(cursor-map-next! cursor)`

Advances the map cursor by one element and returns an array containing the key and value of the entry over which it passed. If the starting position was at the back of the map, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of entries in the underlying map. Moving from the front to the back of the map is guaranteed to take amortized O(n).

```pavo
(let cursor (map-cursor-min {0 :a 1 :b 2 :c}) (do [
    (assert-eq (cursor-map-next! cursor) [0 :a])
    (assert-eq (cursor-map-next! cursor) [1 :b])
    (assert-eq (cursor-map-next! cursor) [2 :c])
    (assert-throw (cursor-map-next! cursor) :cursor-end)
    (assert-throw (cursor-map-next! cursor) :cursor-end)
]))
```

#### `(cursor-map-prev! cursor)`

Retreats the map cursor by one entry and returns an array containing the key and value of the entry over which it passed. If the starting position was at the front of the map, the position remains unchanged and this function throws `:cursor-end`.

Time: Worst-case O(log(n)), where n is the number of entries in the underlying map. Moving from the back to the front of the map is guaranteed to take amortized O(n).

```pavo
(let cursor (map-cursor-max {0 :a 1 :b 2 :c}) (do [
    (assert-eq (cursor-map-prev! cursor) [2 :c])
    (assert-eq (cursor-map-prev! cursor) [1 :b])
    (assert-eq (cursor-map-prev! cursor) [0 :a])
    (assert-throw (cursor-map-prev! cursor) :cursor-end)
    (assert-throw (cursor-map-prev! cursor) :cursor-end)
]))
```

### Symbols

Symbols are values that are only equal to themselves. There's nothing you can do with symbols except comparing them and generating new ones.

#### `(symbol)`

Returns a fresh symbol that is only equal to itself.

```pavo
(assert (let x (symbol) (= x x)))
(assert-not (= (symbol) (symbol)))
```

### Cells

A cell is a mutable storage location, a persistent identity that can refer to different values over time.

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

Updates the cell `cl` to now contain the value `x`. Returns `nil`.

```pavo
(assert-eq (cell-set (cell 42) 43) nil)
(assert-eq ((sf-lambda [x] (sf-do [(cell-set x 43) (cell-get x)])) (cell 42)) 43)
```

### Opaques

An opaque value is one whose internals are hidden. A consumer can operate on them via pre-provided functions. Each opaque value has an associated *type* (a symbol) so that opaques of different types can be distinguished.

#### `(opaque)`

This function allows user code to introduce new opaque types. It returns a map containing three entries:

- `:type` is mapped to the symbol that designates the new type
- `:hide` is mapped to a function that maps a value to an opaque value of the new type containing the input value
- `:unhide` is mapped to a function that maps an opaque value of the new type to the contained value (and throws a type error if the input is not an opaque value of the new type)

```pavo
(let o (opaque) (sf-do [
    (assert-eq ((map-get o :unhide) ((map-get o :hide) 42)) 42)
    (assert-eq (typeof ((map-get o :hide) 42)) (map-get o :type))
    (assert-throw ((map-get o :unhide) 42) {:tag :err-type})
]))
(assert-eq (= (map-get (opaque) :type) (map-get (opaque) :type)) false)
```

#### Equality and Ordering

Pavo defines a [total order](https://en.wikipedia.org/wiki/Total_order) over all values. The order is defined via the `cmp` function that takes two arguments and returns whether the first is `:less`, `:equal` or `greater` than the second.

Since pavo has deterministic semantics, the ordering can not be established according to implementation details such as the numeric value of pointers. Instead, whenever the ordering is essentially arbitrary, the creation order of the values is used.

The precise definition of the ordering is the reflexive, transitive closure of the following relation:

- `nil` < any bool
- `false` < `true`
- `true` < any int
- any int < any float
- any float < any keyword
- any keyword < any identifier
- any identifier < any symbol
- any symbol < any character
- any character < any string
- any string < any bytes
- any bytes < any array
- any array < any application
- any application < any set
- any set < any map
- any map < any function
- any function < any cell
- any cell < any opaque
- two ints are compared by the usual ordering on mathematical integers
- two floats are compared by the usual ordering on the precise rational numbers that they denote
- two keywords are compared [lexicographically](https://en.wikipedia.org/wiki/Lexicographical_order) based on the numeric [ASCII](https://en.wikipedia.org/wiki/ASCII) values of their characters
- two identifiers are compared lexicographically based on the numeric [ASCII](https://en.wikipedia.org/wiki/ASCII) values of their characters
- two symbols are equal if they are the same value, otherwise the one that was created first (through a call to `(symbol)` or `(opaque)`) is less than the other one
- two characters are compared by the usual ordering on natural numbers of the scalar values they denote
- two strings are compared [lexicographically](https://en.wikipedia.org/wiki/Lexicographical_order) based on the ordering on their characters
- two bytes are compared [lexicographically](https://en.wikipedia.org/wiki/Lexicographical_order) based on the ordering on their byte values
- two arrays are compared [lexicographically](https://en.wikipedia.org/wiki/Lexicographical_order) based on the ordering on the contained values
- two applications are compared [lexicographically](https://en.wikipedia.org/wiki/Lexicographical_order) based on the ordering on the contained values
- two sets are compared [lexicographically](https://en.wikipedia.org/wiki/Lexicographical_order) based on the ordering on the contained values, the order in which the items are compared is [from smallest to largest](https://en.wikipedia.org/wiki/Lexicographical_order#Functions_over_a_well-ordered_set)
- two maps are compared as follows (simple extension of lexicographical ordering):
  - the empty map is equal to the empty map
  - the empty map is less than any non-empty map
  - given two nonempty maps whose least keys are not equal, the map with the lesser least key is less than the other map
  - given two nonempty maps whose least keys are equal and whose corresponding values are not equal, the map with the lesser value is less than the other map
  - given two nonempty maps whose least keys are equal and whose corresponding values are equal, the ordering is the same as that on the two maps without their least entries
- two functions are equal if they are the same value, otherwise the one that was created first is less than the other one
  - builtin functions are less than any functions that were created dynamically
  - the ordering on the builtin functions is the lexicographical ordering on their identifiers (this choice is completely arbitrary, but has the nice property that a human doesn't need to look anything up)
- two cells are equal if they are the same value, otherwise the one that was created first (through a call to `(cell v)`) is less than the other one
- two opaques are equal if they are the same value, otherwise the one that was created first is less than the other one
  - builtin opaques are less than any opaques that were created dynamically
  - the ordering on the builtin opaques is the lexicographical ordering on their identifiers (this choice is completely arbitrary, but has the nice property that a human doesn't need to look anything up)

For examples, see the documentation for `(cmp v w)` below.

#### `(cmp v w)`

Returns `:<` if the value `v` is less than the value `w`, `:=` if they are equal, and `:>` if `v` is greater than `w`. The ordering relation is defined above.

```pavo
(assert-eq (cmp nil false) :<)
(assert-eq (cmp false true) :<)
(assert-eq (cmp true -1) :<)
(assert-eq (cmp 999 -1.2) :<)
(assert-eq (cmp 1.2 :zero) :<)
(assert-eq (cmp :bcd $abc) :<)
(assert-eq (cmp $abc (symbol)) :<)
(assert-eq (cmp (symbol) 'a') :<)
(assert-eq (cmp 'b' "a") :<)
(assert-eq (cmp "zzz" @[0]) :<)
(assert-eq (cmp @[1] [0]) :<)
(assert-eq (cmp [1 2 3] $(1)) :<)
(assert-eq (cmp $(1 2 3) @{1}) :<)
(assert-eq (cmp @{1 2 3} {1 2}) :<)
(assert-eq (cmp {1 2} cmp) :<)
(assert-eq (cmp cmp (cell 42)) :<)
(assert-eq (cmp (cell 42) ((map-get (opaque) :hide) 42)) :<)
(assert-eq (cmp -1 0) :<)
(assert-eq (cmp 0 1) :<)
(assert-eq (cmp -0 0) :=)
(assert-eq (cmp -1.0 0.0) :<)
(assert-eq (cmp 0.0 1.0) :<)
(assert-eq (cmp -0.0 0.0) :=)
(assert-eq (cmp :a :b) :<)
(assert-eq (cmp :a :bc) :<)
(assert-eq (cmp :aa :ab) :<)
(assert-eq (cmp :aa :b) :<)
(assert-eq (cmp $a $b) :<)
(assert-eq (cmp $a $bc) :<)
(assert-eq (cmp $aa $ab) :<)
(assert-eq (cmp $aa $b) :<)
(assert-eq (cmp (symbol) (symbol)) :<)
(assert-eq (cmp (map-get (opaque) :type) (symbol)) :<)
(assert-eq (cmp (symbol) (map-get (opaque) :type)) :<)
(assert-eq (cmp 'a' 'b') :<)
(assert-eq (cmp 'A' 'a') :<)
(assert-eq (cmp '#' 'ß') :<)
(assert-eq (cmp "a" "b") :<)
(assert-eq (cmp "a" "bc") :<)
(assert-eq (cmp "aa" "ab") :<)
(assert-eq (cmp "aa" "b") :<)
(assert-eq (cmp @[0] @[1]) :<)
(assert-eq (cmp @[0] @[1 2]) :<)
(assert-eq (cmp @[0 0] @[0 1]) :<)
(assert-eq (cmp @[0 0] @[1]) :<)
(assert-eq (cmp [0] [1]) :<)
(assert-eq (cmp [0] [1 2]) :<)
(assert-eq (cmp [0 0] [0 1]) :<)
(assert-eq (cmp [0 0] [1]) :<)
(assert-eq (cmp $(0) $(1)) :<)
(assert-eq (cmp $(0) $(1 2)) :<)
(assert-eq (cmp $(0 0) $(0 1)) :<)
(assert-eq (cmp $(0 0) $(1)) :<)
(assert-eq (cmp @{0} @{1}) :<)
(assert-eq (cmp @{0} @{1 2}) :<)
(assert-eq (cmp @{0 1} @{0 2}) :<)
(assert-eq (cmp @{0 1} @{2}) :<)
(assert-eq (cmp {} {}) :=)
(assert-eq (cmp {} {0 1}) :<)
(assert-eq (cmp {0 99} {1 2}) :<)
(assert-eq (cmp {0 1} {0 2}) :<)
(assert-eq (cmp {0 1, 2 3} {0 1, 2 4}) :<)
(assert-eq (cmp {0 1} {0 1, 2 3}) :<)
(assert-eq (cmp cmp cmp) :=)
(assert-eq (cmp app-apply cmp) :<)
(assert-eq (cmp cmp (sf-lambda [] nil)) :<)
(assert-eq (cmp (sf-lambda [] nil) (sf-lambda [] nil)) :<)
(assert-eq (cmp (cell 42) (cell 41)) :<)

(let o (opaque) (let hide (map-get o :hide) (do [
    (assert-eq (cmp (hide 42) (hide 41)) :<)
    (assert-eq (cmp cursor-arr-type (hide 42)) :<)
    (assert-eq (cmp cursor-app-type cursor-arr-type) :<)
])))
```

#### `(= v w)`

Returns `true` if `v` and `w` are equal, `false` otherwise. See `(cmp v w)` for more details.

```pavo
(assert-eq (= 0 0) true)
(assert-eq (= 0.0 -0.0) true)
(assert-eq (= 0 0.0) false)
```

#### `(< v w)`

Returns `true` if `v` is less than `w`, `false` otherwise.  See `(cmp v w)` for more details.

```pavo
(assert-eq (< 0 1) true)
(assert-eq (< false true) true)
(assert-eq (< true 0) true)
(assert-eq (< 42 0.1) true) # ints are less than floats in the order over all values
```

#### `(<= v w)`

Returns `true` if `v` is less than or equal to `w`, `false` otherwise.  See `(cmp v w)` for more details.

```pavo
(assert-eq (<= 0 1) true)
(assert-eq (<= 0 0) true)
(assert-eq (<= 42 0.1) true)
```

#### `(> v w)`

Returns `true` if `v` is greater than `w`, `false` otherwise.  See `(cmp v w)` for more details.

```pavo
(assert-eq (> 0 1) false)
(assert-eq (> false true) false)
(assert-eq (> true 0) false)
(assert-eq (> 42 0.1) false) # ints are less than floats in the order over all values
```

#### `(>= v w)`

Returns `true` if `v` is greater than or equal to `w`, `false` otherwise.  See `(cmp v w)` for more details.

```pavo
(assert-eq (>= 0 1) false)
(assert-eq (>= 0 0) true)
(assert-eq (>= 42 0.1) false)
```

### Code as Data

#### `(read s)`

If the string `s` is a pavo expression, returns the value denoted by that expression.

Throws `{:tag :err-not-expression}` if the string is not a valid pavo expression.

Time: O(n), where n is `(string-count <prefix>)`, where `<prefix>` is the longest prefix of `s` that is a pavo expression.

```pavo
(assert-eq (read "42") 42)
(assert-eq (read "(a) ") $(a))
(assert-throw (read "(a) b") {:tag :err-not-expression})
```

#### `(write v)`

Returns a string `s` such that `(read s)` equals the value `v`. The precise definition is given in appendix A.

Throws `{:tag :err-not-writable}` if no such string exists. This is the case if `v` is or contains a function, symbol, cell or opaque.
Throws `{:tag :err-collection-full}` if the resulting string would contain 2^63 or more elements.

Time: Linear in the length of the returned string. Yeah, that's not a proper definition...

```pavo
(assert-eq (write 42) "42")
(assert-eq (write $(a )) "(a)")
(assert-throw (write (symbol)) {:tag :err-not-writable})
```

#### `(check v options)`

Returns whether the value `v` passes the static checks of pavo (special form syntax and binding correctness), with the check-environment determined from the map `options` as follows:

- start out with a check-environment mapping the default toplevel values to false
- if the `options` map contains an entry with key `:remove`:
  - if the corresponding value is not a set, throw a type error
  - otherwise, for each value in the set (in ascending order):
    - if the value is not an identifier, throw a type error
    - otherwise, update the check-environment by removing the identifier if it was in there before
- then, if the `options` map contains an entry with key `:mutable`:
  - if the corresponding value is not a set, throw a type error
  - otherwise, for each value in the set (in ascending order):
    - if the value is not an identifier, throw a type error
    - otherwise, update the check-environment by mapping the identifier to true
- then, if the `options` map contains an entry with key `:immutable`:
  - if the corresponding value is not a set, throw a type error
  - otherwise, for each value in the set (in ascending order):
    - if the value is not an identifier, throw a type error
    - otherwise, update the check-environment by mapping the identifier to false

```pavo
(assert-eq (check 42 {}) true)
(assert-eq (check $int-add {}) true)
(assert-eq (check $int-add {:ignored-key 42}) true)
(assert-eq (check $int-add {:remove @{$int-add}}) false)
(assert-eq (check $foo {}) false)
(assert-eq (check $foo {:immutable @{$foo}}) true)
(assert-eq (check $int-add {:immutable @{$int-add} :remove @{$int-add}}) true)
(assert-eq (check $(sf-set! int-add 42) {}) false)
(assert-eq (check $(sf-set! int-add 42) {:mutable @{$int-add}}) true)
(assert-eq (check $(sf-set! int-add 42) {:immutable @{$int-add}, :mutable @{$int-add}}) false)

(assert-throw (check 42 {:remove :foo}) {:tag :err-type})
(assert-throw (check 42 {:remove @{:foo}}) {:tag :err-type})
(assert-throw (check 42 {:mutable :foo}) {:tag :err-type})
(assert-throw (check 42 {:mutable @{:foo}}) {:tag :err-type})
(assert-throw (check 42 {:immutable :foo}) {:tag :err-type})
(assert-throw (check 42 {:immutable @{:foo}}) {:tag :err-type})
```

#### `(eval v options)`

Performs static checks on the value v and then evaluates it, with the environments depending on the map `options` as follows:

- the evaluation environment starts out as the default toplevel environment (all bindings described in this document, all immutable).
- if the `options` map contains an entry with key `:remove`:
  - if the corresponding value is not a set, throw a type error
  - otherwise, for each value in the set (in ascending order):
    - if the value is not an identifier, throw a type error
    - otherwise, update the environment by removing the identifier if it was a key before
- then, if the `options` map contains an entry with key `:mutable`:
  - if the corresponding value is not a map, throw a type error
  - otherwise, for each entry in the map (in order of ascending keys):
    - if the key is not an identifier, throw a type error
    - otherwise, update the environment by mutably binding the key to its value
- then, if the `options` map contains an entry with key `:immutable`:
  - if the corresponding value is not a map, throw a type error
  - otherwise, for each entry in the map (in order of ascending keys):
    - if the key is not an identifier, throw a type error
    - otherwise, update the environment by immutably binding the key to its value

The value is then checked with the check-environment corresponding to the evaluation environment. If the check is unsuccessfull, throw `{:tag :err-static}`. Otherwise, evaluate `v` in the evaluation environment. If it evaluates successfully, return the result. If it throws some value `<x>`, throw `{:tag :err-eval, :cause <x>}`.

```pavo
(assert-eq (eval 42 {}) 42)
(assert-throw (eval $(sf-throw 17) {}) {:tag :err-eval, :cause 17})
(assert-eq (eval $int-add {}) int-add)
(assert-eq (eval $int-add {:ignored-key 42}) int-add)
(assert-throw (eval $int-add {:remove @{$int-add}}) {:tag :err-static})
(assert-throw (eval $foo {}) {:tag :err-static})
(assert-eq (eval $foo {:immutable {$foo 42}}) 42)
(assert-eq (eval $foo {:mutable {$foo 42}}) 42)
(assert-eq (eval $foo {:immutable {$foo 42} :mutable {$foo 43}}) 42)
(assert-eq (eval $int-add {:immutable {$int-add eval}}) eval)
(assert-eq (eval $int-add {:immutable {$int-add eval} :remove @{$int-add}}) eval)
(assert-throw (eval $(sf-set! int-add 42) {}) {:tag :err-static})
(assert-eq (eval $(sf-set! int-add 42) {:mutable {$int-add int-add}}) nil)
(assert-throw (eval $(sf-set! int-add 42) {:immutable {$int-add int-add}, :mutable {$int-add int-add}}) {:tag :err-static})

(assert-throw (eval 42 {:remove :foo}) {:tag :err-type})
(assert-throw (eval 42 {:remove @{:foo}}) {:tag :err-type})
(assert-throw (eval 42 {:mutable :foo}) {:tag :err-type})
(assert-throw (eval 42 {:mutable {:foo 42}}) {:tag :err-type})
(assert-throw (eval 42 {:immutable :foo}) {:tag :err-type})
(assert-throw (eval 42 {:immutable {:foo 42}}) {:tag :err-type})
```

#### `(expand v options)`

Performs macro expansion of the value `v`, according to the given options.

The definition environment (the environment in which macro definitions are evaluated) depends on the map `options` as follows (this is identical to `eval` with changed keys):

- the definition environment starts out as the default toplevel environment (all bindings described in this document, all immutable).
- if the `options` map contains an entry with key `:def-remove`:
  - if the corresponding value is not a set, throw a type error
  - otherwise, for each value in the set (in ascending order):
    - if the value is not an identifier, throw a type error
    - otherwise, update the environment by removing the identifier if it was a key before
- then, if the `options` map contains an entry with key `:def-mutable`:
  - if the corresponding value is not a map, throw a type error
  - otherwise, for each entry in the map (in order of ascending keys):
    - if the key is not an identifier, throw a type error
    - otherwise, update the environment by mutably binding the key to its value
- then, if the `options` map contains an entry with key `:def-immutable`:
  - if the corresponding value is not a map, throw a type error
  - otherwise, for each entry in the map (in order of ascending keys):
    - if the key is not an identifier, throw a type error
    - otherwise, update the environment by immutably binding the key to its value

The macro environment (the identifiers to expand and the corresponding macro values) depends on the map `options` as follows:

- the macro environment starts out as the default macro environment
- if the `options` map contains an entry with key `:macro-remove`:
  - if the corresponding value is not a set, throw a type error
  - otherwise, for each value in the set (in ascending order):
    - if the value is not an identifier, throw a type error
    - otherwise, update the environment by removing the identifier if it was a key before
- then, if the `options` map contains an entry with key `:macro-add`:
  - if the corresponding value is not a map, throw a type error
  - otherwise, for each entry in the map (in order of ascending keys):
    - if the key is not an identifier, throw a type error
    - otherwise, update the environment by mapping the key to its value

If the macro expansion succeeds, the expanded value is returned. If it errors, this function throws `{:tag :err-expand}`.

```pavo
(assert-eq (expand $(throw nil) {}) $(sf-throw nil))
(assert-eq (expand $(throw) {:macro-remove @{$throw}}) $(throw))
(assert-eq (expand $(foo 1 2) {:macro-add {$foo int-add}}) 3)
(assert-eq (expand $(macro a (sf-lambda [] foo) (a)) {:def-mutable {$foo 42}}) 42)
(assert-throw (expand $(macro a (sf-lambda [] int-max-val) (a)) {:def-remove @{$int-max-val}}) {:tag :err-expand})

(assert-throw (expand 42 {:macro-remove :foo}) {:tag :err-type})
(assert-throw (expand 42 {:macro-remove @{:foo}}) {:tag :err-type})
(assert-throw (expand 42 {:macro-add :foo}) {:tag :err-type})
(assert-throw (expand 42 {:macro-add {:foo 42}}) {:tag :err-type})
(assert-throw (expand 42 {:def-remove :foo}) {:tag :err-type})
(assert-throw (expand 42 {:def-remove @{:foo}}) {:tag :err-type})
(assert-throw (expand 42 {:def-mutable :foo}) {:tag :err-type})
(assert-throw (expand 42 {:def-mutable {:foo 42}}) {:tag :err-type})
(assert-throw (expand 42 {:def-immutable :foo}) {:tag :err-type})
(assert-throw (expand 42 {:def-immutable {:foo 42}}) {:tag :err-type})
```

#### `(exval v options)`

Semantically equivalent to `(eval (expand v options) options)`.

### Miscellaneous

#### `(typeof v)`

If the value `v` is opaque, returns its type symbol, otherwise returns a keyword indicating the type of `v`: `:nil`, `:bool`, `:int`, `:float`, `:char`, `:string`, `:bytes`, `:keyword`, `:identifier`, `:symbol`, `:function`, `:array`, `:application`, `:map`, `:set`, or `:cell`.

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
(let o (opaque) (
    assert-eq (typeof ((map-get o :hide) 42)) (map-get o :type)
    ))
```

#### `(not x)`

Returns `true` if `x` is `nil` or `false`, and `false` otherwise.

Equivalent to `(if x false true)`.

```pavo
(assert-eq (not nil) true)
(assert-eq (not false) true)
(assert-eq (not true) false)
(assert-eq (not 0) false)
(assert-eq (not not) false)
```

#### `(diverge v)`

Immediately and abnormally terminates the execution of the program. Semantically you can think of this as going into an infinite loop, but telling the outside world about it to save resources and give feedback to the programmer. In the pavo semantics, passing the value `v` does nothing whatsoever, but the runtime should somehow pass this value to the outside world for additional context.

Note that this is different from a program terminating through an uncaught throw and you should almost always throw instead of deliberately calling `diverge` (just as there are very few situations where you'd deliberately go into an effect-free infinite loop).

#### `(trace v)`

Returns the value `v`.

Outside the pavo semantics, when running in a debugging mode, this function is expected to communicate the value `v` to the programmer, e.g. by logging it. This should only be done in a debug mode, to prevent accidental semantic effects (which might for example happen if the logging output is later read into the program).

```pavo
(assert-eq (trace 42) 42)
```

### Macros

The functions that compute the builtin macros.

#### `(macro-set! v w)`

The `set!` macro is a shorthand to safe typing the `sf-` prefix of the `sf-set!` special form. `(set! v w)` expands to `(sf-set! v w)`.

```pavo
(assert-eq (macro-set! 42 43) $(sf-set! 42 43))
```

#### `(macro-quote v)`

The `quote` macro is a shorthand to safe typing the `sf-` prefix of the `sf-quote` special form. `(quote v)` expands to `(sf-quote v)`.

```pavo
(assert-eq (macro-quote 42) $(sf-quote 42))
```

#### `(macro-throw v)`

Returns `(sf-throw v)`.

```pavo
(assert-eq (macro-throw 42) $(sf-throw 42))
```

#### `(macro-if u v w)`

Returns `(sf-if u v w)`.

```pavo
(assert-eq (macro-if 0 1 2) $(sf-if 0 1 2))
```

#### `(macro-do [exprs...])`

If none of the expressions is an application whose first item is the keyword `:let`, return `(sf-do [exprs...])`. Otherwise, let `head...` be the expressions up to the first such application, let `<let>` be the application, and let `tail...` be all remaining expressions. If `<let>` does not have exactly three items, throw `{:tag :err-num-args}`. Otherwise, let `<pattern>` be the second item of `<let>` and let `<value>` be the third item of `<let>`. Return `(sf-do [head... (let <pattern> <value> <rec>)])`, where `<rec>` is the result of evaluating `(macro-do [tail...])` (propagating any errors).

```pavo
(assert-eq (macro-do []) $(sf-do []))
(assert-eq (macro-do [0]) $(sf-do [0]))
(assert-eq (macro-do [0 1 2]) $(sf-do [0 1 2]))
(assert-eq (macro-do [0 $(:let a 42) 2 $a]) $(sf-do [0 (let a 42 (sf-do [2 a]))]))
(assert-eq (do [0 (:let a 42) 2 a]) 42)
(assert-eq (macro-do [0 $(:let a 42)]) $(sf-do [0 (let a 42 (sf-do []))]))
(assert-throw (macro-do [$(:let a)]) {:tag :err-num-args})
```

#### `(macro-cond [exprs...])`

If there are exactly zero expressions in the array, returns `nil`. If there is exactly one expression `else`, returns `else`. If there are exactly two expressions `condition` and `then`, returns `(sf-if condition then nil)`. If there are exactly three expressions `condition`, `then` and `else`, returns `(sf-if condition then else)`. If there are four or more expressions `condition`, `then` and `rest...`, returns `(sf-if cond then <rec>)`, where `<rec>` is the result of evaluating `(macro-cond [rest...])` (propagating any errors).

```pavo
(assert-eq (macro-cond []) nil)
(assert-eq (macro-cond [0]) 0)
(assert-eq (macro-cond [0 1]) $(sf-if 0 1 nil))
(assert-eq (macro-cond [0 1 2]) $(sf-if 0 1 2))
(assert-eq (macro-cond [0 1 2 3]) $(sf-if 0 1 (sf-if 2 3 nil)))
(assert-eq (macro-cond [0 1 2 3 4]) $(sf-if 0 1 (sf-if 2 3 4)))
```

#### `(macro-let pattern v body)`

Returns `((lambda [pattern] body) v)`, effectively evaluating `body` in a context where the pattern has been destructured against the value `v`.

```pavo
(assert-eq (let a 42 a) 42)
(assert-eq (macro-let 0 1 2) $((lambda [0] 2) 1))
```

#### `(macro--> v [])` `(macro--> v [app])` `(macro--> v [app rest...])`

If the array is empty, returns `v`. If the array contains one item `app`, returns the result of evaluating `(app-insert app 1 v)`, throwing any error. If the array contains more than one item, `app` and `rest...`, returns the result of evaluating `(macro--> <spliced> [rest...])` (propagating any error), where `<spliced>` is the result of evaluating `(app-insert app 1 v)`, throwing any error.

In effect, this threads the value `v` through the applications, inserting it (or the result of the previous application) as the first argument of the next application.

```pavo
(assert-eq (-> 42 [
    (int-sub ,,, 2) # the commas are whitespace, used here to indicate the insertion point
    (int->float ,,,)
]) 40.0)

(assert-eq (macro--> 42 []) 42)
(assert-eq (macro--> 42 [$(int-sub 2)]) $(int-sub 42 2))
(assert-eq (macro--> 42 [$(int-sub 2) $(int->float)]) $(int->float (int-sub 42 2)))
(assert-throw (macro--> 42 [$int->float]) {:tag :err-type})
(assert-throw (macro--> 42 [$()]) {:tag :err-lookup})
```

#### `(macro-->> v [])` `(macro-->> v [app])` `(macro-->> v [app rest...])`

If the array is empty, returns `v`. If the array contains one item `app`, returns the result of evaluating `(app-insert app (app-count app) v)`, throwing any error. If the array contains more than one item, `app` and `rest...`, returns the result of evaluating `(macro--> <spliced> [rest...])` (propagating any error), where `<spliced>` is the result of evaluating `(app-insert app (app-count app) v)`, throwing any error.

In effect, this threads the value `v` through the applications, inserting it (or the result of the previous application) as the last argument of the next application.

```pavo
(assert-eq (->> 42 [
    (int-sub 2 ,,,) # the commas are whitespace, used here to indicate the insertion point
    (int->float ,,,)
]) -40.0)

(assert-eq (macro-->> 42 []) 42)
(assert-eq (macro-->> 42 [$(int-sub 2)]) $(int-sub 2 42))
(assert-eq (macro-->> 42 [$(int-sub 2) $(int->float)]) $(int->float (int-sub 2 42)))
(assert-eq (macro-->> 42 [$()]) $(42))
(assert-throw (macro-->> 42 [$int->float]) {:tag :err-type})
```

#### `(macro-as-> pattern v [])` `(macro-as-> pattern v [w])` `(macro-as-> pattern v [w rest...])`

If the array is empty, returns `v`. If the array contains one item `w`, returns `(let pattern v w)`. If the array contains more than one item, `w` and `rest...`, returns the result of evaluating `(macro-as-> pattern (let pattern v w) rest...)`, throwing any error.

In effect, this threads the value `v` through the applications, matching it against the pattern between each step to update the bindings with the result of the previous application.

```pavo
(assert-eq (as-> foo 42 [
    (int-sub foo 2)
    (int-sub 3 foo)
]) -37)

(assert-eq (macro-as-> $foo 42 []) 42)
(assert-eq (macro-as-> $foo 42 [$(int-sub foo 2)]) $(let foo 42 (int-sub foo 2)))
(assert-eq (macro-as-> $foo 42 [$(int-sub foo 2) $(int-sub 3 foo)]) $(let foo (let foo 42 (int-sub foo 2)) (int-sub 3 foo)))
```

#### `(macro-or [])` `(macro-or [v])` `(macro-or [v rest...])`

If the array is empty, returns `false`. If the array contains one item `v`, returns `v`. If the array contains two arguments or more items `v` and `rest...`, returns `(let <sym> v (sf-if <sym> <sym> <rec>))`, where `<sym>` is a freshly generated symbol, and `<rec>` is the result of evaluating `(macro-or [rest...])`.

```pavo
(assert-eq (macro-or []) false)
(assert-eq (macro-or [42]) 42)
(assert-eq (macro-or [nil]) nil)
(assert-eq (or [0 1]) 0)
(assert-eq (or [0 false]) 0)
(assert-eq (or [false 1]) 1)
(assert-eq (or [false nil]) nil)
(assert-eq (or [nil false 2]) 2)
(assert-eq (or [nil false 2 3]) 2)
```

#### `(macro-|| v w)`

Returns `(let <sym> v (sf-if <sym> <sym> w))`, where `<sym>` is a freshly generated symbol.

```pavo
(assert-eq (|| 0 1) 0)
(assert-eq (|| 0 false) 0)
(assert-eq (|| false 1) 1)
(assert-eq (|| false nil) nil)
```

#### `(macro-and [])` `(macro-and [v])` `(macro-and [v rest...])`

If the array is empty, returns `true`. If the array contains one item `v`, returns `v`. If the array contains two arguments or more items `v` and `rest...`, returns `(let <sym> v (sf-if <sym> <rec> <sym>))`, where `<sym>` is a freshly generated symbol, and `<rec>` is the result of evaluating `(macro-and [rest...])`.

```pavo
(assert-eq (macro-and []) true)
(assert-eq (macro-and [42]) 42)
(assert-eq (macro-and [nil]) nil)
(assert-eq (and [0 1]) 1)
(assert-eq (and [0 false]) false)
(assert-eq (and [false 1]) false)
(assert-eq (and [false nil]) false)
(assert-eq (and [nil false 2]) nil)
(assert-eq (and [nil false 2 3]) nil)
(assert-eq (and [3 2 false nil]) false)
```

#### `(macro-&& v w)`

Returns `(let <sym> v (sf-if <sym> w <sym>))`, where `<sym>` is a freshly generated symbol.

```pavo
(assert-eq (&& 0 1) 1)
(assert-eq (&& 0 false) false)
(assert-eq (&& false 1) false)
(assert-eq (&& false nil) false)
```

#### `(macro-quasiquote v)`

Returns an expression that evaluates to the value `v` (*not* to the result of evaluating `v`), except that:

- occurences of `(:unquote w)` within `v` evaluate to the result of evaluating `w`
- for each occurence of `(:unquote-splice w)` within an application within `v`, `w` is evaluated and the result is spliced into the containing application.
- occurences of `(:fresh-name some-name)` (names are identifiers or symbols) are replaced with a freshly generated symbol, all such forms with the same name receive the same symbol.

For a precise definition of this function, see the appendix B.

Reminder: \`v is a shorthand for `(quasiquote v)`, `~v` for `(:unquote v)`, `@~v` for `(:unquote-splice v)` and `(@name)` for `(:fresh-name name)` if `name` is a name.

```pavo
(assert-eq `42 42)
(assert-eq `foo $foo)
(assert-eq `[42 foo] [42 $foo])

(assert-eq `() $())
(assert-eq `(42 foo) (arr->app [42 $foo]))

(assert-eq `~(int-add 1 2) 3)
(assert-eq `[42 ~(int-add 1 2)] [42 3])
(assert-eq `(42 ~(int-add 1 2)) $(42 3))

(assert-eq `(0 @~$() 1) $(0 1))
(assert-eq `(0 @~$(1) 2) $(0 1 2))
(assert-eq `(0 @~$(1 2) 3) $(0 1 2 3))

(let expanded (macro-quasiquote $[@foo @bar @foo]) (do [
    (assert-eq (= (arr-get expanded 0) (arr-get expanded 1)) false)
    (assert-eq (arr-get expanded 0) (arr-get expanded 2))
]))
```

#### `(macro-match v p yay nay)`

The most basic pattern-based macro. Evaluates `yay` if `v` matches the pattern `p`, using the bindings introduced by the pattern. Otherwise, evaluates to `nay`.

Whether a value `v` matches a pattern `p` is determined as follows:

- if `p` is `nil`, a bool, an int, a float, a char, a string, a bytes or a keyword, `v` matches it if `(= v p)`
- if `p` is a name (identifier or symbol), the pattern matches and the name is immutably bound to `v`
- if `p` is an array, `v` matches it if:
  - `v` is an array
  - of the same length
  - the items of the value `v` match the subpatterns (items) of the pattern `p` (checking and introducing bindings is done in the order of the items in the arrays)
- if `p` is a map, the `v` matches it if:
  - `v` is a map
  - for each entry (`[k, vp]`) in `p`:
    - `v` contains an entry with key `k`
    - the value of this entry matches the pattern `vp`
- if `p` is an application `(:app rest...)`, `v` matches it if:
  - `v` is an application
  - of the same length there are values in `rest...`
  - the items of the value `v` match the subpatterns (items) of `rest...` (checking and introducing bindings is done in the order of the items in the arrays)
- if `p` is an application `(:mut some-name)`, the pattern matches and the name is mutably bound to `v`
- if `p` is an application `(:guard p_ exp)`, the value is matched against the pattern `p_` and if it matches `p_`, `exp` is evaluated (with the bindings introduced in `p_` in scope). If `exp` evaluates to `nil` or `false`, the value did not match `p`, otherwise it did.
- if `p` is an application `(:named some-name p_)`, the value is immutably bound to the name and then `v` is matched against `p_` (with the binding in scope)
- if `p` is an application `(:named (:mut some-name) p_)`, the value is mutably bound to the name and then `v` is matched against `p_` (with the binding in scope)
- if `p` is an application `(:map-exact some-map)`, works like a regular map pattern except that after checking whether `v` is a map, the number of entries of `v` is compared to the number of entries of the `some-map` pattern
- if `p` is an application `(:= w)`, `v` matches if `(= v w)` is `true`
- if `p` is an application `(:typeof w)`, `v` matches if `(= (typeof v) w)` is `true`

If `p` is neither of the above, throws `{:tag :err-pattern}`.

```pavo
(assert-eq (match 42 42 true false) true)
(assert-eq (match 42 43 true false) false)
(assert-eq (match [] 42 true false) false)

(assert-eq (match 42 n n false) 42)

(assert-eq (match [1] [2] true false) false)
(assert-eq (match [1] [1] true false) true)
(assert-eq (match [1 2] [1 2] true false) true)
(assert-eq (match [1] [a] a false) 1)
(assert-eq (match [1 2] [a b] (int-add a b) false) 3)
(assert-eq (match [1 2] [a 3] (int-add a 3) false) false)
(assert-eq (match [1 2] [3 b] (int-add 3 b) false) false)
(assert-eq (match [1] [a b] (int-add a b) false) false)
(assert-eq (match [1 2 3] [a b] (int-add a b) false) false)
(assert-eq (match 42 [a b] (int-add a b) false) false)

(assert-eq (match {0 42 1 43 2 44} {0 x 1 y} (int-add x y) false) 85)
(assert-eq (match {0 42 2 44} {0 x 1 y} (int-add x y) false) false)

(assert-eq (match $(1) (:app 2) true false) false)
(assert-eq (match $(1) (:app 1) true false) true)
(assert-eq (match $(1 2) (:app 1 2) true false) true)
(assert-eq (match $(1) (:app a) a false) 1)
(assert-eq (match $(1 2) (:app a b) (int-add a b) false) 3)
(assert-eq (match $(1 2) (:app a 3) (int-add a 3) false) false)
(assert-eq (match $(1 2) (:app 3 b) (int-add 3 b) false) false)
(assert-eq (match $(1) (:app a b) (int-add a b) false) false)
(assert-eq (match $(1 2 3) (:app a b) (int-add a b) false) false)
(assert-eq (match 42 (:app a b) (int-add a b) false) false)

(assert-eq (match 42 (:mut n) n false) 42)
(assert-eq (match 42 (:mut n) (do [(set! n (int-add n 1)) n]) false) 43)

(assert-eq (match 42 (:guard n (>= n 17)) n false) 42)
(assert-eq (match 16 (:guard n (>= n 17)) n false) false)
(assert-eq (match [42 3] [(:guard n (>= n 17)) (:guard n (< n 17))] n false) 3)
(assert-eq (match [42 43] [(:guard n (>= n 17)) (:guard m (< n m))] [m n] false) [43 42])

(assert-eq (match [42] (:named outer [inner]) outer false) [42])
(assert-eq (match [42] (:named outer [inner]) inner false) 42)
(assert-eq (match [42] (:named x [x]) x false) 42)
(assert-eq (match [42] (:named (:mut outer) [inner]) (do [(set! outer (arr-update outer 0 17)) outer]) false) [17])

(assert-eq (match {0 42 1 43} (:map-exact {0 x 1 y}) (int-add x y) false) 85)
(assert-eq (match {0 42 1 43 2 44} (:map-exact {0 x 1 y}) (int-add x y) false) false)
(assert-eq (match {0 42} (:map-exact {0 x 1 y}) (int-add x y) false) false)

(assert-eq (match 42 (:= 42) true false) true)
(assert-eq (match 42 (:= 43) true false) false)

(assert-eq (match 42 (:typeof :int) true false) true)
(assert-eq (match 42 (:typeof :float) true false) false)

(assert-throw (macro-match 42 @{} true false) {:tag :err-pattern})
(assert-throw (macro-match [42] [@{}] true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $() true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $(:llll) true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $(:mut) true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $(:mut a b) true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $(:guard a) true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $(:guard a b c) true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $(:named a) true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $(:named a b c) true false) {:tag :err-pattern})
(assert-throw (macro-match 42 $(:named [a] b) true false) {:tag :err-pattern})
(assert-throw (macro-match [42] $(:map-exact [a]) true false) {:tag :err-pattern})
```

#### `(macro-case v [p-1, yay-1, p-2, yay-2, ...])` `(macro-case v [p-1, yay-1, p-2, yay-2, ... else])`

Evaluates to the `yay-n` for the first `p-n` that matches `v` (bringing the bindings of `p-n` into scope). If no pattern matches evaluates to `else` if supplied, or `nil` otherwise.

```pavo
(assert-eq (case 0 [0 42 1 43 :else]) 42)
(assert-eq (case 1 [0 42 1 43 :else]) 43)
(assert-eq (case 2 [0 42 1 43 :else]) :else)
(assert-eq (case 2 [0 42 1 43]) nil)
```

#### `(macro-loop v [p-1, yay-1, p-2, yay-2, ...])`

Evaluate `v`. If it does not match any of the patterns, evaluates to `nil`. Otherwise, evaluate the corresponding `yay-n`, then repeat.

```pavo
(assert-eq (loop 0 [1 2]) nil)

(assert-eq
    (do [
        (:let (:mut sum) 0)
        (:let (:mut n) 0)
        (loop n [
            (:guard x (<= x 10000)) (do [
                    (set! sum (int-add sum x))
                    (set! n (int-add n 1))
                ])
            :foo :bar
        ])
        sum
    ])
    50005000 # 50005000 == 1 + 2 + ... + 10000
)
```

TODO pattern macros: lambda

#### `(macro-fn name [args...] body)`

Defines a function that takes the arguments `args...` and has the body `body`. When evaluating the body, the name `name` is immutably bound to the function itself. It's magic - well actually it's a fixpoint combinator, see appendix C for the precise definition.

```pavo
(assert-eq
    (
        # https://en.wikipedia.org/wiki/Triangular_number
        (fn triangular [acc n] (if
            (= x 0) acc
            (triangular (+ acc n) (- n 1))
        ))
        10000
    )
    50005000 # 50005000 == 1 + 2 + ... + 10000
)
```

#### `(macro-while cond body)`

A [while loop](https://en.wikipedia.org/wiki/While_loop): While `cond` evaluates to neither `nil` nor `false`, evaluates the `body` and then checks the condition again. When `cond` evalutes to `nil` or `false`, evalutes to `nil`.

Implemented as `((fn <sym> [] (sf-if cond (sf-do [body (<sym>)]) nil)))`, where `<sym>` is a new symbol.

```pavo
(assert-eq
    (do [
        (:let (:mut sum) 0)
        (:let (:mut n) 0)
        (while (<= n 10000) (do [
            (set! sum (int-add sum n))
            (set! n (int-add n 1))
        ]))
        sum
    ])
    50005000 # 50005000 == 1 + 2 + ... + 10000
)

# An infinite loop: (while true nil)
```


TODO

- `letfn`
- `lambda`
- `try`

- `case`, `loop` ?

## Appendix A: Precise Definition of `(write v)`

This section defines the return value of `(write v)` for any expression `v`, defined through structural induction (examples/tests are below).

### Base Cases

- `(= v nil)`: `"nil"`
- `(= v true)`: `"true"`
- `(= v false)`: `"false"`
- `(= (typeof v) :int)`:
  - `(>= v 0)`: The decimal representation of the integer (without sign).
  - `(< v 0)`: The minus sign `-` followed by the decimal representation of the absolute value of the integer.
- `(= (typeof v) :int)`: Printed as specified in [ECMAScript 2015](https://www.ecma-international.org/ecma-262/6.0/#sec-tostring-applied-to-the-number-type), except that if the resulting string would not be a float literal, the missing `.0` is inserted
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
- `(= (typeof v) :symbol)`: throw `{:tag :err-not-writable}`
- `(= (typeof v) :function)`: throw `{:tag :err-not-writable}`
- `(= (typeof v) :cell)`: throw `{:tag :err-not-writable}`
- `(= (typeof (typeof v)) :symbol)`: throw `{:tag :err-not-writable}`

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

(assert-eq (write 0.0) "0.0")
(assert-eq (write -0.0) "0.0")
(assert-eq (write 2.0E40) "2.0e+40")
(assert-eq (write 2.0E-40) "2.0e-40")

(assert-eq (write 'a') "'a'")
(assert-eq (write '"') "'\"'")
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
(assert-eq (write "\"") "\"\\\"\"")

(assert-eq (write @[ ]) "@[]")
(assert-eq (write @[ 0x11 ]) "@[17]")
(assert-eq (write @[1, 2]) "@[1 2]")

(assert-eq (write :foo) ":foo")

(assert-eq (write $foo) "foo")

(assert-throw (write (symbol)) {:tag :err-not-writable})
(assert-throw (write write) {:tag :err-not-writable})
(assert-throw (write ((map-get (opaque) :hide) 42)) {:tag :err-not-writable})

(assert-eq (write [ ]) "[]")
(assert-eq (write [ 2]) "[2]")
(assert-eq (write [ 2, 4 ]) "[2 4]")

(assert-eq (write $()) "()")
(assert-eq (write $(2)) "(2)")
(assert-eq (write $(2, 4)) "(2 4)")

(assert-eq (write @{}) "@{}")
(assert-eq (write @{1 }) "@{1}")
(assert-eq (write @{2 , 1  3}) "@{1 2 3}")

(assert-eq (write {}) "{}")
(assert-eq (write {1 nil}) "{1 nil}")
(assert-eq (write {1 nil 1 nil}) "{1 nil}")
(assert-eq (write {2 nil , 1 nil  3 nil}) "{1 nil 2 nil 3 nil}")
```

## Appendix B: Precise Definition of `(quasiquote v)`

TODO, refer to the reference implementation for now.

## Appendix C: Named Recursion

The builtin macros that provide named function that can recur by referring to their own names are implemented through [fixpoint combinators](https://en.wikipedia.org/wiki/Fixed-point_combinator#Fixed_point_combinators_in_lambda_calculus). This explanation assumes familiarity with how the strict Y combinator works. There are *many* resources on the web that explain it.

We write `Y-k` for the strict Y combinator that works for functions of arity k (the "regular" Y combinator is `Y-1`):

```pavo
(sf-lambda [g] (
    (sf-lambda [f] (f f)) # the M combinator
    (sf-lambda [x] (
        g
        (sf-lambda [arg-1 arg-2 ,,, arg-k] ((x x) arg-1 arg-2 ,,, arg-k))
    ))
))
```

Take for example a [tail-recursive](https://en.wikipedia.org/wiki/Tail_call) function to compute [triangular numbers](https://en.wikipedia.org/wiki/Triangular_number):

```pavo
(assert-eq
    (
        (fn triangular [acc n] (if
            (= n 0) acc
            (triangular (int-add acc n) (int-sub n 1))
        ))
        0 10000
    )
    50005000 # 50005000 == 1 + 2 + ... + 10000
)
```

It takes two arguments, so the corresponding fixpoint combinator is `Y-2`:

```pavo
(sf-lambda [g] (
    (sf-lambda [f] (f f))
    (sf-lambda [x] (
        g
        (sf-lambda [arg-1 arg-2] ((x x) arg-1 arg-2))
    ))
))
```

To define `triangular` without named recursion, we wrap it in a lambda that takes the recursion point as its single argument:

```pavo
(sf-lambda [triangular] (lambda [acc n] (if
    (= n 0) acc
    (triangular (int-add acc n) (int-sub n 1))
)))
```

The original `triangular` function is obtained by passing this form to `Y-2`:

```pavo
(
    (sf-lambda [g] (
        (sf-lambda [f] (f f))
        (sf-lambda [x] (
            g
            (sf-lambda [arg-1 arg-2] ((x x) arg-1 arg-2))
        ))
    ))
    (sf-lambda [triangular] (lambda [acc n] (if
        (= n 0) acc
        (triangular (int-add acc n) (int-sub n 1))
    )))
) # evaluates to a function that is equivalent to the recursive definition
```

The `(fn name [args...] body)` macro performs exactly this expansion: Count the number of args to find the appropriate fixpoint combinator, wrap the args and body in a lambda, enclose that lambda in a lambda that provides the recursion point, finally put the combinator and the function in an application. All names in the combinator are freshly generated symbols, the name for the recursion point is the `name` of the fn form.

The macro for defining mutually recursive functions is `letfn`, to make the example more interesting, we add an additional, unused argument to the definition of `odd?`:

```pavo
(assert-eq
    (letfn [
        (even? [n] (if (= n 0) true (odd? (int-sub n 1) nil)))
        (odd? [n _] (if (= n 0) false (even? (int-sub n 1))))
        ]
        (even? 10000)
    )
    true
)
```

The functions are wrapped in a lambda that supplies the recursion points:

```pavo
(sf-lambda [even? odd?] (lambda [n] (if (= n 0) true (odd? (int-sub n 1) nil)))))
(sf-lambda [even? odd?] (lambda [n _] (if (= n 0) false (even? (int-sub n 1))))
```

We now need a 2-adic Y combinator (since there are two functions) that is tailored to their specific function arities. We write `Y<1-2>` for this combinator, and `Y<arity1-arity2-...-arityk>` in general. Note that by design, these are not [poly-variadic](http://okmij.org/ftp/Computation/fixed-point-combinators.html#Poly-variadic) combinators. A poly-variadic combinator can handle an arbitrary number of functions, we deliberately use the appropriate special case. Since the `letfn` macro always knows the number of functions and their arities in advance, there's no need to use a generic map operation.

TODO

```pavo
(
    (sf-lambda [g] (
        (sf-lambda [f] (f f))
        (sf-lambda [x] (
            g
            (sf-lambda [arg-1 arg-2] ((x x) arg-1 arg-2))
        ))
    ))
    (sf-lambda [even? odd?] (lambda [n] (if (= n 0) true (odd? (int-sub n 1) nil)))))
    (sf-lambda [even? odd?] (lambda [n _] (if (= n 0) false (even? (int-sub n 1))))
) # evaluates to [even? odd?]
```

```pavo
(
    (sf-lambda [open-even? open-odd?] [
        (
            (sf-lambda [oe? oo?] (oe? oe? oo?))
            (sf-lambda [oe? oo?] (
                open-even?
                (sf-lambda [n] ((oe? oe? oo?) n))
                second-arg?
            ))
            (sf-lambda [oe? oo?] (
                open-odd?
                (sf-lambda [n _] ((oo? oe? oo?) n _))
                second-arg?
            ))
        )
        (
            (sf-lambda [oe? oo?] (oo? oe? oo?))
            (sf-lambda [oe? oo?] (
                open-even?
                (sf-lambda [n] ((oe? oe? oo?) n))
                second-arg?
            ))
            (sf-lambda [oe? oo?] (
                open-odd?
                (sf-lambda [n _] ((oo? oe? oo?) n _))
                second-arg?
            ))
        )
    ])
    (sf-lambda [even? odd?] (lambda [n] (if (= n 0) true (odd? (int-sub n 1) nil)))))
    (sf-lambda [even? odd?] (lambda [n _] (if (= n 0) false (even? (int-sub n 1))))
) # evaluates to [even? odd?]
```

```pavo
(
    (sf-lambda [open-triangular] [
        (
            (sf-lambda [f] (f f))
            (sf-lambda [x] (
                open-triangular
                (sf-lambda [arg-1 arg-2] ((x x) arg-1 arg-2))
            ))
        )
    ])
    (sf-lambda [triangular] (lambda [acc n] (if
        (= n 0) acc
        (triangular (int-add acc n) (int-sub n 1))
    )))
) # evaluates to a function that is equivalent to the recursive definition
```

```pavo
(
    (sf-lambda [e o] (
        (sf-lambda [f] (f f)) # M combinator
        (sf-lambda [x] (
            e
            (sf-lambda [e_n] ((x x) e_n))
            (sf-lambda [o_n o_foo] ((x x) o_n o_foo))
        ))
    ))
    (sf-lambda [even? odd?] (lambda [n] (if (= n 0) true (odd? (int-sub n 1) nil)))))
    (sf-lambda [even? odd?] (lambda [n foo] (if (= n 0) false (even? (int-sub n 1))))
) # evaluates to even?
```

---

TODO introduce special form sf-letfn

TODO `require` (dynamic linking, *not* loading)

TODO while and loop should evaluate to last loop body (or nil if none)

---

TODO

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
