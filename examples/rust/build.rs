fn main() {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("example.rs");
    futhark_bindgen::build(futhark_bindgen::Backend::C, "example.fut", dest)
}
