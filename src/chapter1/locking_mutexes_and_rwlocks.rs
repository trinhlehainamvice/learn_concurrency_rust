#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    #[test]
    fn test() {
        let m = Mutex::new(0);
        std::thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = m.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }

                    // Another thread need to wait this thread wake up after sleep for 1 second
                    // So if we unlock or drop guard before we put this thread to sleep
                    // Another thread can take mutex
                    drop(guard);
                    std::thread::sleep(Duration::from_secs(1));
                }); // Guard is dropped here
            }
        });

        assert_eq!(m.into_inner().unwrap(), 1000);
    }
}
