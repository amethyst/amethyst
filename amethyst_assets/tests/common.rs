use std::{path::PathBuf, sync::Once};

use amethyst_assets::start_asset_daemon;

pub fn setup_logger() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()
        .expect("Could not start logger");
}

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        setup_logger();
        start_asset_daemon(vec![PathBuf::from("tests/assets")]);
    });
}
