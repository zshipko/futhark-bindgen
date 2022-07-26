use crate::generate::first_uppercase;
use crate::*;
use std::io::Write;

pub struct Rust {
    typemap: BTreeMap<String, String>,
    scope: codegen::Scope,
}

fn type_is_array(t: &str) -> bool {
    t.contains("ArrayF") || t.contains("ArrayI") || t.contains("ArrayU") || t.contains("ArrayB")
}

fn type_is_opaque(a: &str) -> bool {
    a.contains("futhark_opaque_")
}

const RUST_TYPE_MAP: &[(&'static str, &'static str)] = &[];

impl Default for Rust {
    fn default() -> Self {
        let typemap = RUST_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        Rust {
            typemap,
            scope: codegen::Scope::new(),
        }
    }
}

#[derive(Default)]
pub struct ExternFn {
    name: String,
    args: Vec<(String, String)>,
    ret: String,
}

impl ExternFn {
    pub fn new(name: impl Into<String>) -> ExternFn {
        ExternFn {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn arg(mut self, name: impl Into<String>, t: impl Into<String>) -> Self {
        self.args.push((name.into(), t.into()));
        self
    }

    pub fn ret(mut self, r: impl Into<String>) -> Self {
        self.ret = r.into();
        self
    }

    pub fn gen(self, r: &mut Rust) {
        let mut s = format!("extern \"C\" {} fn {}(", '{', self.name);
        let mut args = Vec::new();
        for arg in &self.args {
            args.push(format!("{}: {}", arg.0, arg.1));
        }
        s += &args.join(", ");
        s += ")";
        if !self.ret.is_empty() {
            s += "-> ";
            s += &self.ret;
        }
        s += ";\n}\n";
        r.scope.raw("#[allow(unused)]");
        r.scope.raw(&s);
    }
}

struct ArrayInfo {
    original_name: String,
    ptr: String,
    rust_name: String,
    #[allow(unused)]
    elem: String,
    elem_ptr: String,
}

impl Rust {
    fn generate_array_type(&mut self, a: &manifest::ArrayType) -> Result<ArrayInfo, Error> {
        let elemtype = a.elemtype.to_str();
        let rank = a.rank;

        let original_name = format!("futhark_{elemtype}_{rank}d");
        let rust_name = format!("Array{}D{rank}", elemtype.to_ascii_uppercase(),);
        let ptr = format!("*mut {original_name}");
        let info = ArrayInfo {
            original_name,
            ptr,
            rust_name,
            elem: elemtype.to_string(),
            elem_ptr: format!("*mut {}", elemtype),
        };

        // Original array type (from C)
        self.scope
            .new_struct(&info.original_name)
            .allow("non_camel_case_types")
            .repr("C")
            .field("_private", "[u8; 0]");

        // Rust wrapper
        self.scope
            .new_struct(&info.rust_name)
            .field("ptr", &info.ptr)
            .field("pub shape", &format!("[i64; {}]", a.rank))
            .field("ctx", "*mut futhark_context")
            .field("_t", "std::marker::PhantomData<&'a ()>")
            .generic("'a")
            .vis("pub")
            .doc(&format!("A wrapper around {}", info.original_name));

        // Build Array::new
        let new_fn = format!("futhark_new_{elemtype}_{rank}d");
        let array_impl = self
            .scope
            .new_impl(&info.rust_name)
            .generic("'a")
            .target_generic("'a");
        let mut dim_params = Vec::new();
        for i in 0..a.rank {
            let dim = format!("dims[{i}]");
            dim_params.push(dim);
        }
        let _array_new = array_impl
            .new_fn("new")
            .vis("pub")
            .doc("Create a new, empty array")
            .arg("ctx", "&'a Context")
            .arg("dims", &format!("[i64; {}]", a.rank))
            .ret("Result<Self, Error>")
            .line(&format!("let data = vec![0 as {elemtype}; dims.iter().fold(1, |a, b| a * *b as usize)];"))
            .line("let ptr = unsafe {")
            .line(&format!(
                "    {}(ctx.context, data.as_ptr(), {})",
                &new_fn,
                dim_params.join(", ")
            ))
            .line("};")
            .line("if ptr.is_null() { return Err(Error::NullPtr); }")
            .line("Ok(Self { ptr: ptr as *mut _, shape: dims, ctx: ctx.context, _t: std::marker::PhantomData })");

        // Array::from_slice
        let _array_from_slice = array_impl
            .new_fn("from_slice")
            .vis("pub")
            .doc("Create a new array from an existing slice")
            .arg("ctx", "&'a Context")
            .arg("dims", &format!("[i64; {rank}]"))
            .arg("data", &format!("&[{elemtype}]"))
            .ret("Result<Self, Error>")
            .line("if data.len() as i64 != dims.iter().fold(1, |a, b| a * b) { return Err(Error::InvalidShape); }")
            .line("let ptr = unsafe {")
            .line(&format!(
                "    {}(ctx.context, data.as_ptr(), {})",
                &new_fn,
                dim_params.join(", ")
            ))
            .line("};")
            .line("if ptr.is_null() { return Err(Error::NullPtr); }")
            .line("Ok(Self { ptr: ptr as *mut _, shape: dims, ctx: ctx.context, _t: std::marker::PhantomData })");

        // Array::values
        let _array_values = array_impl
            .new_fn("values")
            .vis("pub")
            .doc("Load array data into a mutable slice")
            .arg_ref_self()
            .arg("data", &format!("&mut [{elemtype}]"))
            .ret("Result<(), Error>")
            .line("if data.len() as i64 != self.shape.iter().fold(1, |a, b| a * b) { return Err(Error::InvalidShape); }")
            .line("let rc = unsafe {")
            .line(&format!("    futhark_values_{elemtype}_{rank}d(self.ctx, self.ptr as *mut _, data.as_mut_ptr())"))
            .line("};")
            .line("if rc != 0 { return Err(Error::Code(rc)) }")
            .line("Ok(())");

        // Array::from_raw
        let _array_from_raw = array_impl
            .new_fn("from_raw")
            .allow("unused")
            .arg("ctx", "*mut futhark_context")
            .arg("data", &info.ptr)
            .ret("Self")
            .line(&format!(
                "let len_ptr = unsafe {} futhark_shape_{elemtype}_{rank}d(ctx, data) {};",
                '{', '}'
            ))
            .line(&format!("let mut shape = [0i64; {rank}];"))
            .line("unsafe {")
            .line(&format!(
                "for i in 0 .. {rank} {} shape[i] = *len_ptr.add(i); {}",
                '{', '}'
            ))
            .line("}")
            .line("Self { ctx, shape, ptr: data, _t: std::marker::PhantomData }");

        // Implement Drop for Array
        let _array_drop = self
            .scope
            .new_impl(&info.rust_name)
            .generic("'a")
            .target_generic("'a")
            .impl_trait("Drop")
            .new_fn("drop")
            .arg_mut_self()
            .line("unsafe {")
            .line(&format!(
                "    futhark_free_{elemtype}_{rank}d(self.ctx, self.ptr as *mut _);",
            ))
            .line("}");

        // Extern definitions

        ExternFn::new(&format!("futhark_shape_{elemtype}_{rank}d"))
            .arg("_", "*mut futhark_context")
            .arg("_", &info.ptr)
            .ret("*const i64")
            .gen(self);

        let mut new = ExternFn::new(new_fn)
            .arg("_", "*mut futhark_context")
            .arg("_", &info.elem_ptr.replace("*mut", "*const"));

        for i in 0..a.rank {
            new = new.arg(&format!("dim{i}"), "i64");
        }

        new.ret(&info.ptr).gen(self);

        let mut new_raw = ExternFn::new(format!("futhark_new_raw_{elemtype}_{rank}d",))
            .arg("_", "*mut futhark_context")
            .arg("_", "*const u8")
            .arg("offset", "i64");

        for i in 0..a.rank {
            new_raw = new_raw.arg(&format!("dim{i}"), "i64");
        }

        new_raw.ret(&info.ptr).gen(self);

        // free
        let _free = ExternFn::new(format!("futhark_free_{elemtype}_{rank}d"))
            .arg("_", "*mut futhark_context")
            .arg("_", &info.ptr)
            .ret("std::os::raw::c_int")
            .gen(self);

        // values
        let _values = ExternFn::new(format!("futhark_values_{elemtype}_{rank}d",))
            .arg("_", "*mut futhark_context")
            .arg("_", &info.ptr)
            .arg("_", &info.elem_ptr)
            .ret("std::os::raw::c_int")
            .gen(self);

        let _values_raw = ExternFn::new(format!("futhark_values_raw_{elemtype}_{rank}d",))
            .arg("_", "*mut futhark_context")
            .arg("_", &info.ptr)
            .ret("*mut u8")
            .gen(self);
        Ok(info)
    }

    fn get_type(typemap: &BTreeMap<String, String>, t: &str) -> String {
        let a = typemap.get(t);
        match a {
            Some(t) => t.clone(),
            None => t.to_string(),
        }
    }

    fn generate_opaque_type(
        &mut self,
        name: &str,
        ty: &manifest::OpaqueType,
    ) -> Result<String, Error> {
        let original_name = format!("futhark_opaque_{name}");
        let rust_name = format!("{}", first_uppercase(name));

        // C type

        self.scope
            .new_struct(&original_name)
            .allow("non_camel_case_types")
            .repr("C")
            .field("_private", "[u8; 0]");

        // Extern definitions

        let mut new = ExternFn::new(format!("futhark_new_opaque_{name}"))
            .arg("_", "*mut futhark_context")
            .arg("out", format!("*mut *mut {original_name}"))
            .ret("std::os::raw::c_int");

        for field in ty.record.fields.iter() {
            let t = Self::get_type(&self.typemap, &field.r#type);
            new = new.arg(format!("field{}", field.name), t);
        }
        new.gen(self);

        let _free = ExternFn::new(format!("futhark_free_opaque_{name}"))
            .arg("_", "*mut futhark_context")
            .arg("_", format!("*mut {original_name}"))
            .ret("std::os::raw::c_int")
            .gen(self);

        for field in ty.record.fields.iter() {
            let t = Self::get_type(&self.typemap, &field.r#type);

            let _project = ExternFn::new(format!("futhark_project_opaque_{name}_{}", field.name))
                .arg("_", "*mut futhark_context")
                .arg("_", format!("*mut {}", t))
                .arg("_", format!("*const {original_name}"))
                .ret("std::os::raw::c_int")
                .gen(self);
        }

        // Rust struct definition

        self.scope
            .new_struct(&rust_name)
            .vis("pub")
            .generic("'a")
            .field("data", &format!("*mut {original_name}"))
            .field("ctx", "*mut futhark_context")
            .field("_t", "std::marker::PhantomData<&'a ()>");

        // Implement drop
        self.scope
            .new_impl(&rust_name)
            .generic("'a")
            .target_generic("'a")
            .impl_trait("Drop")
            .new_fn("drop")
            .arg_mut_self()
            .line("unsafe {")
            .line(&format!(
                "    futhark_free_opaque_{name}(self.ctx, self.data);",
            ))
            .line("}");

        // Open impl block
        let opaque = self
            .scope
            .new_impl(&rust_name)
            .generic("'a")
            .target_generic("'a");

        // Opaque::from_raw
        let _opaque_from_raw = opaque
            .new_fn("from_raw")
            .allow("unused")
            .arg("ctx", "*mut futhark_context")
            .arg("data", &format!("*mut {original_name}"))
            .ret("Self")
            .line("Self { ctx, data, _t: std::marker::PhantomData }");

        // Opaque::new
        let new_fn = opaque
            .new_fn("new")
            .vis("pub")
            .arg("ctx", "&'a Context")
            .ret("Result<Self, Error>");

        let mut args = vec![];
        for field in ty.record.fields.iter() {
            let a = Self::get_type(&self.typemap, &field.r#type);
            let t = Self::get_type(&self.typemap, &a);

            let u = if &t == &field.r#type {
                format!("{t}")
            } else {
                format!("&{t}")
            };

            if type_is_opaque(&a) {
                args.push(format!("field{}.data", field.name));
            } else if type_is_array(&t) {
                args.push(format!("field{}.ptr", field.name));
            } else {
                args.push(format!("field{}", field.name));
            }

            new_fn.arg(&format!("field{}", field.name), &u);
        }

        new_fn
            .line("let mut out: *mut _ = std::ptr::null_mut();")
            .line(format!(
                "let rc = unsafe {} futhark_new_opaque_{name}(ctx.context, &mut out, {}) {};",
                '{',
                args.join(", "),
                '}'
            ))
            .line("if rc != 0 { return Err(Error::Code(rc)); }")
            .line("Ok(Self {data: out, ctx: ctx.context, _t: std::marker::PhantomData})");

        // Implement get functions
        for field in ty.record.fields.iter() {
            let a = Self::get_type(&self.typemap, &field.r#type);
            let t = Self::get_type(&self.typemap, &a);

            let func = opaque
                .new_fn(&format!("get_{}", field.name))
                .vis("pub")
                .arg_ref_self()
                .ret(&format!("Result<{t}, Error>"));

            func
                .line("let mut out = std::mem::MaybeUninit::zeroed();")
                .line(&format!("let rc = unsafe {} futhark_project_opaque_{name}_{}(self.ctx, out.as_mut_ptr(), self.data) {};", '{', field.name, '}'))
                .line("if rc != 0 { return Err(Error::Code(rc)); }")
                .line("let out = unsafe { out.assume_init() };");

            // If the output type is an array or opaque type then we need to wrap the return value
            if type_is_opaque(&a) {
                func.line(&format!("Ok({t}::from_raw(self.ctx, out))"));
            } else if type_is_array(&t) {
                func.line(&format!("Ok({t}::from_raw(self.ctx, out))"));
            } else {
                func.line("Ok(out)");
            }
        }

        Ok(rust_name)
    }

    fn generate_entry_function(
        &mut self,
        name: &str,
        entry: &manifest::Entry,
    ) -> Result<(), Error> {
        let mut c = ExternFn::new(&entry.cfun)
            .arg("_", "*mut futhark_context")
            .ret("std::os::raw::c_int");

        let func = self
            .scope
            .new_impl("Context")
            .new_fn(name)
            .doc(&format!("Entry point: {name}"))
            .vis("pub")
            .ret("Result<(), Error>")
            .arg_ref_self();

        let mut call_args = Vec::new();

        // Output arguments
        for (i, arg) in entry.outputs.iter().enumerate() {
            let a = Self::get_type(&self.typemap, &arg.r#type);

            let name = format!("out{i}");
            c = c.arg(&name, format!("*mut {a}"));

            let t = Self::get_type(&self.typemap, &a);
            func.arg(&name, &format!("&mut {t}"));
            if type_is_array(&t) {
                call_args.push(format!("{name}.ptr as *mut _"))
            } else if type_is_opaque(&a) {
                call_args.push(format!("{name}.data as *mut _"));
            } else {
                call_args.push(format!("{name} as *mut _"));
            }
        }

        // Input arguments
        for (i, arg) in entry.inputs.iter().enumerate() {
            let a = Self::get_type(&self.typemap, &arg.r#type);
            let name = format!("input{i}");
            c = c.arg(&name, a.replace("*mut", "*const"));

            let t = Self::get_type(&self.typemap, &a);

            if type_is_array(&t) {
                func.arg(&name, &format!("&{t}"));
                call_args.push(format!("{name}.ptr as *mut _"));
            } else if type_is_opaque(&a) {
                func.arg(&name, &format!("&{t}"));
                call_args.push(format!("{name}.data as *mut _"));
            } else {
                func.arg(&name, t);
                call_args.push(name);
            }
        }

        func.line("let rc = unsafe {")
            .line(&format!(
                "{}(self.context, {})",
                entry.cfun,
                call_args.join(", ")
            ))
            .line("};")
            .line("if rc != 0 { return Err(Error::Code(rc)) }")
            .line("Ok(())");

        c.gen(self);
        Ok(())
    }
}

impl Generate for Rust {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error> {
        writeln!(config.output_file, "// Generated by futhark-bindgen\n")?;
        self.scope
            .new_struct("futhark_context_config")
            .allow("non_camel_case_types")
            .repr("C")
            .field("_private", "[u8; 0]");

        ExternFn::new("futhark_context_config_new")
            .ret("*mut futhark_context_config")
            .gen(self);

        ExternFn::new("futhark_context_config_free")
            .arg("_", "*mut futhark_context_config")
            .gen(self);

        ExternFn::new("futhark_context_config_set_debugging")
            .arg("_", "*mut futhark_context_config")
            .arg("_", "std::os::raw::c_int")
            .gen(self);

        ExternFn::new("futhark_context_config_set_profiling")
            .arg("_", "*mut futhark_context_config")
            .arg("_", "std::os::raw::c_int")
            .gen(self);

        ExternFn::new("futhark_context_config_set_logging")
            .arg("_", "*mut futhark_context_config")
            .arg("_", "std::os::raw::c_int")
            .gen(self);

        ExternFn::new("futhark_context_config_set_cache_file")
            .arg("_", "*mut futhark_context_config")
            .arg("_", "*const std::os::raw::c_char")
            .gen(self);

        self.scope
            .new_struct("futhark_context")
            .allow("non_camel_case_types")
            .repr("C")
            .field("_private", "[u8; 0]");

        ExternFn::new("futhark_context_new")
            .arg("config", "*mut futhark_context_config")
            .ret("*mut futhark_context")
            .gen(self);
        ExternFn::new("futhark_context_free")
            .arg("_", "*mut futhark_context")
            .gen(self);

        ExternFn::new("futhark_context_sync")
            .arg("_", "*mut futhark_context")
            .ret("std::os::raw::c_int")
            .gen(self);

        ExternFn::new("futhark_context_clear_caches")
            .arg("_", "*mut futhark_context")
            .ret("std::os::raw::c_int")
            .gen(self);

        ExternFn::new("futhark_context_pause_profiling")
            .arg("_", "*mut futhark_context")
            .gen(self);

        ExternFn::new("futhark_context_unpause_profiling")
            .arg("_", "*mut futhark_context")
            .gen(self);

        ExternFn::new("futhark_context_get_error")
            .arg("_", "*mut futhark_context")
            .ret("*mut std::os::raw::c_char")
            .gen(self);

        ExternFn::new("futhark_context_report")
            .arg("_", "*mut futhark_context")
            .ret("*mut std::os::raw::c_char")
            .gen(self);

        ExternFn::new("free")
            .arg("_", "*mut std::ffi::c_void")
            .gen(self);

        let error = self.scope.new_enum("Error").vis("pub").derive("Debug");
        error.new_variant("Code").tuple("std::os::raw::c_int");
        error.new_variant("NullPtr");
        error.new_variant("InvalidShape");

        self.scope
            .new_struct("Options")
            .vis("pub")
            .derive("Debug")
            .derive("Default")
            .derive("Clone")
            .field("debug", "bool")
            .field("profile", "bool")
            .field("logging", "bool")
            .field("cache_file", "Option<std::ffi::CString>");

        // Options
        let opts = self.scope.new_impl("Options");
        opts.new_fn("new")
            .vis("pub")
            .ret("Options")
            .line("Options::default()");
        opts.new_fn("debug")
            .vis("pub")
            .ret("Options")
            .arg_self()
            .line("let mut x = self; x.debug = true; x");
        opts.new_fn("profile")
            .vis("pub")
            .ret("Options")
            .arg_self()
            .line("let mut x = self; x.profile = true; x");
        opts.new_fn("log")
            .vis("pub")
            .ret("Options")
            .arg_self()
            .line("let mut x = self; x.logging = true; x");
        opts.new_fn("cache_file").vis("pub").ret("Options").arg_self().arg("a", "impl AsRef<str>").line(
            "let mut x = self; x.cache_file = Some(std::ffi::CString::new(a.as_ref()).expect(\"Invalid cache file\")); x",
        );

        // Context
        self.scope
            .new_struct("Context")
            .doc("Wrapper around futhark_context")
            .field("config", "*mut futhark_context_config")
            .field("context", "*mut futhark_context")
            .field("_cache_file", "Option<std::ffi::CString>")
            .vis("pub");

        let ctx = self.scope.new_impl("Context");
        let _ctx_new = ctx
            .new_fn("new")
            .vis("pub")
            .doc("Create a new context")
            .ret("Result<Self, Error>")
            .line("let config = unsafe { futhark_context_config_new () };")
            .line("if config.is_null() { return Err(Error::NullPtr) }")
            .line("let context = unsafe { futhark_context_new(config) };")
            .line("if context.is_null() { return Err(Error::NullPtr) }")
            .line("Ok(Context { config, context, _cache_file: None })");

        let _ctx_new_with_options = ctx
            .new_fn("new_with_options")
            .vis("pub")
            .doc("Create a new context with options")
            .ret("Result<Self, Error>")
            .arg("options", "Options")
            .line("let config = unsafe { futhark_context_config_new () };")
            .line("if config.is_null() { return Err(Error::NullPtr) }")
            .line("unsafe { futhark_context_config_set_debugging(config, options.debug as std::os::raw::c_int) }")
            .line("unsafe { futhark_context_config_set_profiling(config, options.profile as std::os::raw::c_int) }")
            .line("unsafe { futhark_context_config_set_logging(config, options.logging as std::os::raw::c_int) }")
            .line("if let Some(c) = &options.cache_file { unsafe { futhark_context_config_set_cache_file(config, c.as_ptr()); } }")
            .line("let context = unsafe { futhark_context_new(config) };")
            .line("if context.is_null() { return Err(Error::NullPtr) }")
            .line("Ok(Context { config, context, _cache_file: options.cache_file })");

        let _ctx_sync = ctx
            .new_fn("sync")
            .doc("Sync context")
            .vis("pub")
            .arg_ref_self()
            .ret("Result<(), Error>")
            .line("let rc = unsafe { futhark_context_sync(self.context) };")
            .line("if rc != 0 { return Err(Error::Code(rc)) }")
            .line("Ok(())");

        let _ctx_clear_caches = ctx
            .new_fn("clear_caches")
            .vis("pub")
            .doc("Clear internal caches")
            .ret("Result<(), Error>")
            .arg_ref_self()
            .line("let rc = unsafe { futhark_context_clear_caches(self.context) };")
            .line("if rc != 0 { return Err(Error::Code(rc)) }")
            .line("Ok(())");

        let _ctx_pause_profiling = ctx
            .new_fn("pause_profiling")
            .vis("pub")
            .doc("Pause profiling")
            .arg_ref_self()
            .line("unsafe { futhark_context_pause_profiling(self.context); }");

        let _ctx_unpause_profiling = ctx
            .new_fn("unpause_profiling")
            .vis("pub")
            .doc("Unpause profiling")
            .arg_ref_self()
            .line("unsafe { futhark_context_unpause_profiling(self.context); }");

        let _ctx_get_error = ctx
            .new_fn("get_error")
            .vis("pub")
            .doc("Get error message")
            .ret("Option<String>")
            .arg_ref_self()
            .line("let s = unsafe { futhark_context_get_error(self.context) };")
            .line("if s.is_null() { return None; }")
            .line("let r = unsafe { std::ffi::CStr::from_ptr(s).to_string_lossy().to_string() };")
            .line("unsafe { free(s as *mut _) };")
            .line("Some(r)");

        let _ctx_report = ctx
            .new_fn("report")
            .vis("pub")
            .doc("Get report with debug and profiling information")
            .ret("Option<String>")
            .arg_ref_self()
            .line("let s = unsafe { futhark_context_report(self.context) };")
            .line("if s.is_null() { return None; }")
            .line("let r = unsafe { std::ffi::CStr::from_ptr(s).to_string_lossy().to_string() };")
            .line("unsafe { free(s as *mut _) };")
            .line("Some(r)");

        let _ctx_drop = self
            .scope
            .new_impl("Context")
            .impl_trait("Drop")
            .new_fn("drop")
            .arg_mut_self()
            .line("unsafe {")
            .line("    futhark_context_free(self.context);")
            .line("    futhark_context_config_free(self.config);")
            .line("}");

        for (name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    let info = self.generate_array_type(a)?;
                    self.typemap.insert(name.clone(), info.ptr.clone());
                    self.typemap.insert(info.ptr, info.rust_name);
                }
                manifest::Type::Opaque(ty) => {
                    let rust_type = self.generate_opaque_type(name, ty)?;
                    self.typemap
                        .insert(name.clone(), format!("*mut futhark_opaque_{name}"));
                    self.typemap
                        .insert(format!("*mut futhark_opaque_{name}"), rust_type);
                }
            }
        }

        for (name, entry) in &library.manifest.entry_points {
            self.generate_entry_function(&name, entry)?;
        }

        write!(config.output_file, "{}", self.scope.to_string())?;
        let _ = std::process::Command::new("rustfmt")
            .arg(&config.output_path)
            .status();
        Ok(())
    }
}
