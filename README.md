# futhark-bindgen

A [futhark](https://futhark-lang.org) binding generator

Currently supported languages:

- Python
- Rust
- OCaml

## Usage

### Command-line

```
$ futhark-bindgen test.fut test.rs # Rust output
$ futhark-bindgen test.fut test.ml # OCaml output
```

See the output of `futhark-bindgen --help` for more information

### Rust+build.rs

Add the following to your `Cargo.toml`

```toml
[build-depencies.futhark-bindgen]
git = "https://github.com/zshipko/futhark-bindgen"
default-features = false
features = ["build"]
```

In `build.rs`:

```rust
fn main() {
  let output = std::path::PathBuf::from(std::env::var("OUT_DIR"));
  let output = output.join("mylib.rs");
  futhark_bindgen::build(futhark_bindgen::Backend::C, "mylib.fut", "mylib.rs");
}
```

In `src/lib.rs`:

```rust
mod futhark {
  include!(concat!(env!("OUT_DIR"), "/mylib.rs"));
}
```

## OCaml+dune

Add the following to your `dune` file:

```
(library
  (name mylib)
  (public_name mylib)
  (libraries ctypes.foreign)
  (foreign_stubs (language c) (names futhark_output)))
```

Using OCaml you will have to manually generate the bindings, or call `futhark-bindgen"
from your `dune` file:

```sh
$ futhark-bindgen mylib.fut -o mylib.ml
```