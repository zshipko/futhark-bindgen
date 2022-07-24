use crate::*;

pub struct Config {
    pub output_file: std::path::PathBuf,
}

impl Config {
    pub fn new(output: impl AsRef<std::path::Path>) -> Config {
        Config {
            output_file: output.as_ref().to_path_buf(),
        }
    }
}

pub trait Codegen {
    fn generate(&mut self, library: &Library, config: &Config) -> Result<(), Error>;
}

pub struct Python;

impl Codegen for Python {
    fn generate(&mut self, library: &Library, config: &Config) -> Result<(), Error> {
        match &library.manifest.backend {
            Backend::Python | Backend::PyOpenCL => {
                if &config.output_file != &library.py_file {
                    std::fs::copy(&library.py_file, &config.output_file)?;
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

impl Codegen for Rust {
    fn generate(&mut self, library: &Library, config: &Config) -> Result<(), Error> {
        Ok(())
    }
}
