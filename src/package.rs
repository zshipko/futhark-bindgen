use crate::*;

/// Compiled Futhark package
#[derive(Debug, Clone)]
pub struct Package {
    /// Manifest, parsed from the manifest file
    pub manifest: Manifest,

    /// Path to the generated C file
    pub c_file: std::path::PathBuf,

    /// Path to the generate C header file
    pub h_file: std::path::PathBuf,

    /// Source file
    pub src: std::path::PathBuf,
}

impl Package {
    /// Link the package
    ///
    /// Note: This should only be used in `build.rs`
    #[cfg(feature = "build")]
    pub fn link(&self) {
        let project = std::env::var("CARGO_PKG_NAME").unwrap();

        let name = format!("futhark_generate_{project}");

        if self.manifest.backend == Backend::ISPC {
            let kernels = self.c_file.with_extension("kernels.ispc");
            let dest = kernels.with_extension("o");
            std::process::Command::new("ispc")
                .arg(&kernels)
                .arg("-o")
                .arg(&dest)
                .arg("--pic")
                .arg("--addressing=64")
                .arg("--target=host")
                .status()
                .expect("Unable to run ispc");

            cc::Build::new()
                .file(&self.c_file)
                .object(&dest)
                .flag("-fPIC")
                .flag("-pthread")
                .flag("-lm")
                .flag("-std=c99")
                .extra_warnings(false)
                .warnings(false)
                .compile(&name);
        } else {
            cc::Build::new()
                .flag("-std=c99")
                .flag("-Wno-unused-parameter")
                .file(&self.c_file)
                .extra_warnings(false)
                .warnings(false)
                .compile(&name);
        }
        println!("cargo:rustc-link-lib={name}");

        let libs = self.manifest.backend.required_c_libs();

        for lib in libs {
            if cfg!(target_os = "macos") && lib == &"OpenCL" {
                println!("cargo:rustc-link-lib=framework={}", lib);
            } else {
                println!("cargo:rustc-link-lib={}", lib);
            }
        }
    }
}
