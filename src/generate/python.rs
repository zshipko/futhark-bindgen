use crate::*;

pub struct Python;

impl Generate for Python {
    fn generate(&mut self, library: &Library, config: &mut Config) -> Result<(), Error> {
        match &library.manifest.backend {
            Backend::Python | Backend::PyOpenCL => {
                if &config.output_path != &library.py_file {
                    std::fs::copy(&library.py_file, &config.output_path)?;
                }
                Ok(())
            }
            _ => panic!("Python codegen must use a Python backend"),
        }
    }
}
