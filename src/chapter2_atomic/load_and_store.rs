#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_atomic_bool() {
        fn some_work() {
            println!("some_work");
        }

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

    #[test]
    fn test_stop_with_park() {
        let num = AtomicUsize::new(0);
        let main_thread = thread::current();

        thread::scope(|s| {
            s.spawn(|| {
                for i in 0..=100 {
                    num.store(i, Relaxed);
                }
                println!("Working Thread: {:?}", thread::current().id());
                main_thread.unpark();
            });

            loop {
                let num = num.load(Relaxed);
                if num == 100 {
                    break;
                }
                println!("Loading {num}/100");
                println!("Waiting Thread: {:?}", thread::current().id());
                thread::park_timeout(Duration::from_secs(1));
            }
        });

        assert_eq!(num.load(Relaxed), 100);
        println!("Main Thread: {:?}", main_thread.id());
        println!("Finished");
    }

    #[test]
    fn test_race() {
        fn lazy_initialize() -> usize {
            2
        }

        static X: AtomicUsize = AtomicUsize::new(0);
        if X.load(Relaxed) == 0 {
            let x = lazy_initialize();
            X.store(x, Relaxed);
        }
    }
}
