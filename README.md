# Pavo Bootstrap

An interpreter for the pavo programming language, implemented in rust.

Pavo is a homoiconic, dynamically typed language with deterministic semantics. It is defined in the [reference.md](./reference.md).

**~~Status: Prerelease, will not be considered released until there is a guide-level explanation of the language and better documentation in general. The code quality of the implementation is deliberately low, this is not intended to be used beyond bootstrapping. This _is_ a working pavo implementation however.~~**

**Status: Please stand by while I defile everything that is holy and convert the language to a C-like syntax. I'm not joking, I decided against the complexity of a macro system. Long live computable compilation functions!**

Usage: `cargo run -- path/to/pavo/file.pavo`

## Implementation Specifics of Note

- `(require v opts)` requires the first argument to be a string, it is interpreted as a path from which a pavo file is loaded
- this implementation does not add any toplevel values/macros beyond those required by the language definition
  - in particular, there is currently no way to do I/O or cause side effects (except for `(trace v)`)
- the time complexity of the cursor operations is O(log(n)), not O(1) as required by the spec
- the time complexity of splitting and slicing sets and maps is O(n), not O(log(n)) as required by the spec

## Contributing

All contributions are welcome, not just those in the form of code patches - giving instructions to machines is often the easy part. Any questions in particular are encouraged, as usually all sides end up improving their understanding. Please don't let worries about your skill level get in the way of contributing. If you end up learning something, your contribution was a success.

If however you think that there is room for disrespect, personal attacks, or any other behavior that goes against this project's [code of conduct](./CODE_OF_CONDUCT.md), go spend your valuable time "helping" some other project instead.

## License

[AGPL-3.0](./LICENSE)
