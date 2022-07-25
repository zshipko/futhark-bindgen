# futhark-bindgen

A [futhark](https://futhark-lang.org) binding generator

Currently supported languages:

- Python
- Rust
- OCaml

## Usage

### Command-line

```
$ futhark-bindgen test.fut -o test.rs
$ futhark-bindgen test.fut -o test.ml
```

See the output of `futhark-bindgen --help` for more information

### build.rs

Add the following to your `Cargo.toml`

```toml
[build-depencies]
futhark-bindgen = {git = "https://github.com/zshipko/futhark-bindgen", features=["build"]}
```

In `build.rs`:

```rust
fn main() {
  let output = std::path::PathBuf::from(std::env::var("OUT_DIR"));
  let output = output.join("myfile.rs");
  futhark_bindgen::build(futhark_bindgen::Backend::C, "myfile.fut", "myfile.rs");
}
```

In `src/lib.rs`:

```rust
mod futhark {
  include!(concat!(env!("OUT_DIR"), "/myfile.rs"));
}
```
