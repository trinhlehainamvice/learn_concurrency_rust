use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;

fn some_work() {}

fn main() {
    let atomic_x = AtomicUsize::new(0);
    static mut NORMAL_X: usize = 0;
    thread::scope(|s| {
        for _ in 0..10000 {
            s.spawn(|| {
                thread::sleep(Duration::from_millis(10));
                atomic_x.fetch_add(1, Relaxed);
            });
        }
        for _ in 0..10000 {
            s.spawn(|| {
                thread::sleep(Duration::from_millis(10));
                unsafe {
                    NORMAL_X += 1;
                }
            });
        }
    });

    println!("Normal: {}", unsafe { NORMAL_X });
    println!("Atomic: {}", atomic_x.load(Relaxed));
    assert_eq!(atomic_x.load(Relaxed), 10000);
}
