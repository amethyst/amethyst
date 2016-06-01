#[macro_use]
extern crate amethyst;
extern crate yaml_rust;

use amethyst::config::Element;
use std::path::Path;

fn main() {
    let config = amethyst::config::Config::from_file(Path::new("config/config.yml"));

    match config {
        Ok(conf) => {
        println!("{}", conf.to_string());
    },
    Err(e) => println!("{:?}", e),
}