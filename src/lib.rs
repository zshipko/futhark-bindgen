pub(crate) use std::collections::BTreeMap;

mod error;
pub(crate) mod generate;
pub mod manifest;

pub use error::Error;
pub use generate::{Config, Generate, OCaml, Rust};
pub use manifest::Manifest;

#[derive(Debug, serde::Deserialize, Clone, Copy)]
pub enum Backend {
    #[serde(rename = "c")]
    C,

    #[serde(rename = "cuda")]
    CUDA,

    #[serde(rename = "opencl")]
    OpenCL,

    #[serde(rename = "multicore")]
    Multicore,

    #[serde(rename = "python")]
    Python,

    #[serde(rename = "pyopencl")]
    PyOpenCL,
}

impl Backend {
    pub fn to_str(&self) -> &'static str {
        match self {
            Backend::C => "c",
            Backend::CUDA => "cuda",
            Backend::OpenCL => "opencl",
            Backend::Multicore => "multicore",
            Backend::Python => "python",
            Backend::PyOpenCL => "pyopencl",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Compiler {
    exe: String,
    backend: Backend,
    src: std::path::PathBuf,
    extra_args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Library {
    pub manifest: Manifest,
    pub c_file: std::path::PathBuf,
    pub h_file: std::path::PathBuf,
    pub src: std::path::PathBuf,
}

impl Library {
    pub fn required_c_libs(&self) -> &'static [&'static str] {
        match self.manifest.backend {
            Backend::CUDA => &["cuda", "cudart", "nvrtc", "m"],
            Backend::OpenCL => &["OpenCL", "m"],
            _ => &[],
        }
    }

    #[cfg(feature = "build")]
    pub fn link(&self) {
        cc::Build::new()
            .flag("-Wno-unused-parameter")
            .file(&self.c_file)
            .compile("futhark_generate");
        println!("cargo:rustc-link-lib=futhark_generate");

        let libs = self.required_c_libs();

        for lib in libs {
            println!("cargo:rustc-link-lib={}", lib);
        }
    }
}

#[cfg(feature = "build")]
pub fn build(
    backend: Backend,
    src: impl AsRef<std::path::Path>,
    dest: impl AsRef<std::path::Path>,
) {
    let lib = Compiler::new(backend, src)
        .compile()
        .expect("Compilation failed")
        .expect("Unable to find manifest file");
    let mut config = Config::new(dest).expect("Unable to configure codegen");
    let mut gen = config.detect().expect("Invalid output language");
    gen.generate(&lib, &mut config)
        .expect("Code generation failed");
    lib.link();
}

impl Compiler {
    pub fn new(backend: Backend, src: impl AsRef<std::path::Path>) -> Compiler {
        Compiler {
            exe: String::from("futhark"),
            src: src.as_ref().to_path_buf(),
            extra_args: Vec::new(),
            backend,
        }
    }

    pub fn set_executable_name(&mut self, name: impl AsRef<str>) {
        self.exe = name.as_ref().into();
    }

    pub fn set_extra_args(&mut self, args: Vec<String>) {
        self.extra_args = args;
    }

    pub fn compile(&self) -> Result<Option<Library>, Error> {
        let ok = std::process::Command::new(&self.exe)
            .arg(self.backend.to_str())
            .args(&self.extra_args)
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

        let manifest = Manifest::parse_file(self.src.with_extension("json"))?;
        let c_file = self.src.with_extension("c");
        let h_file = self.src.with_extension("h");
        Ok(Some(Library {
            manifest,
            c_file,
            h_file,
            src: self.src.clone(),
        }))
    }
}
