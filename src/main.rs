use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;

fn some_work() {}

fn main() {
    static STOP: AtomicBool = AtomicBool::new(false);

    let bg_thread = thread::spawn(|| {
        while !STOP.load(Relaxed) {
            some_work();
        }
    });

    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("command: help"),
            "stop" => break,
            cmd => println!("unknown command: {}", cmd),
        }
    }

    STOP.store(true, Relaxed);

    bg_thread.join().unwrap();
}
