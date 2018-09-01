extern crate vergen;

use vergen::{ConstantsFlags, Vergen};

fn main() {
    let vergen = match Vergen::new(ConstantsFlags::all()) {
        Ok(v) => v,
        Err(err) => {
            panic!(
                "Vergen crate failed to generate version information! {:?}",
                err
            );
        }
    };

    for (k, v) in vergen.build_info() {
        println!("cargo:rustc-env={}={}", k.name(), v);
    }
}
