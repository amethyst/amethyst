extern crate amethyst_config;

use amethyst_config::Element;

fn main() {
    let config = amethyst_config::Config::from_file("../../config/config.yml");

    match config {
        Ok(conf) => {
            println!("{}", conf.to_string());

            if let Err(e) = conf.write_file() {
                println!("{}", e);
            }
        },
        Err(e) => println!("{:?}", e),
    }
}
