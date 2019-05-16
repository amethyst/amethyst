use dirs::home_dir;
use std::{
    fs::{create_dir_all, read_to_string, remove_file, File},
    io::{stderr, stdin, Write},
    path::Path,
};
use vergen::{self, ConstantsFlags};

fn main() {
    let amethyst_home =
        Path::new(&home_dir().expect("Failed to find home directory")).join(".amethyst");
    match amethyst_home.exists() {
        true => match check_sentry_allowed(&amethyst_home) {
            Some(true) => {
                load_sentry_dsn();
            }
            None => {
                ask_write_user_data_collection(&amethyst_home);
            }
            _ => {}
        },
        false => {
            create_dir_all(&amethyst_home).expect("Failed to create amethyst home directory.");
            if ask_write_user_data_collection(&amethyst_home) {
                load_sentry_dsn();
            }
        }
    };

    vergen::generate_cargo_keys(ConstantsFlags::all())
        .unwrap_or_else(|e| panic!("Vergen crate failed to generate version information! {}", e));

    println!("cargo:rerun-if-changed=build.rs");
}

fn check_sentry_allowed(amethyst_home: &Path) -> Option<bool> {
    let sentry_status_file = amethyst_home.join(".sentry_status.txt");
    if sentry_status_file.exists() {
        match read_to_string(&sentry_status_file) {
            Ok(result) => match result.as_str().trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => {
                    remove_file(sentry_status_file).expect("Failed to remove invalid sentry file.");
                    None
                }
            },
            Err(_) => None,
        }
    } else {
        None
    }
}

fn ask_user_data_collection() -> bool {
    eprint!("May we collect anonymous panic data and usage statistics to help improve Amethyst? No personal information is collected or stored. [Y/n]: ");
    stderr().flush().expect("Failed to flush stdout");

    let mut s = String::new();
    stdin()
        .read_line(&mut s)
        .expect("There was an error getting your input");
    s = s.trim().to_lowercase();
    match s.chars().next() {
        Some('n') => false,
        _ => false, // Can't read stdin from the build.rs file. Let's default to false. Set this line to true when a solution is found.
    }
}

fn ask_write_user_data_collection(amethyst_home: &Path) -> bool {
    let mut file = File::create(amethyst_home.join(".sentry_status.txt"))
        .expect("Error writing Sentry status file");
    match ask_user_data_collection() {
        true => {
            let _ = file.write_all(b"true");
            true
        }
        false => {
            let _ = file.write_all(b"false");
            false
        }
    }
}

fn load_sentry_dsn() {
    let sentry_dsn = include_str!(".sentry_dsn.txt");
    println!("cargo:rustc-env=SENTRY_DSN={}", sentry_dsn);
}
