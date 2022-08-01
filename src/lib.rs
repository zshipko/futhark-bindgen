pub(crate) use std::collections::BTreeMap;

mod compiler;
mod error;
pub(crate) mod generate;
mod library;
pub mod manifest;

pub use compiler::Compiler;
pub use error::Error;
pub use generate::{Config, Generate, OCaml, Rust};
pub use library::Library;
pub use manifest::Manifest;

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Backend {
    #[serde(rename = "c")]
    C,

    #[serde(rename = "cuda")]
    CUDA,

    #[serde(rename = "opencl")]
    OpenCL,

    #[serde(rename = "multicore")]
    Multicore,

    #[serde(rename = "ispc")]
    ISPC,

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
            Backend::ISPC => "ispc",
            Backend::Python => "python",
            Backend::PyOpenCL => "pyopencl",
        }
    }

    pub fn required_c_libs(&self) -> &'static [&'static str] {
        match self {
            Backend::CUDA => &["cuda", "cudart", "nvrtc", "m"],
            Backend::OpenCL => &["OpenCL", "m"],
            Backend::Multicore => &["pthread", "m"],
            _ => &[],
        }
    }
}

#[cfg(feature = "build")]
/// Generate the bindings and link the Futhark C code
///
/// `backend` selects the backend to use when generating C code: `futhark $backend --lib`
///
/// `src` is the full path to your Futhark code
///
/// `dest` is expected to be a relative path that will
// be appended to `$OUT_DIR`
pub fn build(
    backend: Backend,
    src: impl AsRef<std::path::Path>,
    dest: impl AsRef<std::path::Path>,
) {
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest = std::path::PathBuf::from(&out).join(dest);
    let lib = Compiler::new(backend, src)
        .with_output_dir(out)
        .compile()
        .expect("Compilation failed")
        .expect("Unable to find manifest file");

    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join(dest);
    let mut config = Config::new(out).expect("Unable to configure codegen");
    let mut gen = config.detect().expect("Invalid output language");
    gen.generate(&lib, &mut config)
        .expect("Code generation failed");
    lib.link();
}
