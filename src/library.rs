use crate::*;

#[derive(Debug, Clone)]
pub struct Library {
    pub manifest: Manifest,
    pub c_file: std::path::PathBuf,
    pub h_file: std::path::PathBuf,
    pub src: std::path::PathBuf,
}

impl Library {
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
                .warnings(false)
                .compile(&name);
        } else {
            cc::Build::new()
                .flag("-Wno-unused-parameter")
                .file(&self.c_file)
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
