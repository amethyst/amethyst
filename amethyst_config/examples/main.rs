use amethyst_config::{Config, ConfigFormat};
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
    let ron_path = format!("{}/examples/config.{}", env!("CARGO_MANIFEST_DIR"), "ron");
    match ExampleConfig::load(&ron_path) {
        Ok(cfg) => {
            println!("RON Config Result:\n{:#?}", cfg);

            if let Err(e) = cfg.write_format(ConfigFormat::Ron, &ron_path) {
                println!("Error:\n{}", e);
            }
        }
        Err(e) => println!("{:?}", e),
    }

    #[cfg(feature = "json")]
    {
        let json_path = format!("{}/examples/config.{}", env!("CARGO_MANIFEST_DIR"), "json");
        match ExampleConfig::load(&json_path) {
            Ok(cfg) => {
                println!("JSON Config Result:\n{:#?}", cfg);

                if let Err(e) = cfg.write_format(ConfigFormat::Json, &json_path) {
                    println!("Error:\n{}", e);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }

    #[cfg(feature = "binary")]
    {
        let binary_path = format!("{}/examples/config.{}", env!("CARGO_MANIFEST_DIR"), "bin");
        match ExampleConfig::load(&binary_path) {
            Ok(cfg) => {
                println!("Binary Config Result:\n{:#?}", cfg);

                if let Err(e) = cfg.write_format(ConfigFormat::Binary, &binary_path) {
                    println!("Error:\n{}", e);
                }
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
