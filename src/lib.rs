pub(crate) use std::collections::BTreeMap;

mod compiler;
mod error;
pub(crate) mod generate;
pub mod manifest;
mod package;

pub use compiler::Compiler;
pub use error::Error;
pub use generate::{Config, Generate, OCaml, Rust};
pub use manifest::Manifest;
pub use package::Package;

/// `Backend` is used to select a backend when running the `futhark` executable
#[derive(Debug, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Backend {
    /// Sequential C backend: `futhark c`
    ///
    /// Requires a C compiler
    #[serde(rename = "c")]
    C,

    /// CUDA backend: `futhark cuda`
    ///
    /// Requires the CUDA runtime and a C compiler
    #[serde(rename = "cuda")]
    CUDA,

    /// OpenCL backend: `futhark opencl`
    ///
    /// Requires OpenCL and a C compiler
    #[serde(rename = "opencl")]
    OpenCL,

    /// Multicore C backend: `futhark multicore`
    ///
    /// Requires a C compiler
    #[serde(rename = "multicore")]
    Multicore,

    /// ISPC backend: `futhark ispc`
    ///
    /// Requires the `ispc` compiler in your `$PATH`
    /// and a C compiler
    #[serde(rename = "ispc")]
    ISPC,
}

impl Backend {
    /// Get the name of a backend
    pub fn to_str(&self) -> &'static str {
        match self {
            Backend::C => "c",
            Backend::CUDA => "cuda",
            Backend::OpenCL => "opencl",
            Backend::Multicore => "multicore",
            Backend::ISPC => "ispc",
        }
    }

    /// Returns the C libraries that need to be linked for a backend
    pub fn required_c_libs(&self) -> &'static [&'static str] {
        match self {
            Backend::CUDA => &["cuda", "cudart", "nvrtc", "m"],
            Backend::OpenCL => &["OpenCL", "m"],
            Backend::Multicore | Backend::ISPC => &["pthread", "m"],
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
