extern crate vergen;

fn main() {
    if let Err(err) = vergen::vergen(vergen::OutputFns::all()) {
        panic!(
            "Vergen crate failed to generate version information! {:?}",
            err
        );
    }

    #[cfg(not(target_os = "emscripten"))]
    println!("cargo:rustc-cfg=parallel");
}
