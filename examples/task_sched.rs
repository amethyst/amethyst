extern crate threadpool;

use threadpool::ThreadPool;
use std::sync::mpsc::channel;

fn main() {
    let blah = ThreadPool::new(4);

    let (tx, rx) = channel();
    for i in 0..1000 {
        let tx = tx.clone();
        blah.execute(move || {
            tx.send(i + 1).unwrap();
        });
    }

    println!("Sum: {}", rx.iter().take(1000).fold(0, |a, b| a + b));
}
