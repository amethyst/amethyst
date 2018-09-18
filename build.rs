extern crate vergen;

use vergen::{ConstantsFlags, Vergen};

fn main() {
    let vergen = Vergen::new(ConstantsFlags::all()).unwrap_or_else(|e| {
        panic!(
            "Vergen crate failed to generate version information! {:?}",
            e
        );
    });

    for (k, v) in vergen.build_info() {
        println!("cargo:rustc-env={}={}", k.name(), v);
    }
}
