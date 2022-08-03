# Rust futhark-bindgen example

Notes about using `futhark-bindgen` with OCaml and `dune`:

- Add your futhark source file to your source directory
- Update your dune file to include the rule for generating the OCaml files and linking
  the C source and the required libraries:
```
(rule
    (targets example.c example.ml example.mli)
    (deps example.fut)
     (action
         (run futhark-bindgen run --backend opencl example.fut example.ml)))

(library
    (name example)
    (public_name futhark-bindgen-example)
    (modules example)
    (libraries ctypes ctypes.foreign)
    (library_flags -linkall -cclib -lOpenCL)
    (foreign_stubs (language c) (names example)))
```