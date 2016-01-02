extern crate threadpool;

use threadpool::ThreadPool;

fn main() {
    let blah = ThreadPool::new(4);
    for i in 0..10000000 {
        blah.execute(move || {
            println!("Hello from parallel thread {}!", i);
        });
    }

    // for i in 0..10000000 {
    //     println!("Hello from linear thread {}!", i);
    // }
}
