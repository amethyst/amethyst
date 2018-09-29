pub use log::LevelFilter;

use fern::{self, FormatCallback};
use log;
use std::{env, fmt, io, str::FromStr};

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
        let use_colors = env::var_os("AMETHYST_LOG_DISABLE_COLORS").is_none();
        let level_filter = env::var("AMETHYST_LOG_LEVEL_FILTER").ok()
            .and_then(|lf| LevelFilter::from_str(&lf).ok())
            .unwrap_or(LevelFilter::Debug);

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
    if env::var("AMETHYST_LOG_DISABLE_COLORS").is_ok() {
        config.use_colors = false;
    }
    if let Ok(lf) = env::var("AMETHYST_LOG_LEVEL_FILTER") {
        config.level_filter = LevelFilter::from_str(&lf).unwrap_or(LevelFilter::Debug)
    }

    let color_config = fern::colors::ColoredLevelConfig::new();
    let format = move |out: FormatCallback, message: &fmt::Arguments, record: &log::Record| {
        let (color, color_reset) = if config.use_colors {
            let color_code = color_config.get_color(&record.level()).to_fg_str();
            (format!("\x1B[{}m", color_code), "\x1B[0m")
        } else {
            (String::new(), "")
        };

        out.finish(format_args!(
            "{color}[{level}][{target}] {message}{color_reset}",
            color = color,
            level = record.level(),
            target = record.target(),
            message = message,
            color_reset = color_reset
        ))
    };

    fern::Dispatch::new()
        .format(format)
        .level(config.level_filter)
        .chain(io::stdout())
        .apply()
        .unwrap_or_else(|_| {
            debug!("Global logger already set, default Amethyst logger will not be used")
        });
}
