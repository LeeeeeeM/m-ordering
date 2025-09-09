use std::{sync::atomic::{AtomicUsize, Ordering}, thread};

fn main() {
    let counter = AtomicUsize::new(0);
    thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| {
                for _ in 0..1000 {
                    thread::sleep(std::time::Duration::from_millis(2));
                    // let current = counter.load(Ordering::Relaxed);
                    // counter.store(current + 1, Ordering::Relaxed);
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
        loop {
            let n = counter.load(Ordering::Relaxed);
            println!("process: {} / 10000 done!", n);
            if n == 10000 {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(1000));
        }
    });
}
