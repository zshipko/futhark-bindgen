use crate::generate::{convert_struct_name, first_uppercase};
use crate::*;
use std::io::Write;

/// Rust codegen
pub struct Rust {
    typemap: BTreeMap<String, String>,
}

fn type_is_array(t: &str) -> bool {
    t.contains("ArrayF") || t.contains("ArrayI") || t.contains("ArrayU") || t.contains("ArrayB")
}

fn type_is_opaque(a: &str) -> bool {
    a.contains("futhark_opaque_")
}

// Rust `f16` codgen requires the `half` crate
const RUST_TYPE_MAP: &[(&str, &str)] = &[("f16", "half::f16")];

impl Default for Rust {
    fn default() -> Self {
        let typemap = RUST_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        Rust { typemap }
    }
}

struct ArrayInfo {
    futhark_type: String,
    rust_type: String,
    #[allow(unused)]
    elem: String,
}

impl Rust {
    fn get_type(typemap: &BTreeMap<String, String>, t: &str) -> String {
        let a = typemap.get(t);
        let x = match a {
            Some(t) => t.clone(),
            None => t.to_string(),
        };
        if x.is_empty() {
            panic!("Unsupported type: {t}");
        }
        x
    }
}

impl Generate for Rust {
    fn array_type(
        &mut self,
        _library: &Package,
        config: &mut Config,
        name: &str,
        a: &manifest::ArrayType,
    ) -> Result<(), Error> {
        let elemtype = a.elemtype.to_str();
        let rank = a.rank;

        let futhark_type = convert_struct_name(&a.ctype).to_string();
        let rust_type = format!("Array{}D{rank}", elemtype.to_ascii_uppercase(),);
        let info = ArrayInfo {
            futhark_type,
            rust_type,
            elem: elemtype.to_string(),
        };

        let mut dim_params = Vec::new();
        let mut new_dim_args = Vec::new();

        for i in 0..a.rank {
            let dim = format!("dims[{i}]");
            dim_params.push(dim);
            new_dim_args.push(format!("dim{i}: i64"));
        }

        writeln!(
            config.output_file,
            include_str!("templates/rust/array.rs"),
            futhark_type = info.futhark_type,
            rust_type = info.rust_type,
            rank = a.rank,
            elemtype = info.elem,
            new_fn = a.ops.new,
            free_fn = a.ops.free,
            values_fn = a.ops.values,
            shape_fn = a.ops.shape,
            dim_params = dim_params.join(", "),
            new_dim_args = new_dim_args.join(", ")
        )?;

        self.typemap
            .insert(name.to_string(), info.futhark_type.clone());
        self.typemap.insert(info.futhark_type, info.rust_type);
        Ok(())
    }

    fn opaque_type(
        &mut self,
        _library: &Package,
        config: &mut Config,
        name: &str,
        ty: &manifest::OpaqueType,
    ) -> Result<(), Error> {
        let futhark_type = convert_struct_name(&ty.ctype).to_string();
        let mut rust_type = first_uppercase(futhark_type.strip_prefix("futhark_opaque_").unwrap());
        if rust_type.chars().next().unwrap().is_numeric() || name.contains(' ') {
            rust_type = format!("Type{}", rust_type);
        }

        writeln!(
            config.output_file,
            include_str!("templates/rust/opaque.rs"),
            futhark_type = futhark_type,
            rust_type = rust_type,
            free_fn = ty.ops.free,
        )?;

        let record = match &ty.record {
            Some(r) => r,
            None => {
                self.typemap.insert(name.to_string(), futhark_type.clone());
                self.typemap.insert(futhark_type, rust_type);
                return Ok(());
            }
        };

        let mut new_call_args = vec![];
        let mut new_params = vec![];
        let mut new_extern_params = vec![];
        for field in record.fields.iter() {
            // Build new function
            let a = Self::get_type(&self.typemap, &field.r#type);
            let t = Self::get_type(&self.typemap, &a);

            let u = if t == field.r#type {
                t.to_string()
            } else {
                format!("&{t}")
            };

            if type_is_opaque(&a) {
                new_call_args.push(format!("field{}.data", field.name));
                new_extern_params.push(format!("field{}: *const {a}", field.name));
            } else if type_is_array(&t) {
                new_call_args.push(format!("field{}.ptr", field.name));
                new_extern_params.push(format!("field{}: *const {a}", field.name));
            } else {
                new_call_args.push(format!("field{}", field.name));
                new_extern_params.push(format!("field{}: {a}", field.name));
            }

            new_params.push(format!("field{}: {u}", field.name));

            // Implement get function

            // If the output type is an array or opaque type then we need to wrap the return value
            let (output, futhark_field_type) = if type_is_opaque(&a) || type_is_array(&t) {
                (
                    format!("Ok({t}::from_ptr(self.ctx, out))"),
                    format!("*mut {a}"),
                )
            } else {
                ("Ok(out)".to_string(), a)
            };

            writeln!(
                config.output_file,
                include_str!("templates/rust/record_project.rs"),
                project_fn = field.project,
                rust_type = rust_type,
                futhark_type = futhark_type,
                field_name = field.name,
                futhark_field_type = futhark_field_type,
                rust_field_type = t,
                output = output
            )?;
        }

        writeln!(
            config.output_file,
            include_str!("templates/rust/record.rs"),
            rust_type = rust_type,
            futhark_type = futhark_type,
            new_fn = record.new,
            new_params = new_params.join(", "),
            new_call_args = new_call_args.join(", "),
            new_extern_params = new_extern_params.join(", "),
        )?;

        self.typemap.insert(name.to_string(), futhark_type.clone());
        self.typemap.insert(futhark_type, rust_type);

        Ok(())
    }

