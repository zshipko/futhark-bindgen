use futhark_bindgen::{build_in_out_dir, Backend};

fn main() {
    build_in_out_dir(Backend::Multicore, "example.fut", "example.rs")
}
