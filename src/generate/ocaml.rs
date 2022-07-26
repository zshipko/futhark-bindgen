use std::io::Write;

pub use crate::*;

pub struct OCaml {
    typemap: BTreeMap<String, String>,
    ctypes_map: BTreeMap<String, String>,
    ba_map: BTreeMap<String, String>,
}

const OCAML_CTYPES_MAP: &[(&'static str, &'static str)] = &[
    ("i8", "int8_t"),
    ("u8", "uint8_t"),
    ("i16", "int16_t"),
    ("u16", "uint16_t"),
    ("i32", "int32_t"),
    ("u32", "uint32_t"),
    ("i64", "int64_t"),
    ("u64", "uint64_t"),
    ("f32", "float"),
    ("f64", "double"),
];

const OCAML_TYPE_MAP: &[(&'static str, &'static str)] = &[
    ("i8", "int"),
    ("u8", "int"),
    ("i16", "int"),
    ("u16", "int"),
    ("i32", "int32"),
    ("i64", "int64"),
    ("u32", "int32"),
    ("u64", "int64"),
    ("f32", "float"),
    ("f64", "float"),
];
const OCAML_BA_TYPE_MAP: &[(&'static str, &'static str)] = &[
    ("i8", "Bigarray.int8_signed_elt"),
    ("u8", "Bigarray.int8_unsigned_elt"),
    ("i16", "Bigarray.int16_signed_elt"),
    ("u16", "Bigarray.int16_unsigned_elt"),
    ("i32", "Bigarray.int32_elt"),
    ("i64", "Bigarray.int64_elt"),
    ("u32", "Bigarray.int32_elt"),
    ("u64", "Bigarray.int64_elt"),
    ("f32", "Bigarray.float32_elt"),
    ("f64", "Bigarray.float64_elt"),
];

impl Default for OCaml {
    fn default() -> Self {
        let typemap = OCAML_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        let ba_map = OCAML_BA_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        let ctypes_map = OCAML_CTYPES_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        OCaml {
            typemap,
            ba_map,
            ctypes_map,
        }
    }
}

impl OCaml {
    fn foreign_function(&mut self, name: &str, ret: &str, args: Vec<&str>) -> String {
        format!(
            "let {name} = Foreign.foreign \"{name}\" ({} @-> returning ({ret}))",
            args.join(" @-> ")
        )
    }
}

fn ascii_titlecase(s: &str) -> String {
    let mut s = s.to_string();
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
    s
}

impl Generate for OCaml {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error> {
        let mut mli_file = std::fs::File::create(config.output_path.with_extension("mli"))?;

        macro_rules! ml {
            ($fmt:expr $(, $arg:expr)*$(,)?) => {
                writeln!(config.output_file, $fmt $(, $arg)*)?;
            };
            ($($fmt:expr $(, $arg:expr)*$(,)?);+$(;)?) => {
                $(
                    ml!($fmt $(, $arg)*);
                )+
            }
        }

        macro_rules! mli {
            ($fmt:expr $(, $arg:expr)*$(,)?) => {
                writeln!(mli_file, $fmt $(, $arg)*)?;
            };
            ($($fmt:expr $(, $arg:expr)*$(,)?);+$(;)?) => {
                $(
                    mli!($fmt $(, $arg)*);
                )+
            }
        }

        macro_rules! ml_no_newline {
            ($fmt:expr $(, $arg:expr)*$(,)?) => {
                write!(config.output_file, $fmt $(, $arg)*)?;
            };
            ($($fmt:expr $(, $arg:expr)*$(,)?);+$(;)?) => {
                $(
                    ml!($fmt $(, $arg)*);
                )+
            }
        }

        macro_rules! mli_no_newline {
           ($fmt:expr $(, $arg:expr)*$(,)?) => {
                write!(mli_file, $fmt $(, $arg)*)?;
            };
            ($($fmt:expr $(, $arg:expr)*$(,)?);+$(;)?) => {
                $(
                    mli!($fmt $(, $arg)*);
                )+
            }
        }

