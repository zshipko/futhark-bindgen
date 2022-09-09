fn main() {
    futhark_bindgen::build(
        futhark_bindgen::Backend::from_env().unwrap_or(futhark_bindgen::Backend::C),
        "example.fut",
        "example.rs",
    )
}
