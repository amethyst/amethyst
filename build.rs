extern crate vergen;

fn main() {
    if let Err(err) = vergen::vergen(vergen::OutputFns::all()) {
        panic!(
            "Vergen crate failed to generate version information! {:?}",
            err
        );
    }

    use std::env;
    match (env::var("CARGO_FEATURE_PARALLEL"), env::var("CARGO_FEATURE_SERIAL")) {
        (Some(_), Some(_)) => panic!("Can not compile in both parallel and serial mode"),
        (None, None) => {
            #[cfg(not(target_os = "emscripten"))]
            println!("cargo:rust-cfg:parallel");
        }
    }
}
