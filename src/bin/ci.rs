//! A small script to run all our CI tests
//!
//! We can just invoke this script on any platform, cutting down duplication.

use std::process::{Command, exit};
use std::fmt::Write;

fn run_test_command<P, A, S>(prog: P, args: A) -> Result<(), String>
where P: AsRef<str>,
      A: AsRef<[S]>,
      S: AsRef<str>
{
    let mut cmd = Command::new(prog.as_ref());
    let args = args.as_ref().iter().map(|a| a.as_ref());
    cmd.args(args);
    println!("Running command {:?}", cmd);
    let output = cmd.output().map_err(|e| format!("io error: {}", e))?;
    if output.status.success() {
        Ok(())
    } else {
        let mut err_msg = String::new();
        writeln!(err_msg, "--- Error ---").unwrap();
        writeln!(err_msg,
                 "The command {:?} exited with error code {:?}",
                 cmd,
                 output.status.code()).unwrap();
        writeln!(err_msg, "--- Stdout ---").unwrap();
        writeln!(err_msg, "{}", String::from_utf8_lossy(&output.stdout)).unwrap();
        writeln!(err_msg, "--- Stderr ---").unwrap();
        writeln!(err_msg, "{}", String::from_utf8_lossy(&output.stderr)).unwrap();
        Err(err_msg)
    }

}

fn run() -> Result<(), String> {
    run_test_command("cargo", &["test", "--all"])?;
    run_test_command("cargo", ["test", "--features", "profiler"])?;
    for package in [
        "amethyst_animation",
        "amethyst_assets",
        "amethyst_audio",
        "amethyst_config",
        "amethyst_controls",
        "amethyst_core",
        "amethyst_gltf",
        "amethyst_input",
        "amethyst_renderer",
        "amethyst_ui",
        "amethyst_utils",
    ].iter() {
        let args = vec!["test", "--features", "profiler", "--package", package.to_owned()];
        run_test_command("cargo", args)?;
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error running ci script: {}", e);
        exit(1);
    }
}
