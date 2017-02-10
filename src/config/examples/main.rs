extern crate amethyst_config;

use amethyst_config::{Config, Element};

fn main() {
    let res = Config::from_file("../../config/config.yml");

    match res {
        Ok(cfg) => {
            println!("{}", cfg.to_string());

            if let Err(e) = cfg.write_file() {
                println!("{}", e);
            }
        },
        Err(e) => println!("{:?}", e),
    }
}
