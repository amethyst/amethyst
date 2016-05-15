extern crate amethyst;

use std::path::Path;

fn main() {
	let config = amethyst::config::Config::from_file(&Path::new("config/config.yml"));

  match config {
    Ok(conf) => println!("{:?}", conf),
    Err(e) => println!("{:?}", e),
  }
}