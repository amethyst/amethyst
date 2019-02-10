use vergen::{self, ConstantsFlags};

fn main() {
    vergen::generate_cargo_keys(ConstantsFlags::all())
        .unwrap_or_else(|e| panic!("Vergen crate failed to generate version information! {}", e));

    println!("cargo:rerun-if-changed=build.rs");
}
