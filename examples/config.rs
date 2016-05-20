#[macro_use]
extern crate amethyst;
extern crate yaml_rust;

use amethyst::config::Element;
use std::path::{PathBuf, Path};

use yaml_rust::Yaml;

fn main() {
  let config = amethyst::config::Config::default();

  /*match config {
    Ok(conf) => {
      println!("{:?}", conf);

      println!("{:?}", conf.write_file());
    },
    Err(e) => println!("{:?}", e),
  };*/

  config.write_file();
}