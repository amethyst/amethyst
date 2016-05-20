#[macro_use]
extern crate amethyst;
extern crate yaml_rust;

use amethyst::config::Element;
use std::path::{PathBuf, Path};

use yaml_rust::Yaml;

fn main() {
  let config = amethyst::config::Config::default();

  println!("{:?}", config.write_file());
}