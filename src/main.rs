use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let queue = Mutex::new(VecDeque::from_iter(0..10));
    let is_empty = Condvar::new();

    thread::scope(|s| {
        s.spawn(|| loop {
            let mut guard = queue.lock().unwrap();
            let item = loop {
                if let Some(item) = guard.pop_front() {
                    break item;
                } else {
                    // wait will unlock guard and block this thread until receive notification (other thread call notify_one or notify_all on is_empty)
                    guard = is_empty.wait(guard).unwrap();
                }
            };
            drop(guard);
            dbg!(item);
        });

        for i in 11..20 {
            queue.lock().unwrap().push_back(i);
            is_empty.notify_one();
            thread::sleep(Duration::from_secs(1));
        }
    });
}
