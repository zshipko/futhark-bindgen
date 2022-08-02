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
    pub auto_sync: bool,
}

impl Config {
    pub fn new(output: impl AsRef<std::path::Path>) -> Result<Config, Error> {
        Ok(Config {
            output_path: output.as_ref().to_path_buf(),
            output_file: std::fs::File::create(output)?,
            auto_sync: true,
        })
    }

    pub fn no_auto_sync(mut self) -> Self {
        self.auto_sync = false;
        self
    }
}

pub trait Generate {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error>;
}

fn rust() -> Box<impl Generate> {
    Box::new(Rust::default())
}

fn ocaml() -> Box<impl Generate> {
    Box::new(OCaml::default())
}

impl Config {
    pub fn detect(&self) -> Option<Box<dyn Generate>> {
        match self
            .output_path
            .extension()
            .map(|x| x.to_str().expect("Invalid extension"))
        {
            Some("rs") => Some(rust()),
            Some("ml") => Some(ocaml()),
            _ => None,
        }
    }
}
