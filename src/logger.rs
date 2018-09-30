pub use log::LevelFilter;

use fern;
use std::{env, io, path::PathBuf, str::FromStr};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Stdout {
    Off,
    Plain,
    Colored,
}

/// Logger configuration object.
#[derive(Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Determines whether to log to the terminal or not.
    pub stdout: Stdout,
    /// Sets the overarching level filter for the logger.
    pub level_filter: LevelFilter,
    /// If set, enables logging to file at the given path.
    pub log_file: Option<PathBuf>,
    /// If set, allows the config values to be overriden via the corresponding environmental variables.
    pub allow_env_override: bool,
}

impl Default for LoggerConfig {
    fn default() -> LoggerConfig {
        LoggerConfig {
            stdout: Stdout::Colored,
            level_filter: LevelFilter::Debug,
            log_file: None,
            allow_env_override: true,
        }
    }
}

/// Starts a basic logger outputting to stdout with color on supported platforms, and/or to file.
///
/// If you do not intend on using the logger builtin to Amethyst, it's highly recommended you
/// initialise your own.
///
/// Configuration of the logger can also be controlled via environment variables:
/// * AMETHYST_LOG_STDOUT - determines the output to the terminal
/// * AMETHYST_LOG_LEVEL_FILTER - sets the log level
/// * AMETHYST_LOG_FILE_PATH - if set, enables logging to the file at the path
pub fn start_logger(mut config: LoggerConfig) {
    if config.allow_env_override {
        env_var_override(&mut config);
    }

    let mut dispatch = basic_dispatch(config.level_filter);

    match config.stdout {
        Stdout::Plain => dispatch = dispatch.chain(io::stdout()),
        Stdout::Colored => dispatch = dispatch.chain(colored_stdout()),
        Stdout::Off => {}
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
    if let Ok(var) = env::var("AMETHYST_LOG_STDOUT") {
        match var.to_lowercase().as_ref() {
            "off" => config.stdout = Stdout::Off,
            "plain" => config.stdout = Stdout::Plain,
            "colored" => config.stdout = Stdout::Colored,
            _ => {}
        }
    }
    if let Ok(var) = env::var("AMETHYST_LOG_LEVEL_FILTER") {
        if let Ok(lf) = LevelFilter::from_str(&var) {
            config.level_filter = lf;
        }
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
