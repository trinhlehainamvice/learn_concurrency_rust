use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
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

    pub fn lock(&self) -> Guard<T> {
        while !self.lock.swap(true, Acquire) {
            std::hint::spin_loop();
        }

        Guard { inner: self }
    }
}

struct Guard<'a, T> {
    inner: &'a SpinLock<T>,
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.inner.data.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner.data.get() }
    }
}

// Guard prevent developer to explicitly unlock the SpinLock
// But auto unlock when Drop
impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.inner.lock.store(false, Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test() {
        let nums = SpinLock::new(Vec::new());
        thread::scope(|s| {
            s.spawn(|| nums.lock().push(1));
            s.spawn(|| {
                let mut guard = nums.lock();
                guard.push(2);
                guard.push(2);
            });
        });

        let guard = nums.lock();
        assert!(*guard == [1, 2, 2] || *guard == [2, 2, 1]);
    }
}
