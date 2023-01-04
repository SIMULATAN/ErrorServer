use std::sync::Arc;
use std::thread;

use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;

use crate::threads::ThreadPool;

pub fn setup(thread_pool: Arc<ThreadPool>) {
    let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();

    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
            thread_pool.print_stats();
            std::process::exit(0);
        }
    });
}
