use std::env;
use std::fs;

// Returns the cargo manifest directory for debug builds or the directory in which the current
// executable resides for release builds, traversing a symlink if necessary.
pub fn application_root_dir() -> String {
    if env::var("PROFILE").unwrap() == "release" {
        let path = env::current_exe().expect("Failed to find executable path.");
        String::from(
            match fs::read_link(path) {
                Ok(target) => target.parent(),
                Err(_) => path.parent(), 
            }.expect("Failed to get parent directory of the executable.").to_str().unwrap()
        )
    } else {
        env!("CARGO_MANIFEST_DIR")
    }
}