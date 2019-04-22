# Pavo Bootstrap

An interpreter for bootstrapping the pavo programming language, implemented in rust.

Pavo is a lisp dialect, featuring:

- hygienic, scoped macros
- immutable arrays, sets and maps
- exceptions
- an event loop accessed via lazy, cancellable promises
- a module loading mechanism that acts as a [capability system](https://en.wikipedia.org/wiki/Capability-based_security) for effectful code
- fully deterministic semantics
- tail recursion elemination (not for all tail calls, but according to a decidable syntactic criterium)

The closest relative of pavo is probably [clojure](https://clojure.org/index), other important influences include [lua](http://www.lua.org/), [E](http://www.erights.org/) and [node.js](https://nodejs.org/en/).

---

# Language Reference

This will move to a dedicated document at some point. The reference is for looking things up, it is not a guided introduction to the language (as of writing, no such document exists).

## Execution Model

This is a coarse overview to put the following, more detailed sections into perspective.

- parsing: utf-8 source code (and encoding of syntactic constructs) is parsed into an object (semantic constructs)
- expansion: transforme this object into a new one by expanding macros
- check: ensure that on the expanded object:
  - all identifiers are either binders, bound, or a special form
  - all special forms are well-formed
  - only mutable bindings are `set!`
- if the checks pass, enqueue the object on the event loop
- run: while the event loop is not empty:
  - take an enqueued object and:
  - eval: evaluate the object by reducing function applications and processing special forms
    - untypically, special forms can be shadowed by bindings
- the successful result of the execution is the object to which the last object on the event loop reduced to
- if any evaluation `throw`s, execution halts and the thrown object is the erroneous resultof the execution

## Special Forms

Unlike some other lisps, these may *not* be implemented as macros, as this would be a source of implementation-defined behavior for the `expand` function.

The syntax is defined in the same way as in the [clojure docs](https://clojure.org/reference/special_forms):

> Headings for the special forms informally describe the special form grammar using regular expression syntax: ? (optional), \* (0 or more), and + (1 or more). Non-terminals are denoted by *italics*.

Additionally, < and > are used to group elements for these regular expressions.

### (quote *expr*)

Evaluates to the expression, without evaluating it.

```pavo
(quote nil) # nil
(quote true) # true
(quote x) # x
(quote quote) # quote
(quote (foo)) # (foo)
(quote (quote x)) # (quote x)
(quote [x quote]) # [x quote]
```

### (do *expr*\*)

Evaluates the expressions in sequence, yields the result of the last one. Evaluates to `nil` if there are zero expressions.

The last *expr* is in tail position.

```pavo
(do :foo) # :foo
(do :foo :bar) # :bar
(do) # nil
```

### (if <*cond* *then*>\* *else*?)

Starting with the first pair *cond* and *then*: Evaluate *cond*. If it evaluates to a value other than `nil` or `false`, the form then evaluates to *then*. Otherwise, the next pair of *cond*/*then* is tried. If all conditions have been tried unsuccesfully, the form evaluates to *else* if supplied or to `nil` otherwise.

All *then* expressions and the *else* expression are in tail position.

```pavo
(if true :then1 :else) # :then1
(if false :then1 :else) # :else
(if false :then1) # nil
(if false :then1 42 :then2 :else) # :then2
(if false :then1 nil :then2 :else) # :else
(if :else) # :else
(if) # nil
```

### (case *cond* <*pattern* *then*>\* *else*?)

Evaluate the *cond* expression. Next, starting with the first pair of *pattern* and *then*: Try to destructure the *cond* against the *pattern*. If successful, the form then evaluates to *then*. Otherwise, the next pair of *cond*/*then* is tried. If all patterns have been tried unsuccesfully, the form evaluates to *else* if supplied or throws the value `{ :tag :no-case, :value cond }` otherwise (where `cond` is the evaluated *cond* expression).

All *then* expressions and the *else* expression are in tail position.

```pavo
(case 42 42 :42) # :42
(case 43 42 :42) # throws { :tag :no-case, :value 43 }
(case 43 42 :42 :else) # :else
(case 43) # throws { :tag :no-case, :value 43 }
(case 43 :else) # :else
(case 43
    42 :42
    43 :43
    :else
) # :43
```

### (throw *expr*?)

Throws the expression, or `nil` if none was supplied.

```pavo
(throw 42) # throws 42
(throw) # throws nil
```

### (try *expr* <*pattern* *then*>\*)

Evaluates to the value of the *epr*. If the expression throws, the thrown value is successively matched against the patterns. The form evaluates to the value of the *then* expression for the first pattern that matches. If no pattern matches, the "caught" value is rethrown unmodified (i.e. it passes right through - in particular debugging stacktraces should not treat it as having been caught).

All *then* expressions are in tail position.

```pavo
(try 42) # 42
(try 42 42 :caught-42) # 42
(try (throw 43)) # throws 43
(try (throw 43) 42 :caught-42) # throws 43
(try (throw 43) 43 :caught-43) # :caught-43
(try (throw 43)
    42 :caught-42
    43 :caught-43
) # :caught-43
(try (throw 44)
    41 :caught-41
    42 :caught-42
) # throws 43
```

### (fn *name*? *pattern* *body*)
### (fn *name*? (*pattern* *body*)\*)

Evaluates to a function. If a *name* identifier is given, this name is bound in the function body to the function itself, allowing for direct recursion. When invoking the function, the supplied against the first *pattern* (the only one in case of the paren-less syntax of the form). If it matches, the function then evaluates to the corresponding *body*, with identifiers bound according to the pattern. If the pattern does not match, the next *pattern*/*body* pair is tried. If all patterns have been tried unsuccesfully (or the form didn't define any in the first place), the value `{ :tag :no-case, :value arg }` is thrown (where `arg` is the argument to the function application).

```pavo
((fn [x] x) 42) # 42
((fn identity [x] 42)) # 42
((fn)) # throws { :tag :no-case, :value [] }
((fn assert-42 [42] nil) 42) # nil
((fn assert-42 [42] nil) [42]) # throws { :tag :no-case, :value [[42]] }
((fn assert-42-or-43
    ([42] :42)
    ([43] :43)
) 43) # :43
```

Pavo guarantees tail-call recursion elimination for application expressions in tail position whose first entry (i.e. the function expression) is the *name* of the function, but only if the *name* has not been rebound. This is only a small subset of all cases where recursion *could* be eliminated. Implementations are encouraged to restrict their tco to exactly this subset, everything else would only lead to broken programs that appear to work fine.

```pavo
((fn is-positive [n] (if
    (< n 0) false
    (= n 0) true
    (is-positive (- n 1))
)) 99999) # true, tco guarantees no stack overflow

((fn omega _ (omega))) # Never terminates, tco guarantees constant stack space usage.

((fn not-quite-omega _ (do
    (omega)
    42
))) # Stack overflow, the recursive call is not in tail position.

((fn should-not-be-omega _ (
    let w should-not-be-omega
        (w)
    )
))
# While tco *can* be applied here, it fails the syntactic criterion,
# w is not the name of the function. Thus this is allowed (and should)
# to cause a stack overflow.

((fn should-not-be-omega-either _ (
    let should-not-be-omega-either should-not-be-omega-either
        (should-not-be-omega-either)
    )
))
# This also fails the syntactic criterion, the name has been rebound.
```

TODO def, let, defn, letfn

## Patterns

Patterns are used to introduce bindings to an environment, destructuring some value in the process. In all composite patterns, identifiers that are bound later shadow equal earlier identifiers.

### Identifier

An identifier is a pattern. It matches any value, and the identifier is bound to that value.

```pavo
(case 42 x :then :else) # :then
(case 42 x x :else) # 42
```

### Atomic

Any of the atomic expressions (`nil`, bools, ints, floats, chars, strings, bytes and keywords) are patterns. They match only themselves (i.e. values that are equal to the value of the atomic expression) and do not bind any identifiers.

```pavo
(case 42 42 :then :else) # :then
(case 17 0x11 :then :else) # :then
(case 43 42 :then :else) # :else
```

### Array

An array pattern fails to match if the value is not an array of at least the same length as the pattern. If it is, it attempts to match the inner patterns from left to right, immediately failing to match if any of them fails to match.

```pavo
(case [42] [x] x :else) # 42
(case 42 [x] x :else) # :else
(case [42] [x, y] :else) # :else
(case [42, 43] [x] x :else) # 42
```

TODO set, map, app (starts with :app keyword), mut, immut, named (mut?), opt (with the ability to specify the default, defaulting to nil), alt, guard

TODO add examples where a guard/default is an infinite loop, but the evaluation order of the containing composite pattern means it is never executed (also for length comparisons before starting to match inner patterns)

<!-- ## Syntax (Expressions)

Expressions are the only syntactic category in pavo, every program is an expression. Pavo happens to be homoiconic, i.e. there is a total, injective function from expressions to objects. This function is called "read", the part of a pavo implementation that converts between utf-8 encoded syntax and the internal representation of expression is called the "reader". There may be multiple syntactic representations of the same expression (e.g. differences in whitespace). For each expression, there is exactly one canonic syntactic representation. Unless you want to compute hash values of programs, you probably don't need to care about that.

### Whitespace

Whitespace itself is not an expression, it is ignored by the reader, except that it can serve as a boundary between tokens (e.g. `[a b]` is an array of *two* identifiers, not of one). The ASCII space, newline, carriage return, and tab characters are whitespace, as is the comma `,`. Any sequence of characters beginning with `#` and ending with a newline is considered whitespace.

### Nil

`nil` is an expression.

### Bools

`true` and `false` are expressions.

### Ints

Ints have both a decimal and a hexadecimal representation. The canonic syntax of an int expression is its decimal representation (without a sign if it is positive).

A decimal int consists of an optional sign (`+` or `-`), followed by at least one decimal digit, such that the encoded integer is between `- 2^63` and `2^63 - 1` (inclusive). Numbers outside that range are *not* valid syntax.

A decimal int consists of an optional sign (`+` or `-`), followed by at least one hexadecimal digit (both uppercase and lowercase are allowed), such that the encoded integer is between `- 2^63` and `2^63 - 1` (inclusive). Numbers outside that range are *not* valid syntax.

### Floats

Syntactically, floats consist of an optional sign (`+` or `-`), followed by at least one decimal digit, followed by a dot (`.`), followed by either:

- a sequence of decimal digits, or
- an optional sign (`+` or `-`), followed by either `e` or `E`, followed by at least one decimal digit

This syntax is then interpreted as a rational number and rounded ([round-to-even](https://en.wikipedia.org/wiki/Rounding#Round_half_to_even)) to an IEEE754 64 bit float. If the result is negative zero, the expression is positive zero instead (that is to say, pavo only has *one* zero, and is thus *not* strictly IEEE754 compatible). If the result is an infinity, the syntax is *not* a valid expression (pavo doesn't have inifinities either).

The canonic syntax of a float is obtained by following the rules given [here](https://spec.scuttlebutt.nz/feed/datamodel.html#signing-encoding-floats), except that the `-6 < n <= 0` becomes a `-5 < n <= 0` case instead (a change that just so happens to reduce the maximum length of a canonical float expression from 25 to 24 bytes).

### Chars

A char is a [Unicode scalar value](http://www.unicode.org/glossary/#unicode_scalar_value) (*not* a [Unicode code point](http://www.unicode.org/glossary/#code_point)).

A char can be encoded either literally or through an escape sequence. The literal encoding can be used for all chars other than `'` (`0x27`) and `\` (`0x5c`) and consists of a `'` (`0x27`), followed by the utf-8 encoding of the Unicode scalar value, followed by another `'` (`0x27`). The escape sequence encoding consists of a `'` (`0x27`), followed by an escape sequence, followed by another `'` (`0x27`). The following escape sequences are defined:

- `\'` for the char `'` (`0x27`)
- `\\` for the char `\` (`0x5c`)
- `\t` for the char `horizontal tab` (`0x09`)
- `\n` for the char `new line` (`0x0a`)
- `\0` for the char `null` (`0x00`)
- `\{DIGITS}`, where `DIGITS` is the ASCII decimal representation of any scalar value. `DIGITS` must consist of one to six characters.

Literals or escape sequences that do not correspond to a Unicode scalar value are *not* valid expressions.

The canonic syntax for the character `'` is `'\''`, the canonic syntax for the character `\` is `'\\'`, the canonic syntax for all other characters is their literal encoding.

### Strings

A string is an ordered sequence of [Unicode scalar values](http://www.unicode.org/glossary/#unicode_scalar_value) whose [utf-8](https://en.wikipedia.org/wiki/UTF-8) encoding takes up no more than `(2^63) - 1` bytes.

A string is encoded as a `"` (`0x22`), followed by up to `(2^63) - 1` bytes worth of character encodings (see next paragraph), followed by another `"` (`0x22`).

Each character can either be encoded literally or through an escape sequence. The literal encoding can be used for all scalar values other than `"` (`0x22`) and `\` (`0x5c`) and consists of the utf-8 encoding of the scalar value. Alternatively, any of the following escape sequences can be used:

- `\"` for the character `"` (`0x22`)
- `\\` for the character `\` (`0x5c`)
- `\t` for the character `horizontal tab` (`0x09`)
- `\n` for the character `new line` (`0x0a`)
- `\0` for the character `null` (`0x00`)
- `\{DIGITS}`, where `DIGITS` is the ASCII decimal representation of any scalar value. `DIGITS` must consist of one to six characters.

Strings that contain a literal or an escape sequence that does not correspond to a Unicode scalar value are *not* valid expressions. In particular, Unicode code points that are not scalar values are not allowed, even when they form valid surrogate pairs.

The canonic syntax for a string is obtained by encoding the character `"` as `'\"'`, the character `\` as `'\\'`, and all other characters as utf-8.

### Identifiers

Any sequence of length at least one and at most 255 of the characters `!`, `*`, `+`, `-`, `_`, `?`, `<`, `>`, `=` and the ASCII alphanumerics is an identifier, except for those starting with a decimal digit and the sequences `nil`, `true` and `false`.

### Keywords

The character `:` followed by any sequence of length at least one and at most 255 of the characters `!`, `*`, `+`, `-`, `_`, `?`, `<`, `>`, `=` and the ASCII alphanumerics is a keyword. -->
