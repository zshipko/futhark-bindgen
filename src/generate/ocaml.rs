use std::io::Write;

pub use crate::*;

pub struct OCaml {
    typemap: BTreeMap<String, String>,
}

const OCAML_TYPE_MAP: &[(&'static str, &'static str)] = &[
    ("i32", "int32"),
    ("i64", "int64"),
    ("f32", "float"),
    ("f64", "double"),
];

impl Default for OCaml {
    fn default() -> Self {
        let typemap = OCAML_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        OCaml { typemap }
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
        writeln!(config.output_file, "open Ctypes\nmodule Bindings = struct")?;
        writeln!(
            config.output_file,
            "external _stub: unit -> unit = \"futhark_context_new\""
        )?;
        writeln!(
            config.output_file,
            "  let context = typedef (ptr void) \"context\""
        )?;
        writeln!(
            config.output_file,
            "  let context_config = typedef (ptr void) \"context_config\""
        )?;
        writeln!(
            config.output_file,
            "  {}",
            self.foreign_function("futhark_context_new", "context", vec!["context_config"])
        )?;
        writeln!(
            config.output_file,
            "  {}",
            self.foreign_function("futhark_context_free", "int", vec!["context"])
        )?;

        writeln!(
            config.output_file,
            "  {}",
            self.foreign_function("futhark_context_sync", "int", vec!["context"])
        )?;

        writeln!(
            config.output_file,
            "  {}",
            self.foreign_function("futhark_context_config_new", "context_config", vec!["void"])
        )?;
        writeln!(
            config.output_file,
            "  {}",
            self.foreign_function("futhark_context_config_free", "int", vec!["context"])
        )?;

        for (name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    let elemtype = a.elemtype.to_str().to_string();
                    let ocaml_elemtype = self
                        .typemap
                        .get(&elemtype)
                        .cloned()
                        .unwrap_or_else(|| elemtype.clone());
                    let rank = a.rank;
                    let ocaml_name = format!("array_{elemtype}_{rank}d");
                    self.typemap.insert(name.clone(), ocaml_name.clone());
                    let elem_ptr = format!("ptr {ocaml_elemtype}");
                    writeln!(
                        config.output_file,
                        "  let {ocaml_name} = typedef (ptr void) \"array_{elemtype}_{rank}d\""
                    )?;
                    let mut new_args = vec!["context", &elem_ptr];
                    for _ in 0..rank {
                        new_args.push("int64_t");
                    }
                    writeln!(
                        config.output_file,
                        "  {}",
                        self.foreign_function(
                            &format!("futhark_new_{elemtype}_{rank}d"),
                            &ocaml_name,
                            new_args
                        )
                    )?;

                    writeln!(
                        config.output_file,
                        "  {}",
                        self.foreign_function(
                            &format!("futhark_values_{elemtype}_{rank}d"),
                            "int",
                            vec!["context", &ocaml_name, &elem_ptr]
                        )
                    )?;
                    writeln!(
                        config.output_file,
                        "  {}",
                        self.foreign_function(
                            &format!("futhark_free_{elemtype}_{rank}d"),
                            "int",
                            vec!["context", &ocaml_name]
                        )
                    )?;
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
                args.push(t);
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
            writeln!(
                config.output_file,
                "  {}",
                self.foreign_function(&entry.cfun, "int", args)
            )?;
        }

        writeln!(config.output_file, "end")?;

        writeln!(
            config.output_file,
            "type error = InvalidShape | Code of int"
        )?;

        writeln!(config.output_file, "open Bigarray")?;

        writeln!(config.output_file, "module Context = struct")?;
        writeln!(
            config.output_file,
            "  type t = {} handle: unit ptr; config: unit ptr {}",
            '{', '}'
        )?;
        writeln!(config.output_file, "  let free t = ignore (Bindings.futhark_context_free t.handle); ignore (Bindings.futhark_context_config_free t.config)")?;
        writeln!(config.output_file, "  let v () =")?;
        writeln!(
            config.output_file,
            "    let config = Bindings.futhark_context_config_new () in"
        )?;
        writeln!(
            config.output_file,
            "    let handle = Bindings.futhark_context_new config in"
        )?;

        writeln!(
            config.output_file,
            "    let t = {} handle; config {} in",
            '{', '}'
        )?;
        writeln!(config.output_file, "    Gc.finalise free t; t")?;
        writeln!(
            config.output_file,
            "  let sync t = let rc = Bindings.futhark_context_sync t.handle in if rc <> 0 then Error (Code rc) else Ok ()"
        )?;
        writeln!(config.output_file, "end")?;

        writeln!(
            config.output_file,
            "type futhark_array = {} ptr: unit ptr; shape: int array; ctx: Context.t {}",
            '{', '}'
        )?;

        for (name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    let rank = a.rank;
                    let elemtype = a.elemtype.to_str().to_string();
                    let ocaml_elemtype = self
                        .typemap
                        .get(&elemtype)
                        .cloned()
                        .unwrap_or_else(|| elemtype.clone());
                    let ocaml_name = self.typemap.get(name).unwrap();
                    writeln!(
                        config.output_file,
                        "module {} = struct",
                        ascii_titlecase(&ocaml_name),
                    )?;

                    writeln!(config.output_file, "  type t = futhark_array",)?;
                    writeln!(
                        config.output_file,
                        "  let free t = ignore (Bindings.futhark_free_{}_{}d t.ctx.Context.handle t.ptr)",
                        elemtype, rank,
                    )?;

                    // v
                    writeln!(config.output_file, "  let v ctx dims =")?;
                    writeln!(config.output_file, "    let data = CArray.make {ocaml_elemtype} (Array.fold_left ( * ) 1 dims) in")?;
                    write!(
                        config.output_file,
                        "    let ptr = Bindings.futhark_new_{elemtype}_{rank}d ctx.Context.handle (CArray.start data)",
                    )?;

                    for i in 0..rank {
                        write!(config.output_file, " (Int64.of_int dims.({i}))")?;
                    }
                    writeln!(config.output_file, " in")?;
                    writeln!(
                        config.output_file,
                        "    let t = {} ptr; ctx; shape = dims {} in",
                        '{', '}'
                    )?;
                    writeln!(config.output_file, "    Gc.finalise free t; t")?;

                    // of_bigarray

                    writeln!(config.output_file, "  let of_bigarray ctx ba =")?;
                    writeln!(config.output_file, "    let dims = Genarray.dims ba in")?;
                    write!(
                        config.output_file,
                        "    let ptr = Bindings.futhark_new_{}_{}d ctx.Context.handle (bigarray_start genarray ba)",
                        elemtype, rank
                    )?;

                    for i in 0..rank {
                        write!(config.output_file, " (Int64.of_int dims.({i}))")?;
                    }
                    writeln!(config.output_file, " in")?;
                    writeln!(
                        config.output_file,
                        "    let t = {} ptr; ctx; shape = dims {} in",
                        '{', '}'
                    )?;
                    writeln!(config.output_file, "    Gc.finalise free t; t")?;

                    // values

                    writeln!(config.output_file, "  let values t ba =")?;
                    writeln!(config.output_file, "    let dims = Genarray.dims ba in")?;
                    writeln!(config.output_file, "    if not (Array.for_all2 Int.equal t.shape dims) then Error (InvalidShape)")?;
                    writeln!(config.output_file, "    else")?;
                    writeln!(config.output_file, "      let rc = Bindings.futhark_values_{}_{}d t.ctx.Context.handle t.ptr (bigarray_start genarray ba) in", elemtype, rank)?;
                    writeln!(
                        config.output_file,
                        "      if rc <> 0 then Error (Code rc) else Ok ()"
                    )?;

                    writeln!(config.output_file, "  let shape t = t.shape")?;

                    // Close module
                    writeln!(config.output_file, "end")?;
                }
                manifest::Type::Opaque(_) => todo!(),
            }
        }

        writeln!(config.output_file, "module Entry = struct")?;
        for (name, entry) in &library.manifest.entry_points {
            write!(config.output_file, "  let {} ctx", name)?;

            for (i, _out) in entry.outputs.iter().enumerate() {
                write!(config.output_file, " out{i}")?;
            }

            for (i, _input) in entry.inputs.iter().enumerate() {
                write!(config.output_file, " input{i}")?;
            }

            writeln!(config.output_file, " =")?;
            write!(
                config.output_file,
                "    let rc = Bindings.futhark_entry_{} ctx.Context.handle",
                name
            )?;

            for (i, out) in entry.outputs.iter().enumerate() {
                let t = self.typemap.get(&out.r#type);
                let t = match t {
                    Some(t) => t.clone(),
                    None => out.r#type.clone(),
                };
                if t.contains("array_") {
                    write!(config.output_file, " out{i}.ptr")?;
                } else {
                    write!(config.output_file, " out{i}")?;
                }
            }

            for (i, input) in entry.inputs.iter().enumerate() {
                let t = self.typemap.get(&input.r#type);
                let t = match t {
                    Some(t) => t.clone(),
                    None => input.r#type.clone(),
                };
                if t.contains("array_") {
                    write!(config.output_file, " input{i}.ptr")?;
                } else {
                    write!(config.output_file, " input{i}")?;
                }
            }
            writeln!(config.output_file, " in")?;
            writeln!(
                config.output_file,
                "    if rc <> 0 then Error (Code rc) else Ok ()"
            )?;
        }
        writeln!(config.output_file, "end")?;

        Ok(())
    }
}
