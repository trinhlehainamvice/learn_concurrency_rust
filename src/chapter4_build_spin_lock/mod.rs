mod safe_version;
mod unsafe_version;

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Release};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_minimal() {
        struct SpinLock {
            lock: AtomicBool,
        };

        impl SpinLock {
            pub const fn new() -> SpinLock {
                SpinLock {
                    lock: AtomicBool::new(false),
                }
            }

            pub fn lock(&self) {
                // If swap atomic operator return false, mean it is locking
                while !self.lock.swap(true, Acquire) {
                    std::hint::spin_loop();
                }
            }

            pub fn unlock(&self) {
                self.lock.store(false, Release);
            }
        }

        fn generate_data() -> u32 {
            static mut DATA: u32 = 0;
            unsafe {
                DATA += 1;
                DATA
            }
        }

        static LOCK: SpinLock = SpinLock::new();
        static mut DATA: [u32; 10] = [0; 10];

        for i in 0..10 {
            thread::spawn(move || {
                LOCK.lock();
                unsafe { DATA[i] = generate_data() };
                LOCK.unlock();
            });
        }

        thread::sleep(Duration::from_millis(100));

        for i in unsafe { &DATA } {
            println!("DATA: {}", *i);
        }
    }
}
