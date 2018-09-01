use std::env;
use std::fs;

// Returns the cargo manifest directory when running the executable with cargo
// or the directory in which the executable resides otherwise,
// traversing symlinks if necessary.
pub fn application_root_dir() -> String {
    env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        let mut path = env::current_exe().expect("Failed to find executable path.");
        while let Ok(target) = fs::read_link(path.clone()) {
            path = target;
        }
        String::from(
            path.parent()
                .expect("Failed to get parent directory of the executable.")
                .to_str()
                .unwrap(),
        )
    })
}
