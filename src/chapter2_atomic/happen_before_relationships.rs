use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

static X: AtomicU32 = AtomicU32::new(0);

fn f() {
    let x = X.load(Relaxed);
    // Thread is spawned after X is stored as 1
    // And Thread is joined before X is stored as 3
    // So the value of x loaded from X can only be 1 or 2
    assert!(x == 1 || x == 2);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test() {
        X.store(1, Relaxed);
        // Thread is spawned after X is stored as 1
        let t = thread::spawn(f);
        X.store(2, Relaxed);
        t.join().unwrap();
        // Thread is joined before X is stored as 3
        X.store(3, Relaxed);
    }
}
