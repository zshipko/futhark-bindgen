fn main() {
    futhark_bindgen::build(futhark_bindgen::Backend::C, "example.fut", "example.rs")
}
