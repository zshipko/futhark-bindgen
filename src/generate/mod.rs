use crate::*;

mod ocaml;
mod rust;

pub use ocaml::OCaml;
pub use rust::Rust;

pub(crate) fn first_uppercase(s: &str) -> String {
    let mut s = s.to_string();
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
    s
}

pub(crate) fn convert_struct_name(s: &str) -> &str {
    s.strip_prefix("struct")
        .unwrap()
        .strip_suffix('*')
        .unwrap()
        .strip_prefix(|x: char| x.is_ascii_whitespace())
        .unwrap()
        .strip_suffix(|x: char| x.is_ascii_whitespace())
        .unwrap()
}

/*pub(crate) fn first_lowercase(s: &str) -> String {
    let mut s = s.to_string();
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
    s
}*/

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
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error> {
        self.bindings(library, config)?;
        for (name, ty) in &library.manifest.types {
            match ty {
                manifest::Type::Array(ty) => {
                    self.array_type(library, config, &name, ty)?;
                }
                manifest::Type::Opaque(ty) => {
                    self.opaque_type(library, config, &name, ty)?;
                }
            }
        }

        for (name, entry) in &library.manifest.entry_points {
            self.entry(library, config, &name, entry)?;
        }
        self.format(&config.output_path)?;
        Ok(())
    }

    fn bindings(&mut self, _library: &Library, _config: &mut Config) -> Result<(), Error> {
        Ok(())
    }

    fn array_type(
        &mut self,
        library: &Library,
        config: &mut Config,
        name: &str,
        ty: &manifest::ArrayType,
    ) -> Result<(), Error>;

    fn opaque_type(
        &mut self,
        library: &Library,
        config: &mut Config,
        name: &str,
        ty: &manifest::OpaqueType,
    ) -> Result<(), Error>;

    fn entry(
        &mut self,
        library: &Library,
        config: &mut Config,
        name: &str,
        entry: &manifest::Entry,
    ) -> Result<(), Error>;

    fn format(&mut self, _output: &std::path::Path) -> Result<(), Error> {
        Ok(())
    }
}

fn rust() -> Box<impl Generate> {
    Box::new(Rust::default())
}

fn ocaml(config: &Config) -> Box<impl Generate> {
    Box::new(OCaml::new(config).unwrap())
}

impl Config {
    pub fn detect(&self) -> Option<Box<dyn Generate>> {
        match self
            .output_path
            .extension()
            .map(|x| x.to_str().expect("Invalid extension"))
        {
            Some("rs") => Some(rust()),
            Some("ml") => Some(ocaml(self)),
            _ => None,
        }
    }
}
