# Pavo Bootstrap

An interpreter for bootstrapping the pavo programming language, implemented in rust.

Pavo is a lisp dialect, featuring:

- hygienic, scoped macros
- immutable arrays, sets and maps
- exceptions
- an event loop accessed via lazy, cancellable promises
- object-orientation through append-only [prototypes](https://en.wikipedia.org/wiki/Prototype-based_programming)
- a module loading that acts as a [capability system](https://en.wikipedia.org/wiki/Capability-based_security) for effectful code
- the ability to opt into fully deterministic semantics

Beyond these features, pavo makes a few unorthodox choices in the set of built-in types. The author thinks they make sense, your mileage may vary. The closest relative of pavo is probably [clojure](https://clojure.org/index), other important influences include [lua](http://www.lua.org/), [E](http://www.erights.org/) and [node.js](https://nodejs.org/en/).
