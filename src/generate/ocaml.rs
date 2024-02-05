use std::io::Write;

use crate::generate::{convert_struct_name, first_uppercase};
use crate::*;

/// OCaml codegen
pub struct OCaml {
    typemap: BTreeMap<String, String>,
    ctypes_map: BTreeMap<String, String>,
    ba_map: BTreeMap<String, (String, String)>,
    mli_file: std::fs::File,
}

const OCAML_CTYPES_MAP: &[(&str, &str)] = &[
    ("i8", "char"),
    ("u8", "uint8_t"),
    ("i16", "int16_t"),
    ("u16", "uint16_t"),
    ("i32", "int32_t"),
    ("u32", "uint32_t"),
    ("i64", "int64_t"),
    ("u64", "uint64_t"),
    ("f16", ""), // No half type in OCaml
    ("f32", "float"),
    ("f64", "double"),
    ("bool", "bool"),
];

const OCAML_TYPE_MAP: &[(&str, &str)] = &[
    ("i8", "char"),
    ("u8", "UInt8.t"),
    ("i16", "int"),
    ("u16", "UInt16.t"),
    ("i32", "int32"),
    ("i64", "int64"),
    ("u32", "UInt32.t"),
    ("u64", "UInt64.t"),
    ("f16", ""), // No half type in OCaml
    ("f32", "float"),
    ("f64", "float"),
    ("bool", "bool"),
];

const OCAML_BA_TYPE_MAP: &[(&str, (&str, &str))] = &[
    ("i8", ("int", "Bigarray.int8_signed_elt")),
    ("u8", ("int", "Bigarray.int8_unsigned_elt")),
    ("i16", ("int", "Bigarray.int16_signed_elt")),
    ("u16", ("int", "Bigarray.int16_unsigned_elt")),
    ("i32", ("int32", "Bigarray.int32_elt")),
    ("i64", ("int64", "Bigarray.int64_elt")),
    ("u32", ("int32", "Bigarray.int32_elt")),
    ("u64", ("int64", "Bigarray.int64_elt")),
    ("f16", ("", "")), // No half Bigarray kind
    ("f32", ("float", "Bigarray.float32_elt")),
    ("f64", ("float", "Bigarray.float64_elt")),
    ("bool", ("int", "Bigarray.int8_unsigned_elt")),
];

fn type_is_array(t: &str) -> bool {
    t.contains("array_f") || t.contains("array_i") || t.contains("array_u") || t.contains("array_b")
}

fn type_is_opaque(t: &str) -> bool {
    t.contains(".t")
}

fn ba_kind(t: &str) -> String {
    let mut s = t.strip_suffix("_elt").unwrap().to_string();

    if let Some(r) = s.get_mut(8..9) {
        r.make_ascii_uppercase();
    }

    s
}

impl OCaml {
    /// Create new OCaml codegen instance
    pub fn new(config: &Config) -> Result<Self, Error> {
        let typemap = OCAML_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        let ba_map = OCAML_BA_TYPE_MAP
            .iter()
            .map(|(a, (b, c))| (a.to_string(), (b.to_string(), c.to_string())))
            .collect();

        let ctypes_map = OCAML_CTYPES_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();

        let mli_path = config.output_path.with_extension("mli");
        let mli_file = std::fs::File::create(mli_path)?;
        Ok(OCaml {
            typemap,
            ba_map,
            ctypes_map,
            mli_file,
        })
    }

    fn foreign_function(&mut self, name: &str, ret: &str, args: Vec<&str>) -> String {
        format!(
            "let {name} = fn \"{name}\" ({} @-> returning ({ret}))",
            args.join(" @-> ")
        )
    }

    fn get_ctype(&self, t: &str) -> String {
        let x = self
            .ctypes_map
            .get(t)
            .cloned()
            .unwrap_or_else(|| t.to_string());
        if x.is_empty() {
            panic!("Unsupported type: {t}");
        }
        x
    }

    fn get_type(&self, t: &str) -> String {
        let x = self
            .typemap
            .get(t)
            .cloned()
            .unwrap_or_else(|| t.to_string());
        if x.is_empty() {
            panic!("Unsupported type: {t}");
        }
        x
    }

    fn get_ba_type(&self, t: &str) -> (String, String) {
        let x = self.ba_map.get(t).cloned().unwrap();
        if x.0.is_empty() {
            panic!("Unsupported type: {t}");
        }
        x
    }
}

