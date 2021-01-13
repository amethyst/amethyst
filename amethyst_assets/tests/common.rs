use std::{panic, path::PathBuf, sync::Once};

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
        .level(log::LevelFilter::Trace)
        .level_for("mio", log::LevelFilter::Error)
        .chain(std::io::stdout())
        .apply()
        .expect("Could not start logger");
}

static INIT: Once = Once::new();

pub(crate) fn run_test<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    INIT.call_once(|| {
        setup_logger();
        start_asset_daemon(vec![PathBuf::from("tests/assets")]);
    });

    let result = panic::catch_unwind(|| test());

    assert!(result.is_ok())
}
