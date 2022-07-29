use futhark_bindgen::{build_in_out_dir, Backend};

fn main() {
    build_in_out_dir(Backend::C, "example.fut", "example.rs")
}
