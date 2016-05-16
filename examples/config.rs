extern crate amethyst;

use amethyst::config::{FromFile, ConfigMeta};
use std::path::Path;

fn main() {
  // should be the root config
	let config = amethyst::config::Config::from_file(&Path::new("config\\config.yml"));

  match config {
    Ok(conf) => println!("{:?}", conf),
    Err(e) => println!("{:?}", e),
  }
}