impl Generate for OCaml {
    fn bindings(&mut self, pkg: &Package, config: &mut Config) -> Result<(), Error> {
        writeln!(self.mli_file, "(* Generated by futhark-bindgen *)\n")?;
        writeln!(config.output_file, "(* Generated by futhark-bindgen *)\n")?;

        let mut generated_foreign_functions = Vec::new();
        match pkg.manifest.backend {
            Backend::Multicore => {
                generated_foreign_functions.push(format!(
                    "  {}",
                    self.foreign_function(
                        "futhark_context_config_set_num_threads",
                        "void",
                        vec!["context_config", "int"]
                    )
                ));
            }
            Backend::CUDA | Backend::OpenCL => {
                generated_foreign_functions.push(format!(
                    "  {}",
                    self.foreign_function(
                        "futhark_context_config_set_device",
                        "void",
                        vec!["context_config", "string"]
                    )
                ));
            }
            _ => (),
        }

        for (name, ty) in &pkg.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    let elemtype = a.elemtype.to_str().to_string();
                    let ctypes_elemtype = self.get_ctype(&elemtype);
                    let rank = a.rank;
                    let ocaml_name = format!("array_{elemtype}_{rank}d");
                    self.typemap.insert(name.clone(), ocaml_name.clone());
                    self.ctypes_map.insert(name.clone(), ocaml_name.clone());
                    let elem_ptr = format!("ptr {ctypes_elemtype}");
                    generated_foreign_functions.push(format!(
                        "  let {ocaml_name} = typedef (ptr void) \"{ocaml_name}\""
                    ));
                    let mut new_args = vec!["context", &elem_ptr];
                    new_args.resize(rank as usize + 2, "int64_t");
                    generated_foreign_functions.push(format!(
                        "  {}",
                        self.foreign_function(&a.ops.new, &ocaml_name, new_args)
                    ));
                    generated_foreign_functions.push(format!(
                        "  {}",
                        self.foreign_function(
                            &a.ops.values,
                            "int",
                            vec!["context", &ocaml_name, &elem_ptr]
                        )
                    ));
                    generated_foreign_functions.push(format!(
                        "  {}",
                        self.foreign_function(&a.ops.free, "int", vec!["context", &ocaml_name])
                    ));
                    generated_foreign_functions.push(format!(
                        "  {}",
                        self.foreign_function(
                            &a.ops.shape,
                            "ptr int64_t",
                            vec!["context", &ocaml_name]
                        )
                    ));
                }
                manifest::Type::Opaque(ty) => {
                    let futhark_name = convert_struct_name(&ty.ctype);
                    let mut ocaml_name = futhark_name
                        .strip_prefix("futhark_opaque_")
                        .unwrap()
                        .to_string();
                    if ocaml_name.chars().next().unwrap().is_numeric() || name.contains(' ') {
                        ocaml_name = format!("type_{ocaml_name}");
                    }

                    self.typemap
                        .insert(name.clone(), format!("{}.t", first_uppercase(&ocaml_name)));
                    self.ctypes_map.insert(name.to_string(), ocaml_name.clone());
                    generated_foreign_functions.push(format!(
                        "  let {ocaml_name} = typedef (ptr void) \"{futhark_name}\""
                    ));

                    let free_fn = &ty.ops.free;
                    generated_foreign_functions.push(format!(
                        "  {}",
                        self.foreign_function(free_fn, "int", vec!["context", &ocaml_name])
                    ));

                    let record = match &ty.record {
                        Some(r) => r,
                        None => continue,
                    };

                    let new_fn = &record.new;
                    let mut args = vec!["context".to_string(), format!("ptr {ocaml_name}")];
                    for f in record.fields.iter() {
                        let cty = self
                            .ctypes_map
                            .get(&f.r#type)
                            .cloned()
                            .unwrap_or_else(|| f.r#type.clone());

                        // project function
                        generated_foreign_functions.push(format!(
                            "  {}",
                            self.foreign_function(
                                &f.project,
                                "int",
                                vec!["context", &format!("ptr {cty}"), &ocaml_name]
                            )
                        ));

                        args.push(cty);
                    }
                    let args = args.iter().map(|x| x.as_str()).collect();
                    generated_foreign_functions
                        .push(format!("  {}", self.foreign_function(new_fn, "int", args)));
                }
            }
        }

