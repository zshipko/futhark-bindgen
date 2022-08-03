# futhark-bindgen

<a href="https://crates.io/crates/futhark-bindgen">
    <img src="https://img.shields.io/crates/v/futhark-bindgen.svg">
</a>

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
$ futhark-bindgen run test.fut test.rs # Rust output to ./test.rs
$ futhark-bindgen run test.fut test.ml # OCaml output to ./test.ml
```

The `--backend` flag can be used to select which Futhark backend to use: `c`, `multicore`,
`cuda`, `opencl` or `ispc`

See the output of `futhark-bindgen --help` for more information

## Example projects

- [https://github.com/zshipko/futhark-bindgen/tree/main/examples/rust](Rust)
- [https://github.com/zshipko/futhark-bindgen/tree/main/examples/ocaml](OCaml)
