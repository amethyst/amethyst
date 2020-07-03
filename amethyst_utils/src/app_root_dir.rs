//! Provides the directory of the executable.

use std::{env, io, path};

/// Returns the cargo manifest directory when running the executable with cargo or the directory in
/// which the executable resides otherwise, traversing symlinks if necessary.
///
/// The algorithm used is:
///
/// * If the `CARGO_MANIFEST_DIR` environment variable is defined it is used as application root.
///   This simplifies running development projects through `cargo run`.
///   See the [cargo reference documentation][cargo-ref] for more details.
/// * If the executable name can be found using [`std::env::current_exe`], resolve all symlinks and
///   use the directory it resides in as application root.
///
/// If none of the above works, an error is returned.
///
/// [cargo-ref]: https://doc.rust-lang.org/cargo/reference/environment-variables.html
/// [`std::env::current_exe`]: https://doc.rust-lang.org/std/env/fn.current_exe.html
pub fn application_root_dir() -> Result<path::PathBuf, io::Error> {
    if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
        return Ok(path::PathBuf::from(manifest_dir));
    }

    let mut exe = dunce::canonicalize(env::current_exe()?)?;

    // Modify in-place to avoid an extra copy.
    if exe.pop() {
        return Ok(exe);
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to find an application root",
    ))
}

/// Same as `application_root_dir`, but extends the root directory with the given path.
pub fn application_dir<P>(path: P) -> Result<path::PathBuf, io::Error>
where
    P: AsRef<path::Path>,
{
    Ok(application_root_dir()?.join(path))
}
