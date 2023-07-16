#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, RwLock};
    use std::time::Duration;

    #[test]
    fn test() {
        let m = Mutex::new(0);
        std::thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard: MutexGuard<i32> = m.lock().unwrap();
                    let deref_mut = &mut *guard;
                    for _ in 0..100 {
                        *deref_mut += 1;
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

    #[test]
    fn test_when_mutex_guard_dropped() {
        let m_nums = Mutex::new(vec![1, 2, 3]);
        std::thread::scope(|s| {
            s.spawn(|| {
                if let Some(num) = m_nums.lock().unwrap().pop() {
                    println!("{num:?}");
                }
            });
            s.spawn(|| {
                if let Some(num) = m_nums.lock().unwrap().first_mut() {
                    *num += 10;
                }
            });
        });

        assert_eq!(m_nums.into_inner().unwrap(), vec![11, 2]);
    }

    #[test]
    fn test_rwlock() {
        let nums = RwLock::new(vec![1, 2, 3]);
        std::thread::scope(|s| {
            s.spawn(|| {
                if let Some(num) = nums.write().unwrap().first_mut() {
                    *num += 10;
                }
            });
            s.spawn(|| {
                let read_guard = nums.read().unwrap();
                // let write_guard = nums.write().unwrap(); // Deadlock here when attempt to read and write on the same thread
                println!("{read_guard:?}");
            });
            s.spawn(|| {
                let read_guard = nums.read().unwrap();
                println!("{read_guard:?}");
            });
            s.spawn(|| {
                let last = nums.write().unwrap().pop();
                if let Some(num) = last {
                    println!("{num:?}");
                }
            });
        });

        assert_eq!(nums.into_inner().unwrap(), vec![11, 2]);
    }
}
