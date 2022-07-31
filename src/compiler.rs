use crate::*;

#[derive(Debug, Clone)]
pub struct Compiler {
    exe: String,
    backend: Backend,
    src: std::path::PathBuf,
    extra_args: Vec<String>,
    output_dir: std::path::PathBuf,
}

impl Compiler {
    pub fn new(backend: Backend, src: impl AsRef<std::path::Path>) -> Compiler {
        Compiler {
            exe: String::from("futhark"),
            src: src.as_ref().to_path_buf(),
            extra_args: Vec::new(),
            output_dir: src
                .as_ref()
                .canonicalize()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf(),
            backend,
        }
    }

    pub fn with_executable_name(mut self, name: impl AsRef<str>) -> Self {
        self.exe = name.as_ref().into();
        self
    }

    pub fn with_extra_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }

    pub fn with_output_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.output_dir = dir.as_ref().to_path_buf();
        self
    }

    pub fn compile(&self) -> Result<Option<Library>, Error> {
        let output = &self
            .output_dir
            .join(self.src.with_extension("").file_name().unwrap());
        let ok = std::process::Command::new(&self.exe)
            .arg(self.backend.to_str())
            .args(&self.extra_args)
            .args(&["-o", &output.to_string_lossy()])
            .arg("--lib")
            .arg(&self.src)
            .status()?
            .success();

        if !ok {
            return Err(Error::CompilationFailed);
        }

        match &self.backend {
            Backend::Python | Backend::PyOpenCL => return Ok(None),
            _ => (),
        }

        let manifest = Manifest::parse_file(output.with_extension("json"))?;
        let c_file = output.with_extension("c");
        let h_file = output.with_extension("h");
        Ok(Some(Library {
            manifest,
            c_file,
            h_file,
            src: self.src.clone(),
        }))
    }
}
