use std::{sync::atomic::{AtomicUsize, Ordering}, thread};

fn main() {
    let counter = AtomicUsize::new(0);
    thread::scope(|s| {
        for _ in 0..1000 {
            s.spawn(|| {
                incr(&counter);
            });
        }
    });
    println!("counter: {}", counter.load(Ordering::Relaxed));
}

fn incr(counter: &AtomicUsize) {
    let mut current = counter.load(Ordering::Relaxed);
    loop {
        let new_val = current + 1;
        match counter.compare_exchange(current, new_val, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(x) => {
                println!("current: {}, new_val: {}, but get: {}", current, new_val, x);
                current = x;
            },
        }
    }
}