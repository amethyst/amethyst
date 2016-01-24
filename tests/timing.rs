// Tests for amethyst_engine::timing

extern crate amethyst_engine;

use amethyst_engine::Stopwatch;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn stopwatch_elapsed() {
    let mut watch = Stopwatch::new();

    watch.start();
    sleep(Duration::from_secs(2));
    watch.stop();

    assert_eq!(2, watch.elapsed().num_seconds());
}

#[test]
fn stopwatch_reset() {
    let mut watch = Stopwatch::new();

    watch.start();
    sleep(Duration::from_secs(2));
    watch.stop();
    watch.reset();

    assert_eq!(0, watch.elapsed().num_nanoseconds().unwrap());
}

#[test]
fn stopwatch_restart() {
    let mut watch = Stopwatch::new();

    watch.start();
    sleep(Duration::from_secs(2));
    watch.stop();

    watch.restart();
    sleep(Duration::from_secs(1));
    watch.stop();

    assert_eq!(1, watch.elapsed().num_seconds());
}
