## 0.2.8

- Improved handling of C pointers in OCaml finalizers

## 0.2.7

- Fixed possible double frees when GC is triggered after `free` has already been called

## 0.2.6

- Implement `std::error::Error` for generated `Error` type
- Added support for HIP backend 

## 0.2.5

- Add `Backend::from_env`

## 0.2.4

- Use `-O3` flag when compiling C files

## 0.2.3

- Add `of_array1`, `get_array1` and `values_array1` to OCaml Array bindings

## 0.2.2

- Automatically add `rerun-if-changed` to Rust build script
- Add `of_array` to generated OCaml `Array_*_*` types

## 0.2.1

- Fix capitalization of Bool in Rust generated code

## 0.2.0

- Added more methods to `Generate` trait to make it easier to add new
  languages
- Fix OCaml codegen for `u8` arrays
- Support boolean arrays in OCaml
- Support `f16` values in Rust using `half` crate

## 0.1.1

- Fix codegen for entries that return no value
- Fix codegen for structs and enums

## 0.1.0

- Initial release
