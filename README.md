# futhark-bindgen

A [futhark](https://futhark-lang.org) binding generator

Currently supported languages:

- Rust
- OCaml

## Command-line

```
$ futhark-bindgen run test.fut test.rs # Rust output
$ futhark-bindgen run test.fut test.ml # OCaml output
```

See the output of `futhark-bindgen --help` for more information

## Example projects

- Rust: [https://github.com/zshipko/futhark-bindgen/tree/main/examples/rust](examples/rust)
- OCaml: [https://github.com/zshipko/futhark-bindgen/tree/main/examples/ocaml](examples/ocaml)
