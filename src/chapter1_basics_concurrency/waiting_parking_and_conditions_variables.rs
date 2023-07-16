#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::{Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn thread_parking() {
        let queue = Mutex::new(VecDeque::from_iter(0..10));

        thread::scope(|s| {
            let t = s.spawn(|| loop {
                let last = queue.lock().unwrap().pop_back();
                if let Some(last) = last {
                    println!("{last:?}");
                } else {
                    thread::park();
                }
            });

            for i in 11..20 {
                queue.lock().unwrap().push_back(i);
                t.thread().unpark();
                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    #[test]
    fn condition_variables() {
        let queue = Mutex::new(VecDeque::from_iter(0..10));
        let is_empty = Condvar::new();

        thread::scope(|s| {
            s.spawn(|| loop {
                let mut guard = queue.lock().unwrap();
                let item = loop {
                    if let Some(item) = guard.pop_front() {
                        break item;
                    } else {
                        guard = is_empty.wait(guard).unwrap();
                    }
                };
                drop(guard);
                dbg!(item);
            });

            for i in 11..20 {
                queue.lock().unwrap().push_back(i);
                is_empty.notify_one();
            }
        });
    }
}
