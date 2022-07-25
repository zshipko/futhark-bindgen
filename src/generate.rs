use crate::*;

use std::io::Write;

pub struct Config {
    pub output_path: std::path::PathBuf,
    pub output_file: std::fs::File,
}

impl Config {
    pub fn new(output: impl AsRef<std::path::Path>) -> Result<Config, Error> {
        Ok(Config {
            output_path: output.as_ref().to_path_buf(),
            output_file: std::fs::File::create(output)?,
        })
    }
}

pub trait Generate {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error>;
}

pub struct Python;

impl Generate for Python {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error> {
        match &library.manifest.backend {
            Backend::Python | Backend::PyOpenCL => {
                if &config.output_path != &library.py_file {
                    std::fs::copy(&library.py_file, &config.output_path)?;
                }
                Ok(())
            }
            _ => panic!("Python codegen must use a Python backend"),
        }
    }
}

pub struct Rust {
    typemap: BTreeMap<String, String>,
    scope: codegen::Scope,
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
        let mut s = format!("unsafe extern \"C\" fn {}(", self.name);
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
        s += ";\n";
        r.scope.raw(&s);
    }
}

struct ArrayInfo {
    original_name: String,
    ptr: String,
    rust_name: String,
    elem: String,
    elem_ptr: String,
}