        ml!(
            "open Ctypes";
            "";
            "module Bindings = struct";
            "  external _stub: unit -> unit = \"futhark_context_new\"";
            "  let context = typedef (ptr void) \"context\"";
            "  let context_config = typedef (ptr void) \"context_config\"";
            "  {}", self.foreign_function("futhark_context_new", "context", vec!["context_config"]);
            "  {}", self.foreign_function("futhark_context_free", "int", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_sync", "int", vec!["context"]);
            "  {}", self.foreign_function("futhark_context_config_new", "context_config", vec!["void"]);
            "  {}", self.foreign_function("futhark_context_config_free", "int", vec!["context"]);
        );

        for (name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    let elemtype = a.elemtype.to_str().to_string();
                    let ctypes_elemtype = self
                        .ctypes_map
                        .get(&elemtype)
                        .cloned()
                        .unwrap_or_else(|| elemtype.clone());
                    let rank = a.rank;
                    let ocaml_name = format!("array_{elemtype}_{rank}d");
                    self.typemap.insert(name.clone(), ocaml_name.clone());
                    let elem_ptr = format!("ptr {ctypes_elemtype}");
                    ml!("  let {ocaml_name} = typedef (ptr void) \"array_{elemtype}_{rank}d\"");
                    let mut new_args = vec!["context", &elem_ptr];
                    for _ in 0..rank {
                        new_args.push("int64_t");
                    }
                    ml!(
                        "  {}",
                        self.foreign_function(
                            &format!("futhark_new_{elemtype}_{rank}d"),
                            &ocaml_name,
                            new_args
                        );

                        "  {}",
                        self.foreign_function(
                            &format!("futhark_values_{elemtype}_{rank}d"),
                            "int",
                            vec!["context", &ocaml_name, &elem_ptr]
                        );
                        "  {}",
                        self.foreign_function(
                            &format!("futhark_free_{elemtype}_{rank}d"),
                            "int",
                            vec!["context", &ocaml_name]
                        )
                    );
                }
                manifest::Type::Opaque(_) => todo!(),
            }
        }

        for (_name, entry) in &library.manifest.entry_points {
            let mut args = vec!["context".to_string()];

            for out in &entry.outputs {
                let t = self.typemap.get(&out.r#type);
                let t = match t {
                    Some(t) => t.clone(),
                    None => out.r#type.clone(),
                };
                if t.contains("array_") {
                    args.push(t);
                } else {
                    args.push(format!("ptr {t}"));
                }
            }

            for input in &entry.inputs {
                let t = self.typemap.get(&input.r#type);
                let t = match t {
                    Some(t) => t.clone(),
                    None => input.r#type.clone(),
                };
                args.push(t);
            }

            let args = args.iter().map(|x| x.as_str()).collect();
            ml!("  {}", self.foreign_function(&entry.cfun, "int", args));
        }

        ml!("end");

        let error_t = "type error = InvalidShape | NullPtr | Code of int\nexception Error of error";
        ml!(
            "{}", error_t;
            "let () = Printexc.register_printer (function \
            | Error InvalidShape -> Some \"futhark error: invalid shape\" \
            | Error NullPtr -> Some \"futhark error: null pointer\" \
            | Error (Code c) -> Some (Printf.sprintf \"futhark error: code %d\" c) | _ -> None)"
        );

        mli!("{}", error_t); // mli

        ml!(
            "open Bigarray";
            "";
            "module Context = struct";
            "  type t = {} handle: unit ptr; config: unit ptr {}", '{', '}';
            "";
            "  let free t = \
            ignore (Bindings.futhark_context_free t.handle); \
            ignore (Bindings.futhark_context_config_free t.config)";
            "";
            "  let v () =";
            "    let config = Bindings.futhark_context_config_new () in";
            "    if is_null config then raise (Error NullPtr);";
            "    let handle = Bindings.futhark_context_new config in";
            "    if is_null handle then (ignore @@ Bindings.futhark_context_config_free config; raise (Error NullPtr));";
            "    let t = {} handle; config {} in", '{', '}';
            "    Gc.finalise free t; t";
            "";
            "  let sync t = let rc = Bindings.futhark_context_sync t.handle in if rc <> 0 then raise (Error (Code rc))";
            "end"
        );

        // mli
        mli!(
            "module Context: sig";
            "  type t";
            "  val v: unit -> t";
            "  val sync: t -> unit";
            "end");

        ml!(
            "type futhark_array = {} ptr: unit ptr; shape: int array; ctx: Context.t {}",
            '{',
            '}'
        );

        for (name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    let rank = a.rank;
                    let elemtype = a.elemtype.to_str().to_string();
                    let ctypes_elemtype = self
                        .ctypes_map
                        .get(&elemtype)
                        .cloned()
                        .unwrap_or_else(|| elemtype.clone());
                    let ocaml_name = self.typemap.get(name).unwrap();
                    let module_name = ascii_titlecase(&ocaml_name);
                    ml!(
                        "module {} = struct", &module_name;
                        "  type t = futhark_array";
                        "";
                        "  let free t = ignore (Bindings.futhark_free_{elemtype}_{rank}d t.ctx.Context.handle t.ptr)";
                        "";
                        "  let v ctx dims =";
                        "    let data = CArray.make {ctypes_elemtype} (Array.fold_left ( * ) 1 dims) in"
                    );
                    ml_no_newline!(
                        "    let ptr = Bindings.futhark_new_{elemtype}_{rank}d ctx.Context.handle (CArray.start data)"
                    );

                    for i in 0..rank {
                        ml_no_newline!(" (Int64.of_int dims.({i}))");
                    }
                    ml!(
                        " in";
                        "    if is_null ptr then raise (Error NullPtr);";
                        "    let t = {} ptr; ctx; shape = dims {} in", '{', '}';
                        "    Gc.finalise free t; t";
                        "";
                        "  let of_bigarray ctx ba =";
                        "    let dims = Genarray.dims ba in";
                        "    let ptr = Bindings.futhark_new_{elemtype}_{rank}d ctx.Context.handle (bigarray_start genarray ba)";
                    );

                    for i in 0..rank {
                        ml_no_newline!(" (Int64.of_int dims.({i}))");
                    }
                    ml!(
                        " in";
                        "    if is_null ptr then raise (Error NullPtr);";
                        "    let t = {} ptr; ctx; shape = dims {} in", '{', '}';
                        "    Gc.finalise free t; t";
                        "";
                        "  let values t ba =";
                        "    let dims = Genarray.dims ba in";
                        "    if not (Array.for_all2 Int.equal t.shape dims) then raise (Error (InvalidShape));";
                        "    let rc = Bindings.futhark_values_{elemtype}_{rank}d t.ctx.Context.handle t.ptr (bigarray_start genarray ba) in";
                        "    if rc <> 0 then raise (Error (Code rc))";
                        "";
                        "  let shape t = t.shape";
                        "end"
                    );

                    let ocaml_elemtype = self
                        .ctypes_map
                        .get(&elemtype)
                        .cloned()
                        .unwrap_or_else(|| elemtype.clone());

                    let ba_elemtype = self
                        .ba_map
                        .get(&elemtype)
                        .cloned()
                        .unwrap_or_else(|| elemtype.clone());

                    // mli
                    mli!(
                        "module {module_name}: sig";
                        "  type t";
                        "  val shape: t -> int array";
                        "  val v: Context.t -> int array -> t";
                        "  val of_bigarray: Context.t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t -> t";
                        "  val values: t -> ({ocaml_elemtype}, {ba_elemtype}, Bigarray.c_layout) Bigarray.Genarray.t -> unit";
                        "end"
                    );
                }
                manifest::Type::Opaque(_) => todo!(),
            }
        }