        for entry in pkg.manifest.entry_points.values() {
            let mut args = vec!["context".to_string()];

            for out in &entry.outputs {
                let t = self.get_ctype(&out.r#type);

                args.push(format!("ptr {t}"));
            }

            for input in &entry.inputs {
                let t = self.get_ctype(&input.r#type);
                args.push(t);
            }

            let args = args.iter().map(|x| x.as_str()).collect();
            generated_foreign_functions.push(format!(
                "  {}",
                self.foreign_function(&entry.cfun, "int", args)
            ));
        }

        let generated_foreign_functions = generated_foreign_functions.join("\n");

        writeln!(
            config.output_file,
            include_str!("templates/ocaml/bindings.ml"),
            generated_foreign_functions = generated_foreign_functions
        )?;

        writeln!(self.mli_file, include_str!("templates/ocaml/bindings.mli"))?;

        let (extra_param, extra_line, extra_mli) = match pkg.manifest.backend {
            Backend::Multicore => (
                "?(num_threads = 0)",
                "    Bindings.futhark_context_config_set_num_threads config num_threads;",
                "?num_threads:int ->",
            ),

            Backend::CUDA | Backend::OpenCL => (
                "?device",
                "    Option.iter (Bindings.futhark_context_config_set_device config) device;",
                "?device:string ->",
            ),
            _ => ("", "", ""),
        };

        writeln!(
            config.output_file,
            include_str!("templates/ocaml/context.ml"),
            extra_param = extra_param,
            extra_line = extra_line
        )?;
        writeln!(
            self.mli_file,
            include_str!("templates/ocaml/context.mli"),
            extra_mli = extra_mli
        )?;

        Ok(())
    }

    fn array_type(
        &mut self,
        _pkg: &Package,
        config: &mut Config,
        name: &str,
        ty: &manifest::ArrayType,
    ) -> Result<(), Error> {
        let rank = ty.rank;
        let elemtype = ty.elemtype.to_str().to_string();
        let ocaml_name = self.typemap.get(name).unwrap();
        let module_name = first_uppercase(ocaml_name);
        let mut dim_args = Vec::new();
        for i in 0..rank {
            dim_args.push(format!("(Int64.of_int dims.({i}))"));
        }

        let (ocaml_elemtype, ba_elemtype) = self.get_ba_type(&elemtype);
        let ocaml_ctype = self.get_ctype(&elemtype);

        writeln!(
            config.output_file,
            include_str!("templates/ocaml/array.ml"),
            module_name = module_name,
            elemtype = elemtype,
            rank = rank,
            dim_args = dim_args.join(" "),
            ocaml_elemtype = ocaml_elemtype,
            ba_elemtype = ba_elemtype,
            ba_kind = ba_kind(&ba_elemtype),
            ocaml_ctype = ocaml_ctype,
        )?;

        writeln!(
            self.mli_file,
            include_str!("templates/ocaml/array.mli"),
            module_name = module_name,
            ocaml_elemtype = ocaml_elemtype,
            ba_elemtype = ba_elemtype,
        )?;

        Ok(())
    }

    fn opaque_type(
        &mut self,
        _pkg: &Package,
        config: &mut Config,
        name: &str,
        ty: &manifest::OpaqueType,
    ) -> Result<(), Error> {
        let futhark_name = convert_struct_name(&ty.ctype);
        let mut ocaml_name = futhark_name
            .strip_prefix("futhark_opaque_")
            .unwrap()
            .to_string();
        if ocaml_name.chars().next().unwrap().is_numeric() || name.contains(' ') {
            ocaml_name = format!("type_{ocaml_name}");
        }
        let module_name = first_uppercase(&ocaml_name);
        self.typemap
            .insert(ocaml_name.clone(), format!("{module_name}.t"));

        let free_fn = &ty.ops.free;

        writeln!(config.output_file, "module {module_name} = struct")?;
        writeln!(self.mli_file, "module {module_name} : sig")?;

        writeln!(
            config.output_file,
            include_str!("templates/ocaml/opaque.ml"),
            free_fn = free_fn,
            name = ocaml_name,
        )?;
        writeln!(self.mli_file, include_str!("templates/ocaml/opaque.mli"),)?;

        let record = match &ty.record {
            Some(r) => r,
            None => {
                writeln!(config.output_file, "end")?;
                writeln!(self.mli_file, "end")?;
                return Ok(());
            }
        };

        let mut new_params = Vec::new();
        let mut new_call_args = Vec::new();
        let mut new_arg_types = Vec::new();
        for f in record.fields.iter() {
            let t = self.get_type(&f.r#type);

            new_params.push(format!("field{}", f.name));

            if type_is_array(&t) {
                new_call_args.push(format!("field{}.ptr", f.name));
                new_arg_types.push(format!("{}.t", first_uppercase(&t)));
            } else if type_is_opaque(&t) {
                new_call_args.push(format!("field{}.opaque_ptr", f.name));
                new_arg_types.push(t.to_string());
            } else {
                new_call_args.push(format!("field{}", f.name));
                new_arg_types.push(t.to_string());
            }
        }

        writeln!(
            config.output_file,
            include_str!("templates/ocaml/record.ml"),
            new_params = new_params.join(" "),
            new_fn = record.new,
            new_call_args = new_call_args.join(" "),
        )?;

        writeln!(
            self.mli_file,
            include_str!("templates/ocaml/record.mli"),
            new_arg_types = new_arg_types.join(" -> ")
        )?;

        for f in record.fields.iter() {
            let t = self.get_type(&f.r#type);
            let name = &f.name;
            let project = &f.project;

            let (out, out_type) = if type_is_opaque(&t) {
                let call = t.replace(".t", ".of_ptr");
                (format!("{call} t.opaque_ctx !@out"), t.to_string())
            } else if type_is_array(&t) {
                let array = first_uppercase(&t);
                (
                    format!("{array}.of_ptr t.opaque_ctx !@out"),
                    format!("{}.t", first_uppercase(&t)),
                )
            } else {
                ("!@out".to_string(), t.to_string())
            };

            let alloc_type = if type_is_array(&t) {
                format!("Bindings.{t}")
            } else if type_is_opaque(&t) {
                t
            } else {
                self.get_ctype(&f.r#type)
            };

            writeln!(
                config.output_file,
                include_str!("templates/ocaml/record_project.ml"),
                name = name,
                s = alloc_type,
                project = project,
                out = out
            )?;
            writeln!(
                self.mli_file,
                include_str!("templates/ocaml/record_project.mli"),
                name = name,
                out_type = out_type
            )?;
        }

        writeln!(config.output_file, "end\n")?;
        writeln!(self.mli_file, "end\n")?;

        Ok(())
    }

    fn entry(
        &mut self,
        _pkg: &Package,
        config: &mut Config,
        name: &str,
        entry: &manifest::Entry,
    ) -> Result<(), Error> {
        let mut arg_types = Vec::new();
        let mut return_type = Vec::new();
        let mut entry_params = Vec::new();
        let mut call_args = Vec::new();
        let mut out_return = Vec::new();
        let mut out_decl = Vec::new();

        for (i, out) in entry.outputs.iter().enumerate() {
            let t = self.get_type(&out.r#type);
            let ct = self.get_ctype(&out.r#type);

            let mut ocaml_elemtype = t.clone();

            // Transform into `Module.t`
            if ocaml_elemtype.contains("array_") {
                ocaml_elemtype = first_uppercase(&ocaml_elemtype) + ".t"
            }

            return_type.push(ocaml_elemtype);

            let i = if entry.outputs.len() == 1 {
                String::new()
            } else {
                i.to_string()
            };

            if type_is_array(&t) || type_is_opaque(&t) {
                out_decl.push(format!("  let out{i}_ptr = allocate (ptr void) null in"));
            } else {
                out_decl.push(format!("  let out{i}_ptr = allocate_n {ct} ~count:1 in"));
            }

            call_args.push(format!("out{i}_ptr"));

            if type_is_array(&t) {
                let m = first_uppercase(&t);
                out_return.push(format!("({m}.of_ptr ctx !@out{i}_ptr)"));
            } else if type_is_opaque(&t) {
                let m = first_uppercase(&t);
                let m = m.strip_suffix(".t").unwrap_or(&m);
                out_return.push(format!("({m}.of_ptr ctx !@out{i}_ptr)"));
            } else {
                out_return.push(format!("!@out{i}_ptr"));
            }
        }

        for (i, input) in entry.inputs.iter().enumerate() {
            entry_params.push(format!("input{i}"));

            let mut ocaml_elemtype = self.get_type(&input.r#type);

            // Transform into `Module.t`
            if type_is_array(&ocaml_elemtype) {
                ocaml_elemtype = first_uppercase(&ocaml_elemtype) + ".t"
            }

            arg_types.push(ocaml_elemtype);

            let t = self.get_type(&input.r#type);
            if type_is_array(&t) {
                call_args.push(format!("input{i}.ptr"));
            } else if type_is_opaque(&t) {
                call_args.push(format!("input{i}.opaque_ptr"));
            } else {
                call_args.push(format!("input{i}"));
            }
        }

        writeln!(
            config.output_file,
            include_str!("templates/ocaml/entry.ml"),
            name = name,
            entry_params = entry_params.join(" "),
            out_decl = out_decl.join("\n"),
            call_args = call_args.join(" "),
            call_tmp = call_args.join(", "),
            out_return = out_return.join(", ")
        )?;

        let return_type = if return_type.is_empty() {
            "unit".to_string()
        } else {
            return_type.join(" * ")
        };
        writeln!(
            self.mli_file,
            include_str!("templates/ocaml/entry.mli"),
            name = name,
            arg_types = arg_types.join(" -> "),
            return_type = return_type,
        )?;

        Ok(())
    }
}
