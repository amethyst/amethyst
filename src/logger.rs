pub use log::LevelFilter;

use log::debug;
use serde::{Deserialize, Serialize};

use std::{env, fmt, io, path::PathBuf, str::FromStr};

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Determines whether to log to the terminal or not.
    pub stdout: StdoutLog,
    /// Sets the overarching level filter for the logger.
    pub level_filter: LevelFilter,
    /// If set, enables logging to file at the given path.
    pub log_file: Option<PathBuf>,
    /// If set, allows the config values to be overriden via the corresponding environmental variables.
    pub allow_env_override: bool,
    /// Sets a different level for gfx_device_gl if Some
    pub log_gfx_device_level: Option<LevelFilter>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            stdout: StdoutLog::Colored,
            level_filter: LevelFilter::Info,
            log_file: None,
            allow_env_override: true,
            log_gfx_device_level: Some(LevelFilter::Warn),
        }
    }
}

/// Allows the creation of a custom logger with a set of custom configurations. If no custom
/// formatting or configuration is required [`start_logger`] can be used instead.
///
/// # Examples
/// ```
/// amethyst::Logger::from_config(Default::default())
///     .level_for("gfx_device_gl", amethyst::LogLevelFilter::Warn)
///     .level_for("gfx_glyph", amethyst::LogLevelFilter::Error)
///     .start();
///
/// amethyst::Logger::from_config_formatter(Default::default(), |out, message, record| {
///     out.finish(format_args!(
///         "[{level}][{target}] {message}",
///         level = record.level(),
///         target = record.target(),
///         message = message,
///     ))
/// }).start();
/// ```
#[allow(missing_debug_implementations)]
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
        Self { dispatch }
    }

    /// Create a new Logger with a passed in formatter callback
    fn new_formatter<F>(formatter: F) -> Self
    where
        F: Fn(fern::FormatCallback<'_>, &fmt::Arguments<'_>, &log::Record<'_>)
            + Sync
            + Send
            + 'static,
    {
        let dispatch = fern::Dispatch::new().format(formatter);
        Self { dispatch }
    }

    /// Create a new logger from [`LoggerConfig`] and the Logger it will be added to
    fn new_with_config(mut config: LoggerConfig, mut logger: Self) -> Self {
        if config.allow_env_override {
            env_var_override(&mut config);
        }

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

        if let Some(log_gfx_device_level) = config.log_gfx_device_level {
            logger.dispatch = logger
                .dispatch
                .level_for("gfx_device_gl", log_gfx_device_level);
        }

        if let Some(path) = config.log_file {
            if let Ok(log_file) = fern::log_file(path) {
                logger.dispatch = logger.dispatch.chain(log_file)
            } else {
                eprintln!("Unable to access the log file, as such it will not be used")
            }
        }

        logger
    }

    /// Create a new Logger from [`LoggerConfig`]
    pub fn from_config(config: LoggerConfig) -> Self {
        Logger::new_with_config(config, Logger::new())
    }

    /// Create a new Logger from [`LoggerConfig`] and a formatter
    pub fn from_config_formatter<F>(config: LoggerConfig, formatter: F) -> Self
    where
        F: Fn(fern::FormatCallback<'_>, &fmt::Arguments<'_>, &log::Record<'_>)
            + Sync
            + Send
            + 'static,
    {
        Logger::new_with_config(config, Logger::new_formatter(formatter))
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
/// Configuration of the logger can also be controlled via environment variables:
/// * `AMETHYST_LOG_STDOUT` - determines the output to the terminal
///     * "no" / "off" / "0" disables logging to stdout
///     * "plain" / "yes" / "1" enables logging to stdout
///     * "colored" / "2" enables logging and makes it colored
/// * `AMETHYST_LOG_LEVEL_FILTER` - sets the log level
///     * "off" disables all logging
///     * "error" enables only error logging
///     * "warn" only errors and warnings are emitted
///     * "info" only error, warning and info messages
///     * "debug" everything except trace
///     * "trace" everything
/// * `AMETHYST_LOG_FILE_PATH` - if set, enables logging to the file at the path
///     * the value is expected to be a path to the logging file
pub fn start_logger(config: LoggerConfig) {
    Logger::from_config(config).start();
}

fn env_var_override(config: &mut LoggerConfig) {
    if let Ok(var) = env::var("AMETHYST_LOG_STDOUT") {
        match var.to_lowercase().as_ref() {
            "off" | "no" | "0" => config.stdout = StdoutLog::Off,
            "plain" | "yes" | "1" => config.stdout = StdoutLog::Plain,
            "colored" | "2" => config.stdout = StdoutLog::Colored,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn check_stdout_override() {
        let mut config = LoggerConfig::default();
        assert_eq!(config.stdout, StdoutLog::Colored);

        env::set_var("AMETHYST_LOG_STDOUT", "pLaIn");
        env_var_override(&mut config);
        env::remove_var("AMETHYST_LOG_STDOUT");

        assert_eq!(config.stdout, StdoutLog::Plain);
    }
}
