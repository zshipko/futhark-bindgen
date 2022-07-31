# futhark-bindgen

A [Futhark](https://futhark-lang.org) binding generator.

`futhark-bindgen` uses the [manifest](https://futhark.readthedocs.io/en/latest/c-api.html#manifest) created by Futhark 
to generate bindings for multiple languages.

Supported languages:

- Rust
- OCaml

## Installation

With `cargo`:

```
$ cargo install futhark-bindgen
```

From source:

```
$ make install PREFIX=~/.local/bin
```

## Command-line usage

```
$ futhark-bindgen run test.fut test.rs # Rust output
$ futhark-bindgen run test.fut test.ml # OCaml output
```

See the output of `futhark-bindgen --help` for more information

## Example projects

- Rust: [https://github.com/zshipko/futhark-bindgen/tree/main/examples/rust](examples/rust)
- OCaml: [https://github.com/zshipko/futhark-bindgen/tree/main/examples/ocaml](examples/ocaml)