        ml!("module Entry = struct");
        mli!("module Entry: sig");
        for (name, entry) in &library.manifest.entry_points {
            ml_no_newline!("  let {} ctx", name);
            mli_no_newline!("  val {}: Context.t", name); // mli

            for (i, out) in entry.outputs.iter().enumerate() {
                let mut ocaml_elemtype = self
                    .typemap
                    .get(&out.r#type)
                    .cloned()
                    .unwrap_or_else(|| out.r#type.clone());

                // Transform into `Module.t`
                if ocaml_elemtype.contains("array_") {
                    ocaml_elemtype = ascii_titlecase(&ocaml_elemtype) + ".t"
                } else {
                    // Otherwise convert to Ctypes.ptr
                    ocaml_elemtype += " Ctypes.ptr";
                }

                let i = if entry.outputs.len() == 1 {
                    String::new()
                } else {
                    format!("{i}")
                };

                ml_no_newline!(" ~out{i}");
                mli_no_newline!(" -> out{i}:{ocaml_elemtype}"); // mli
            }

            for (i, input) in entry.inputs.iter().enumerate() {
                ml_no_newline!(" input{i}");

                let mut ocaml_elemtype = self
                    .typemap
                    .get(&input.r#type)
                    .cloned()
                    .unwrap_or_else(|| input.r#type.clone());

                // Transform into `Module.t`
                if ocaml_elemtype.contains("array_") {
                    ocaml_elemtype = ascii_titlecase(&ocaml_elemtype) + ".t"
                }

                mli_no_newline!(" -> {}", ocaml_elemtype); // mli
            }
            mli!(" -> unit"); // mli

            ml!(
                " =";
                "    let rc = Bindings.futhark_entry_{name} ctx.Context.handle";
            );

            for (i, out) in entry.outputs.iter().enumerate() {
                let t = self.typemap.get(&out.r#type);
                let t = match t {
                    Some(t) => t.clone(),
                    None => out.r#type.clone(),
                };

                let i = if entry.outputs.len() == 1 {
                    String::new()
                } else {
                    format!("{i}")
                };
                if t.contains("array_") {
                    ml_no_newline!(" out{i}.ptr");
                } else {
                    ml_no_newline!(" out{i}");
                }
            }

            for (i, input) in entry.inputs.iter().enumerate() {
                let t = self.typemap.get(&input.r#type);
                let t = match t {
                    Some(t) => t.clone(),
                    None => input.r#type.clone(),
                };
                if t.contains("array_") {
                    ml_no_newline!(" input{i}.ptr");
                } else {
                    ml_no_newline!(" input{i}");
                }
            }
            ml!(
                " in";
                "    if rc <> 0 then raise (Error (Code rc))"
            );
        }
        ml!("end");
        mli!("end");

        Ok(())
    }
}
