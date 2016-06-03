#[macro_use]
extern crate amethyst_config;
extern crate yaml_rust;

use amethyst_config::Element;
use std::path::Path;

fn main() {
    let config = amethyst_config::Config::from_file(Path::new("../../config/config.yml"));

    match config {
        Ok(conf) => {
            println!("{}", conf.to_string());
        },
        Err(e) => println!("{:?}", e),
    }
}