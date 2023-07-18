use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

// Allow SpinLock to be shared between threads
unsafe impl<T> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(val: T) -> SpinLock<T> {
        SpinLock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(val),
        }
    }

    pub fn lock(&mut self) -> &mut T {
        while !self.lock.swap(true, Acquire) {
            std::hint::spin_loop();
        }

        unsafe { &mut *self.data.get() }
    }

    pub unsafe fn unlock(&self) {
        self.lock.store(false, Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test() {
        fn generate_data() -> u32 {
            static mut DATA: u32 = 0;
            unsafe {
                DATA += 1;
                DATA
            }
        }

        static mut LOCK: SpinLock<i32> = SpinLock::new(0);

        for i in 0..10 {
            thread::spawn(move || unsafe {
                let data = LOCK.lock();
                *data += 1;
                LOCK.unlock();
            });
        }

        thread::sleep(Duration::from_millis(100));

        let data = unsafe { LOCK.lock() };
        assert_eq!(*data, 10);
        unsafe { LOCK.unlock() };
    }
}
