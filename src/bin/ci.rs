//! A small script to run all our CI tests
//!
//! We can just invoke this script on any platform, cutting down duplication.

use std::process::exit;

fn run() -> Result<(), String> {
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error running ci script: {}", e);
        exit(1);
    }
}
