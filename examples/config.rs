extern crate amethyst;

use std::path::Path;

fn main() {
	println!("{:?}", amethyst::config::Config::from_file(&Path::new("config/config.yml")));
}