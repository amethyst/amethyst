pub use log::LevelFilter;

use log::debug;
use serde::{Deserialize, Serialize};

use std::{env, io, path::PathBuf, str::FromStr};

/// An enum that contains options for logging to the terminal.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StdoutLog {
    /// Disables logging to the terminal.
    Off,
    /// Enables logging to the terminal without colored output.
    Plain,
    /// Enables logging to the terminal with colored output on supported platforms.
    Colored,
}

/// Logger configuration object.
#[derive(Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Determines whether to log to the terminal or not.
    pub stdout: StdoutLog,
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
            stdout: StdoutLog::Colored,
            level_filter: LevelFilter::Debug,
            log_file: None,
            allow_env_override: true,
        }
    }
}

/// Allows the creation of a logger with a set of custom configurations. If no custom configuration
/// is required [`start_logger`] can be used instead.
///
/// # Examples
/// ```
/// amethyst::Logger::from_config(Default::default())
///     .level_for("gfx_device_gl", amethyst::LogLevelFilter::Warn)
///     .level_for("gfx_glyph", amethyst::LogLevelFilter::Error)
///     .start();
/// ```
pub struct Logger {
    dispatch: fern::Dispatch,
}

impl Logger {
    fn new() -> Self {
        let dispatch = fern::Dispatch::new().format(|out, message, record| {
            out.finish(format_args!(
                "[{level}][{target}] {message}",
                level = record.level(),
                target = record.target(),
                message = message,
            ))
        });
        Logger { dispatch }
    }

    /// Create a new Logger from [`LoggerConfig`]
    pub fn from_config(mut config: LoggerConfig) -> Self {
        if config.allow_env_override {
            env_var_override(&mut config);
        }

        let mut logger = Logger::new();
        logger.dispatch = logger.dispatch.level(config.level_filter);

        match config.stdout {
            StdoutLog::Plain => logger.dispatch = logger.dispatch.chain(io::stdout()),
            StdoutLog::Colored => {
                logger.dispatch = logger
                    .dispatch
                    .chain(colored_stdout(fern::colors::ColoredLevelConfig::new()))
            }
            StdoutLog::Off => {}
        }

        if let Some(path) = config.log_file {
            match fern::log_file(path) {
                Ok(log_file) => logger.dispatch = logger.dispatch.chain(log_file),
                Err(_) => eprintln!("Unable to access the log file, as such it will not be used"),
            }
        }

        logger
    }

    /// Set individual log levels for modules.
    pub fn level_for<T: Into<std::borrow::Cow<'static, str>>>(
        mut self,
        module: T,
        level: LevelFilter,
    ) -> Self {
        self.dispatch = self.dispatch.level_for(module, level);
        self
    }

    /// Starts [`Logger`] by consuming it.
    pub fn start(self) {
        self.dispatch.apply().unwrap_or_else(|_| {
            debug!("Global logger already set, default Amethyst logger will not be used")
        });
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
pub fn start_logger(config: LoggerConfig) {
    Logger::from_config(config).start();
}

fn env_var_override(config: &mut LoggerConfig) {
    if let Ok(var) = env::var("AMETHYST_LOG_STDOUT") {
        match var.to_lowercase().as_ref() {
            "off" => config.stdout = StdoutLog::Off,
            "plain" => config.stdout = StdoutLog::Plain,
            "colored" => config.stdout = StdoutLog::Colored,
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

fn colored_stdout(color_config: fern::colors::ColoredLevelConfig) -> fern::Dispatch {
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
