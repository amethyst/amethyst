use std::io;

use fern;
use log::LevelFilter;

/// Starts a basic logger outputting to stdout with color on supported platforms.
///
/// If you do not intend on using the logger builtin to Amethyst, it's highly recommended you
/// initialise your own.
pub fn start_logger() {
    let color_config = fern::colors::ColoredLevelConfig::new();

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color}[{level}][{target}] {message}{color_reset}",
                color = format_args!(
                    "\x1B[{}m",
                    color_config.get_color(&record.level()).to_fg_str()
                ),
                level = record.level(),
                target = record.target(),
                message = message,
                color_reset = "\x1B[0m"
            ))
        })
        .level(LevelFilter::Debug)
        .chain(io::stdout())
        .apply()
        .unwrap_or_else(|_| {
            debug!("Global logger already set, default Amethyst logger will not be used")
        });
}