use std::{
    fs::{create_dir_all, read_to_string, remove_file},
    path::Path,
};

use dirs_next::config_dir;
use vergen::{self, ConstantsFlags};

fn main() {
    let amethyst_home = config_dir().map(|p| p.as_path().join("amethyst"));

    if let Some(amethyst_home) = amethyst_home {
        if amethyst_home.exists() {
            if let Some(true) = check_sentry_allowed(&amethyst_home) {
                load_sentry_dsn();
            };
        } else {
            create_dir_all(&amethyst_home).expect("Failed to create amethyst home directory.");
        };
    }

    vergen::generate_cargo_keys(ConstantsFlags::all())
        .unwrap_or_else(|e| panic!("Vergen crate failed to generate version information! {}", e));

    println!("cargo:rerun-if-changed=build.rs");
}

fn check_sentry_allowed(amethyst_home: &Path) -> Option<bool> {
    let sentry_status_file = amethyst_home.join("sentry_status.txt");
    if sentry_status_file.exists() {
        match read_to_string(&sentry_status_file) {
            Ok(result) => {
                match result.as_str().trim() {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => {
                        remove_file(sentry_status_file)
                            .expect("Failed to remove invalid sentry file.");
                        None
                    }
                }
            }
            Err(_) => None,
        }
    } else {
        None
    }
}

fn load_sentry_dsn() {
    let sentry_dsn = include_str!(".sentry_dsn.txt");
    println!("cargo:rustc-env=SENTRY_DSN={}", sentry_dsn);
}
