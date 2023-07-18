#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering::Relaxed;
    use std::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, AtomicUsize};
    use std::thread;
    use std::time::Duration;

    fn process_some_work(t: &i32) {
        thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test() {
        let num = &AtomicUsize::new(0);
        thread::scope(|s| {
            for t in 0..4 {
                s.spawn(move || {
                    for _ in 0..25 {
                        // - Because t may live shorter than this thread
                        // So capture t as a reference &i32 is not allowed
                        // Solve this by use move capture to copy t inside this thread
                        // - Move capture will move AtomicUsize into this thread
                        // So another thread can't access this AtomicUsize
                        // Solve this by reference to AtomicUsize [&AtomicUsize]
                        // So the reference will be moved (copied in this case) into this thread
                        process_some_work(&t);
                        num.fetch_add(1, Relaxed);
                    }
                });
            }

            loop {
                if num.load(Relaxed) == 100 {
                    break;
                }
            }
        });

        assert_eq!(num.load(Relaxed), 100);
        println!("Done");
    }

    #[test]
    fn test_statistics() {
        let num = &AtomicUsize::new(0);
        let total_time = &AtomicU64::new(0);
        let max_time = &AtomicU64::new(0);

        thread::scope(|s| {
            for t in 0..4 {
                s.spawn(move || {
                    for _ in 0..25 {
                        let start = std::time::Instant::now();
                        process_some_work(&t);
                        num.fetch_add(1, Relaxed);
                        let end = start.elapsed().as_micros() as u64;
                        total_time.fetch_add(end, Relaxed);
                        max_time.fetch_add(end, Relaxed);
                    }
                });
            }

            loop {
                let num = num.load(Relaxed);
                if num == 100 {
                    break;
                }
                if num == 0 {
                    println!("Process haven't started yet!");
                } else {
                    println!("Processing ... {num}/100");
                }
                thread::sleep(Duration::from_secs(1));
            }
        });

        println!("Total time: {:?}", total_time.load(Relaxed));
        println!("Max time: {:?}", max_time.load(Relaxed));
        assert_eq!(num.load(Relaxed), 100);
        println!("Done");
    }

    #[test]
    fn test_allocate_new_id() {
        fn allocate_new_id() -> Option<u32> {
            static ID: AtomicU32 = AtomicU32::new(0);
            let id = ID.fetch_add(1, Relaxed);
            const LIMIT: u32 = 10;
            if id >= LIMIT {
                ID.fetch_sub(1, Relaxed);
                return None;
            }
            Some(id)
        }

        let mut threads = vec![];
        for num in 0..20 {
            let thread = thread::spawn(move || {
                let key = allocate_new_id();
                if let Some(key) = key {
                    println!("Valid Key: {key}");
                } else {
                    println!("Invalid Key in thread {num}th");
                }
            });
            threads.push(thread);
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_generate_key_with_fetch() {
        fn allocate_new_id() -> Option<u32> {
            static ID: AtomicU32 = AtomicU32::new(0);
            let id = ID.load(Relaxed);
            const LIMIT: u32 = 10;
            // (id + 1) might cause overflow
            if (id + 1) > LIMIT {
                println!("Loaded ID: {id}");
                return None;
            }

            // According to Bard, we haven't checked value af atomic id after fetch_add
            // So there is still a case if multiple thread load id that not overflow yet
            // But cause overflow when numbers of threads add up to be overflow when fetch_add to current atomic id at the same time
            Some(ID.fetch_add(1, Relaxed))
        }

        let mut threads = vec![];
        for num in 0..20 {
            let thread = thread::spawn(move || {
                let key = allocate_new_id();
                if let Some(key) = key {
                    println!("Valid Key: {key}");
                } else {
                    println!("Invalid Key in thread {num}th");
                }
            });
            threads.push(thread);
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn test_compare_exchange() {
        fn increment(num: &AtomicI32) {
            let mut current = num.load(Relaxed);
            loop {
                let new = current + 1;
                match num.compare_exchange(current, new, Relaxed, Relaxed) {
                    Ok(_) => break,
                    // If value is already changed by another thread, update current to that value
                    Err(value) => current = value,
                }
            }
        }

        fn allocate_new_id() -> Option<u32> {
            static ID: AtomicU32 = AtomicU32::new(0);
            let mut id = ID.load(Relaxed);
            const LIMIT: u32 = 10;
            loop {
                if id >= LIMIT {
                    println!("Loaded ID: {id}");
                    return None;
                }
                match ID.compare_exchange(id, id + 1, Relaxed, Relaxed) {
                    Ok(_) => return Some(id),
                    Err(value) => id = value,
                }
            }
        }

        fn generate_unique_key() -> u32 {
            1
        }

        fn get_key() -> u32 {
            static KEY: AtomicU32 = AtomicU32::new(0);
            let key = KEY.load(Relaxed);
            if key == 0 {
                // if the time this thread load key is still not generated
                // generate new key and expect that atomic key hasn't changed
                match KEY.compare_exchange(0, generate_unique_key(), Relaxed, Relaxed) {
                    Ok(new_key) => new_key,
                    // if key is already generated by another thread, use that key
                    Err(generated_key) => generated_key,
                }
            } else {
                key
            }
        }

        let mut threads = vec![];
        for num in 0..20 {
            let thread = thread::spawn(move || {
                let key = allocate_new_id();
                if let Some(key) = key {
                    println!("Valid Key: {key}");
                } else {
                    println!("Invalid Key in thread {num}th");
                }
            });
            threads.push(thread);
        }

        for thread in threads {
            thread.join().unwrap();
        }
    }
}
