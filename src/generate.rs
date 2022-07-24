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
}

const RUST_TYPE_MAP: &[(&'static str, &'static str)] = &[];

impl Default for Rust {
    fn default() -> Self {
        let typemap = RUST_TYPE_MAP
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        Rust { typemap }
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

    pub fn gen(self, scope: &mut codegen::Scope) {
        let mut s = format!("unsafe extern \"C\" fn {}(", self.name);
        for arg in &self.args {
            s += &format!("{}: {}, ", arg.0, arg.1);
        }
        s += ")";
        if !self.ret.is_empty() {
            s += "-> ";
            s += &self.ret;
        }
        s += ";\n";
        scope.raw(&s);
    }
}

impl Rust {
    fn generate_array_type(
        scope: &mut codegen::Scope,
        a: &manifest::ArrayType,
    ) -> Result<String, Error> {
        let original_name = format!("futhark_{}_{}d", a.elemtype.to_str(), a.rank);
        let name = format!("Array_{}_{}d", a.elemtype.to_str(), a.rank);
        scope.new_struct(&original_name).repr("C");

        scope
            .new_struct(&name)
            .field("inner", format!("*mut {original_name}"));

        let mut new = ExternFn::new(format!("futhark_new_{}_{}d", a.elemtype.to_str(), a.rank))
            .arg("_", "*mut futhark_context")
            .arg("_", format!("*mut {}", a.elemtype.to_str()));

        for i in 0..a.rank {
            new = new.arg(&format!("dim{i}"), "i64");
        }

        new.ret(&format!("*mut {original_name}")).gen(scope);

        // free
        let _free = ExternFn::new(format!("futhark_free_{}_{}d", a.elemtype.to_str(), a.rank))
            .arg("_", "*mut futhark_context")
            .arg("_", &format!("*mut {name}"))
            .ret("std::os::raw::c_int")
            .gen(scope);

        Ok(name)
    }
}

impl Generate for Rust {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error> {
        let mut scope = codegen::Scope::new();

        scope.new_struct("futhark_context_config").repr("C");

        ExternFn::new("futhark_context_config_new")
            .ret("*mut futhark_context_config")
            .gen(&mut scope);
        ExternFn::new("futhark_context_config_free")
            .arg("_", "*mut futhark_context_config")
            .gen(&mut scope);

        scope.new_struct("futhark_context").repr("C");

        ExternFn::new("futhark_context_new")
            .arg("config", "*mut futhark_context_config")
            .ret("*mut futhark_context")
            .gen(&mut scope);
        ExternFn::new("futhark_context_free")
            .arg("_", "*mut futhark_context")
            .gen(&mut scope);

        for (_name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(a) => {
                    Rust::generate_array_type(&mut scope, a)?;
                }
                _ => (), // TODO
            }
        }

        write!(config.output_file, "{}", scope.to_string())?;
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
