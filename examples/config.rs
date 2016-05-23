#[macro_use]
extern crate amethyst;
extern crate yaml_rust;

use amethyst::config::{Element, ConfigMeta, ConfigError};
use std::path::{PathBuf, Path};

use yaml_rust::Yaml;

config!(Test {
	field: i32 = 50,
});

fn main() {
  let config = amethyst::config::Config::from_file(Path::new("config/config.yml"));

  match config {
    Ok(conf) => {
        conf.write_file();
    },
    Err(e) => println!("{:?}", e),
  }
}