impl Rust {
    fn generate_array_type(&mut self, a: &manifest::ArrayType) -> Result<ArrayInfo, Error> {
        let original_name = format!("futhark_{}_{}d", a.elemtype.to_str(), a.rank);
        let rust_name = format!("Array_{}_{}d", a.elemtype.to_str(), a.rank);
        let ptr = format!("*mut {original_name}");
        let info = ArrayInfo {
            original_name,
            ptr,
            rust_name,
            elem: a.elemtype.to_str().to_string(),
            elem_ptr: format!("*mut {}", a.elemtype.to_str()),
        };

        self.scope.new_struct(&info.original_name).repr("C");

        self.scope
            .new_struct(&info.rust_name)
            .field("ptr", &info.ptr)
            .field("pub shape", &format!("[i64; {}]", a.rank))
            .field("_t", "std::marker::PhantomData<&'a ()>")
            .vis("pub")
            .generic("'a");

        let new_fn = format!("futhark_new_{}_{}d", a.elemtype.to_str(), a.rank);
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
            .arg("ctx", "&'a Context")
            .arg("dims", &format!("[i64; {}]", a.rank))
            .ret("Result<Self, Error>")
            .line("let ptr = unsafe {")
            .line(&format!(
                "    {}(ctx.context, std::ptr::null(), {})",
                &new_fn,
                dim_params.join(", ")
            ))
            .line("};")
            .line("if ptr.is_null() { return Err(Error::NullPtr); }")
            .line("Ok(Self { ptr, shape: dims, _t: std::marker::PhantomData })");

        let _array_from_slice = array_impl
            .new_fn("from_slice")
            .vis("pub")
            .arg("ctx", "&'a Context")
            .arg("dims", &format!("[i64; {}]", a.rank))
            .arg("data", &format!("&[{}]", a.elemtype.to_str()))
            .ret("Result<Self, Error>")
            .line("if data.len() != dims.iter().fold(1, |a, b| a * b) { return Err(Error::InvalidShape); }")
            .line("let ptr = unsafe {")
            .line(&format!(
                "    {}(ctx.context, data.as_ptr(), {})",
                &new_fn,
                dim_params.join(", ")
            ))
            .line("};")
            .line("if ptr.is_null() { return Err(Error::NullPtr); }")
            .line("Ok(Self { ptr, _t: std::marker::PhantomData })");

        let _array_values = array_impl
            .new_fn("values")
            .vis("pub")
            .arg("ctx", "&'a Context")
            .arg("data", &format!("&mut [{}]", a.elemtype.to_str()))
            .ret("Result<(), Error>")
            .line("if data.len() != self.shape.fold(1, |a, b| a * b) { return Err(Error::InvalidShape); }")
            .line("let rc = unsafe {")
            .line(&format!("    futhark_values_{}_{}d(ctx.context, self.ptr, data.as_mut_ptr())", a.elemtype.to_str(), a.rank))
            .line("};")
            .line("if rc != 0 { return Err(Error::Code(rc)) }")
            .line("Ok(())");

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
                "    futhark_free_{}_{}d(self.ptr);",
                a.elemtype.to_str(),
                a.rank
            ))
            .line("}");

        // new

        let mut new = ExternFn::new(new_fn)
            .arg("_", "*mut futhark_context")
            .arg("_", &info.elem_ptr.replace("*mut", "*const"));

        for i in 0..a.rank {
            new = new.arg(&format!("dim{i}"), "i64");
        }

        new.ret(&info.ptr).gen(self);

        let mut new_raw = ExternFn::new(format!(
            "futhark_new_raw_{}_{}d",
            a.elemtype.to_str(),
            a.rank
        ))
        .arg("_", "*mut futhark_context")
        .arg("_", "*const u8")
        .arg("offset", "i64");

        for i in 0..a.rank {
            new_raw = new_raw.arg(&format!("dim{i}"), "i64");
        }

        new_raw.ret(&info.ptr).gen(self);

        // free
        let _free = ExternFn::new(format!("futhark_free_{}_{}d", a.elemtype.to_str(), a.rank))
            .arg("_", "*mut futhark_context")
            .arg("_", &info.ptr)
            .ret("std::os::raw::c_int")
            .gen(self);

        // values
        let _values = ExternFn::new(format!(
            "futhark_values_{}_{}d",
            a.elemtype.to_str(),
            a.rank
        ))
        .arg("_", "*mut futhark_context")
        .arg("_", &info.ptr)
        .arg("_", &info.elem_ptr)
        .ret("std::os::raw::c_int")
        .gen(self);

        let _values_raw = ExternFn::new(format!(
            "futhark_values_raw_{}_{}d",
            a.elemtype.to_str(),
            a.rank
        ))
        .arg("_", "*mut futhark_context")
        .arg("_", &info.ptr)
        .ret("*mut u8")
        .gen(self);
        Ok(info)
    }

    fn generate_entry_function(&mut self, entry: &manifest::Entry) -> Result<(), Error> {
        let mut c = ExternFn::new(&entry.cfun).arg("_", "*mut futhark_context");

        for (i, arg) in entry.outputs.iter().enumerate() {
            let t = self.typemap.get(&arg.r#type);
            let t = match t {
                Some(t) => t,
                None => &arg.r#type,
            };
            c = c.arg(format!("out{i}"), t)
        }

        for (i, arg) in entry.inputs.iter().enumerate() {
            let t = self.typemap.get(&arg.r#type);
            let t = match t {
                Some(t) => t,
                None => &arg.r#type,
            };
            c = c.arg(format!("input{i}"), t.replace("*mut", "*const"));
        }

        c.ret("std::os::raw::c_int").gen(self);
        Ok(())
    }
}

impl Generate for Rust {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error> {
        self.scope.new_struct("futhark_context_config").repr("C");

        ExternFn::new("futhark_context_config_new")
            .ret("*mut futhark_context_config")
            .gen(self);
        ExternFn::new("futhark_context_config_free")
            .arg("_", "*mut futhark_context_config")
            .gen(self);

        self.scope.new_struct("futhark_context").repr("C");

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

        let error = self.scope.new_enum("Error").vis("pub").derive("Debug");
        error.new_variant("Code").tuple("std::os::raw::c_int");
        error.new_variant("NullPtr");
        error.new_variant("InvalidShape");

        self.scope
            .new_struct("Context")
            .field("config", "*mut futhark_context_config")
            .field("context", "*mut futhark_context")
            .vis("pub");

        let ctx = self.scope.new_impl("Context");
        let _ctx_new = ctx
            .new_fn("new")
            .vis("pub")
            .ret("Result<Self, Error>")
            .line("let config = unsafe { futhark_context_config_new () };")
            .line("if config.is_null() { return Err(Error::NullPtr) }")
            .line("let context = unsafe { futhark_context_new(config) };")
            .line("if context.is_null() { return Err(Error::NullPtr) }")
            .line("Ok(Context { config, context })");

        let _ctx_sync = ctx
            .new_fn("sync")
            .vis("pub")
            .arg_self()
            .ret("Result<(), Error>")
            .line("let rc = unsafe { futhark_context_sync(self.context) };")
            .line("if rc != 0 { return Err(Error::Code(rc)) }")
            .line("Ok(())");

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
                    self.typemap.insert(name.clone(), info.ptr);
                }
                _ => (), // TODO
            }
        }

        for (_name, entry) in &library.manifest.entry_points {
            self.generate_entry_function(entry)?;
        }

        write!(config.output_file, "{}", self.scope.to_string())?;
        let _ = std::process::Command::new("rustfmt")
            .arg(&config.output_path)
            .status();
        Ok(())
    }
}

fn rust() -> Box<impl Generate> {
    Box::new(Rust::default())
}

fn python() -> Box<impl Generate> {
    Box::new(Python)
}

impl Config {
    pub fn detect(&self) -> Option<Box<dyn Generate>> {
        match self
            .output_path
            .extension()
            .map(|x| x.to_str().expect("Invalid extension"))
        {
            Some("rs") => Some(rust()),
            Some("py") => Some(python()),
            _ => None,
        }
    }
}