    fn entry(
        &mut self,
        _library: &Package,
        config: &mut Config,
        name: &str,
        entry: &manifest::Entry,
    ) -> Result<(), Error> {
        let mut call_args = Vec::new();
        let mut entry_params = Vec::new();
        let mut return_type = Vec::new();
        let mut out_decl = Vec::new();
        let mut futhark_entry_params = Vec::new();
        let mut entry_return = Vec::new();

        // Output arguments
        for (i, arg) in entry.outputs.iter().enumerate() {
            let a = Self::get_type(&self.typemap, &arg.r#type);

            let name = format!("out{i}");

            let t = Self::get_type(&self.typemap, &a);

            if type_is_array(&t) || type_is_opaque(&a) {
                futhark_entry_params.push(format!("{name}: *mut *mut {a}"));
            } else {
                futhark_entry_params.push(format!("{name}: *mut {a}"));
            }

            if type_is_array(&t) || type_is_opaque(&a) {
                entry_return.push(format!("{t}::from_ptr(self, {name}.assume_init())",));
            } else {
                entry_return.push(format!("{name}.assume_init()"));
            }

            out_decl.push(format!("let mut {name} = std::mem::MaybeUninit::zeroed();"));
            call_args.push(format!("{name}.as_mut_ptr()"));
            return_type.push(t);
        }

        // Input arguments
        for (i, arg) in entry.inputs.iter().enumerate() {
            let a = Self::get_type(&self.typemap, &arg.r#type);
            let name = format!("input{i}");

            let t = Self::get_type(&self.typemap, &a);

            if type_is_array(&t) {
                futhark_entry_params.push(format!("{name}: *const {a}"));

                entry_params.push(format!("{name}: &{t}"));
                call_args.push(format!("{name}.ptr as *mut _"));
            } else if type_is_opaque(&a) {
                futhark_entry_params.push(format!("{name}: *const {a}"));

                entry_params.push(format!("{name}: &{t}"));
                call_args.push(format!("{name}.data as *mut _"));
            } else {
                futhark_entry_params.push(format!("{name}: {a}"));
                entry_params.push(format!("{name}: {t}"));
                call_args.push(name);
            }
        }

        let (entry_return_type, entry_return) = match entry.outputs.len() {
            0 => ("()".to_string(), "()".to_string()),
            1 => (return_type.join(", "), entry_return.join(", ")),
            _ => (
                format!("({})", return_type.join(", ")),
                format!("({})", entry_return.join(", ")),
            ),
        };

        writeln!(
            config.output_file,
            include_str!("templates/rust/entry.rs"),
            entry_fn = entry.cfun,
            entry_name = name,
            entry_params = entry_params.join(", "),
            entry_return_type = entry_return_type,
            out_decl = out_decl.join(";\n"),
            call_args = call_args.join(", "),
            entry_return = entry_return,
            futhark_entry_params = futhark_entry_params.join(", "),
        )?;

        Ok(())
    }

    fn bindings(&mut self, library: &Package, config: &mut Config) -> Result<(), Error> {
        writeln!(config.output_file, "// Generated by futhark-bindgen\n")?;
        let backend_extern_functions = match &library.manifest.backend {
            Backend::Multicore => {
                "fn futhark_context_config_set_num_threads(_: *mut futhark_context_config, _: std::os::raw::c_int);"
            }
            Backend::OpenCL | Backend::CUDA => {
                "fn futhark_context_config_set_device(_: *mut futhark_context_config, _: *const std::os::raw::c_char);"
            }
            _ => "",
        };

        let backend_options = match library.manifest.backend {
            Backend::Multicore => {
                "pub fn threads(mut self, n: u32) -> Options { self.num_threads = n as u32; self }"
            }
            Backend::CUDA | Backend::OpenCL => {
                "pub fn device(mut self, s: impl AsRef<str>) -> Options { self.device = Some(std::ffi::CString::new(s.as_ref()).expect(\"Invalid device\")); self }"
            }
            _ => "",
        };

        let configure_num_threads = if library.manifest.backend == Backend::Multicore {
            "futhark_context_config_set_num_threads(config, options.num_threads as std::os::raw::c_int);"
        } else {
            "let _ = &options.num_threads;"
        };

        let configure_set_device = if matches!(
            library.manifest.backend,
            Backend::CUDA | Backend::OpenCL
        ) {
            "if let Some(d) = &options.device { futhark_context_config_set_device(config, d.as_ptr()); }"
        } else {
            "let _ = &options.device;"
        };

        writeln!(
            config.output_file,
            include_str!("templates/rust/context.rs"),
            backend_options = backend_options,
            configure_num_threads = configure_num_threads,
            configure_set_device = configure_set_device,
            backend_extern_functions = backend_extern_functions,
        )?;

        Ok(())
    }

    fn format(&mut self, path: &std::path::Path) -> Result<(), Error> {
        let _ = std::process::Command::new("rustfmt").arg(&path).status();
        Ok(())
    }
}
