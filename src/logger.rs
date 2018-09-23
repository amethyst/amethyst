use fern;
pub use log::LevelFilter;
use std::env;
use std::io;

/// Logger configuration object.
#[derive(Clone, Copy)]
pub struct LoggerConfig {
    /// Whether to use color output when logging to the terminal or not.
    pub use_colors: bool,
    /// Sets the overarching level filter for the logger.
    pub level_filter: LevelFilter,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        let use_colors = if let Ok(_) = env::var("AMETHYST_LOG_DISABLE_COLORS") {
            false
        } else {
            true
        };
        let level_filter = if let Ok(lf) = env::var("AMETHYST_LOG_LEVEL_FILTER") {
            use std::str::FromStr;
            LevelFilter::from_str(&lf).unwrap_or(LevelFilter::Debug)
        } else {
            LevelFilter::Debug
        };
        LoggerConfig {
            use_colors,
            level_filter,
        }
    }
}

/// Starts a basic logger outputting to stdout with color on supported platforms.
///
/// If you do not intend on using the logger builtin to Amethyst, it's highly recommended you
/// initialise your own.
///
/// Configuration of the logger can also be controlled via environment variables:
///
/// * AMETHYST_LOG_DISABLE_COLORS - if set, disables colors for the log output
/// * AMETHYST_LOG_LEVEL_FILTER - sets the log level
///
pub fn start_logger(mut config: LoggerConfig) {
    if let Ok(_) = env::var("AMETHYST_LOG_DISABLE_COLORS") {
        config.use_colors = false;
    }
    if let Ok(lf) = env::var("AMETHYST_LOG_LEVEL_FILTER") {
        use std::str::FromStr;
        config.level_filter = LevelFilter::from_str(&lf).unwrap_or(LevelFilter::Debug)
    }
    let color_config = fern::colors::ColoredLevelConfig::new();

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color}[{level}][{target}] {message}{color_reset}",
                color = if config.use_colors {
                    format!(
                        "\x1B[{}m",
                        color_config.get_color(&record.level()).to_fg_str()
                    )
                } else {
                    String::from("")
                },
                level = record.level(),
                target = record.target(),
                message = message,
                color_reset = if config.use_colors { "\x1B[0m" } else { "" }
            ))
        })
        .level(config.level_filter)
        .chain(io::stdout())
        .apply()
        .unwrap_or_else(|_| {
            debug!("Global logger already set, default Amethyst logger will not be used")
        });
}
