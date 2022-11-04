use crate::*;

/// Wrapper around the Futhark compiler
#[derive(Debug, Clone)]
pub struct Compiler {
    exe: String,
    backend: Backend,
    src: std::path::PathBuf,
    extra_args: Vec<String>,
    output_dir: std::path::PathBuf,
}

impl Compiler {
    /// Create a new `Compiler` instance with the selected backend and Futhark source file
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

    /// By default the executable name is set to `futhark`, this function can be
    /// used to set a different name or path
    pub fn with_executable_name(mut self, name: impl AsRef<str>) -> Self {
        self.exe = name.as_ref().into();
        self
    }

    /// Supply additional arguments to be passed to the `futhark` executable
    pub fn with_extra_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }

    /// Set the output directory where the C files and manifest will be created
    pub fn with_output_dir(mut self, dir: impl AsRef<std::path::Path>) -> Self {
        self.output_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Compile the package
    ///
    /// This will generate a C file, C header file and manifest
    pub fn compile(&self) -> Result<Package, Error> {
        // Create -o argument
        let output = &self
            .output_dir
            .join(self.src.with_extension("").file_name().unwrap());

        let ok = std::process::Command::new(&self.exe)
            .arg(self.backend.to_str())
            .args(&self.extra_args)
            .args(["-o", &output.to_string_lossy()])
            .arg("--lib")
            .arg(&self.src)
            .status()?
            .success();

        if !ok {
            return Err(Error::CompilationFailed);
        }

        // Load manifest after successful compilation
        let manifest = Manifest::parse_file(output.with_extension("json"))?;
        let c_file = output.with_extension("c");
        let h_file = output.with_extension("h");
        Ok(Package {
            manifest,
            c_file,
            h_file,
            src: self.src.clone(),
        })
    }
}
