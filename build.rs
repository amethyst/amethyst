extern crate vergen;

fn main() {
    if let Err(err) = vergen::vergen(vergen::OutputFns::all()) {
        panic!(
            "Vergen crate failed to generate version information! {:?}",
            err
        );
    }

    use std::env;
    if let Ok(target) = env::var("TARGET") {
        match target.as_ref() {
            "asmjs-unknown-emscripten" => { },
            "wasm32-unknown-emscripten" => { },
            "wasm32-unknown-unknown" => { },
            _ => println!("cargo:rustc-cfg=parallel"),
        }
    } else {
        panic!("Could not identify the building target");
    }
}
