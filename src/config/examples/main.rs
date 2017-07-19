
#[macro_use]
extern crate amethyst_config;

use amethyst_config::Config;

config! {
    #[derive(Debug)]
    pub struct DisplayConfig {
        pub title: String = "Amethyst game".to_string(),
        pub brightness: f64 = 1.0,
        pub fullscreen: bool = false,
        pub dimensions: (u16, u16) = (1024, 768),
        pub min_dimensions: Option<(u16, u16)> = None,
        pub max_dimensions: Option<(u16, u16)> = None,
        pub vsync: bool = true,
        pub multisampling: u16 = 0,
        pub visibility: bool = true,
    }
}

config! {
    #[derive(Debug)]
    pub struct LoggingConfig {
        pub file_path: String = "new_project.log".to_string(),
        pub output_level: String = "warn".to_string(),
        pub logging_level: String = "debug".to_string(),
    }
}

config! {
    #[derive(Debug)]
    pub struct ExampleConfig {
        /// Configuration for display and graphics
        pub display: DisplayConfig = DisplayConfig::default(),
        /// Configuration for output
        pub logging: LoggingConfig = LoggingConfig::default(),
    }
}

fn main() {
    let path = format!("{}/examples/config.yml", env!("CARGO_MANIFEST_DIR"));
    let res = ExampleConfig::load_no_fallback(&path);

    match res {
        Ok(cfg) => {
            println!("{:#?}", cfg);

            if let Err(e) = cfg.write(&path) {
                println!("{}", e);
            }
        },
        Err(e) => println!("{:?}", e),
    }
}
