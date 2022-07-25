use crate::*;

mod python;
mod rust;

pub use python::Python;
pub use rust::Rust;

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
