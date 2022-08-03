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

/// Code generation config
pub struct Config {
    /// Output file
    pub output_path: std::path::PathBuf,

    /// Path to output file
    pub output_file: std::fs::File,
}

impl Config {
    /// Create a new config using the provided output file path
    pub fn new(output: impl AsRef<std::path::Path>) -> Result<Config, Error> {
        Ok(Config {
            output_path: output.as_ref().to_path_buf(),
            output_file: std::fs::File::create(output)?,
        })
    }
}

pub trait Generate {
    /// Iterates through the manifest and generates code
    fn generate(&mut self, pkg: &Package, config: &mut Config) -> Result<(), Error> {
        self.bindings(pkg, config)?;
        for (name, ty) in &pkg.manifest.types {
            match ty {
                manifest::Type::Array(ty) => {
                    self.array_type(pkg, config, name, ty)?;
                }
                manifest::Type::Opaque(ty) => {
                    self.opaque_type(pkg, config, name, ty)?;
                }
            }
        }

        for (name, entry) in &pkg.manifest.entry_points {
            self.entry(pkg, config, name, entry)?;
        }
        self.format(&config.output_path)?;
        Ok(())
    }

    /// Step 1: generate any setup code or low-level bindings
    fn bindings(&mut self, _pkg: &Package, _config: &mut Config) -> Result<(), Error> {
        Ok(())
    }

    /// Step 2: generate code for array types
    fn array_type(
        &mut self,
        pkg: &Package,
        config: &mut Config,
        name: &str,
        ty: &manifest::ArrayType,
    ) -> Result<(), Error>;

    /// Step 3: generate code for opaque types
    fn opaque_type(
        &mut self,
        pkg: &Package,
        config: &mut Config,
        name: &str,
        ty: &manifest::OpaqueType,
    ) -> Result<(), Error>;

    /// Generate code for entry functions
    fn entry(
        &mut self,
        pkg: &Package,
        config: &mut Config,
        name: &str,
        entry: &manifest::Entry,
    ) -> Result<(), Error>;

    /// Run any formatting program or post-processing on the output file
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
    /// Automatically detect output language
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
