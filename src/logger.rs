pub use log::LevelFilter;

use fern;
use std::{env, io, path::PathBuf, str::FromStr};

/// Logger configuration object.
#[derive(Clone)]
pub struct LoggerConfig {
    /// Determines whether to log to the terminal or not.
    pub use_stdout: bool,
    /// Whether to use color output when logging to the terminal or not.
    pub use_colors: bool,
    /// Sets the overarching level filter for the logger.
    pub level_filter: LevelFilter,
    /// If set, enables logging to file at the given path.
    pub log_file: Option<PathBuf>,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            use_stdout: env::var_os("AMETHYST_LOG_DISABLE_STDOUT").is_none(),
            use_colors: env::var_os("AMETHYST_LOG_DISABLE_COLORS").is_none(),
            level_filter: env::var("AMETHYST_LOG_LEVEL_FILTER").ok()
                .and_then(|lf| LevelFilter::from_str(&lf).ok())
                .unwrap_or(LevelFilter::Debug),
            log_file: env::var("AMETHYST_LOG_FILE_PATH").ok()
                .map(|path| PathBuf::from(path)),
        }
    }
}

/// Starts a basic logger outputting to stdout with color on supported platforms.
///
/// If you do not intend on using the logger builtin to Amethyst, it's highly recommended you
/// initialise your own.
///
/// Configuration of the logger can also be controlled via environment variables:
/// * AMETHYST_LOG_DISABLE_STDOUT - if set, disables logging to the terminal
/// * AMETHYST_LOG_DISABLE_COLORS - if set, disables colors for the log output
/// * AMETHYST_LOG_LEVEL_FILTER - sets the log level
/// * AMETHYST_LOG_FILE_PATH - if set, enables logging to the file at the path
pub fn start_logger(mut config: LoggerConfig) {
    env_var_override(&mut config);
    let mut dispatch = basic_dispatch(config.level_filter);

    if config.use_stdout {
        dispatch = if config.use_colors {
            dispatch.chain(colored_stdout())
        } else {
            dispatch.chain(io::stdout())
        }
    }

    if let Some(path) = config.log_file {
        match fern::log_file(path) {
            Ok(log_file) => dispatch = dispatch.chain(log_file),
            Err(_) => eprintln!("Unable to access the log file, as such it will not be used"),
        }
    }

    dispatch.apply().unwrap_or_else(|_| {
        debug!("Global logger already set, default Amethyst logger will not be used")
    });
}

fn env_var_override(config: &mut LoggerConfig) {
    if env::var("AMETHYST_LOG_DISABLE_STDOUT").is_ok() {
        config.use_stdout = false;
    }
    if env::var("AMETHYST_LOG_DISABLE_COLORS").is_ok() {
        config.use_colors = false;
    }
    if let Ok(lf) = env::var("AMETHYST_LOG_LEVEL_FILTER") {
        config.level_filter = LevelFilter::from_str(&lf).unwrap_or(LevelFilter::Debug)
    }
    if let Ok(path) = env::var("AMETHYST_LOG_FILE_PATH") {
        config.log_file = Some(PathBuf::from(path));
    }
}

fn basic_dispatch(level_filter: LevelFilter) -> fern::Dispatch {
    fern::Dispatch::new()
        .level(level_filter)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{level}][{target}] {message}",
                level = record.level(),
                target = record.target(),
                message = message,
            ))
        })
}

fn colored_stdout() -> fern::Dispatch {
    let color_config = fern::colors::ColoredLevelConfig::new();

    fern::Dispatch::new()
        .chain(io::stdout())
        .format(move |out, message, record| {
            let color = color_config.get_color(&record.level());
            out.finish(format_args!(
                "{color}{message}{color_reset}",
                color = format!("\x1B[{}m", color.to_fg_str()),
                message = message,
                color_reset = "\x1B[0m",
            ))
        })
}
