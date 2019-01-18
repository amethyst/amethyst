use amethyst_config::Config;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DisplayConfig {
    pub title: String,
    pub brightness: f64,
    #[serde(default)]
    pub fullscreen: bool,
    pub dimensions: (u16, u16),
    pub min_dimensions: Option<(u16, u16)>,
    pub max_dimensions: Option<(u16, u16)>,
    pub vsync: bool,
    pub multisampling: u16,
    pub visibility: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub file_path: String,
    pub output_level: String,
    pub logging_level: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ExampleConfig {
    /// Configuration for display and graphics
    pub display: DisplayConfig,
    /// Configuration for output
    pub logging: LoggingConfig,
}

fn main() {
    let path = format!("{}/examples/display_config.ron", env!("CARGO_MANIFEST_DIR"));
    let res = ExampleConfig::load_no_fallback(&path);

    match res {
        Ok(cfg) => {
            println!("{:#?}", cfg);

            if let Err(e) = cfg.write(&path) {
                println!("{}", e);
            }
        }
        Err(e) => println!("{:?}", e),
    }
}
