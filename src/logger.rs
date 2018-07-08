use std::io;

use fern;
pub use log::LevelFilter;

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
        LoggerConfig {
            use_colors: true,
            level_filter: LevelFilter::Debug,
        }
    }
}

/// Starts a basic logger outputting to stdout with color on supported platforms.
///
/// If you do not intend on using the logger builtin to Amethyst, it's highly recommended you
/// initialise your own.
pub fn start_logger(config: LoggerConfig) {
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
