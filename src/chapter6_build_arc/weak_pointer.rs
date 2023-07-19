use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::{fence, AtomicUsize};

struct ArcInner<T> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    value: UnsafeCell<ManuallyDrop<T>>,
}

impl<T> ArcInner<T> {
    fn new(value: T) -> ArcInner<T> {
        ArcInner {
            strong: AtomicUsize::new(1),
            weak: AtomicUsize::new(1),
            value: UnsafeCell::new(ManuallyDrop::new(value)),
        }
    }
}

struct Arc<T> {
    ptr: NonNull<ArcInner<T>>,
}

unsafe impl<T: Send + Sync> Sync for Arc<T> {}
unsafe impl<T: Send + Sync> Send for Arc<T> {}

impl<T> Arc<T> {
    pub fn new(value: T) -> Arc<T> {
        Arc {
            ptr: NonNull::from(Box::leak(Box::new(ArcInner::new(value)))),
        }
    }

    pub fn downgrade(this: &Self) -> Weak<T> {
        let mut weak_count = unsafe { this.ptr.as_ref().weak.load(Relaxed) };
        loop {
            // Check overflow
            // If it's overflowed, then wait until available
            if weak_count == usize::MAX {
                std::hint::spin_loop();
                weak_count = unsafe { this.ptr.as_ref().weak.load(Relaxed) };
                continue;
            }

            // Check if count is already taken
            if let Err(last_weak_count) = unsafe {
                this.ptr.as_ref().weak.compare_exchange_weak(
                    weak_count,
                    weak_count + 1,
                    Acquire,
                    Relaxed,
                )
            } {
                weak_count = last_weak_count;
                continue;
            }

            return Weak { ptr: this.ptr };
        }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr.as_ref().value.get() }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Arc<T> {
        unsafe {
            self.ptr.as_ref().strong.fetch_add(1, Relaxed);
        }
        Arc { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        unsafe {
            if self.ptr.as_ref().strong.fetch_sub(1, Release) != 1 {
                return;
            }
            fence(Acquire);
            // Drop value and call destructor (aka drop)
            ManuallyDrop::drop(&mut *self.ptr.as_ref().value.get());
            drop(Weak { ptr: self.ptr });
        }
    }
}

struct Weak<T> {
    ptr: NonNull<ArcInner<T>>,
}

unsafe impl<T: Send + Sync> Sync for Weak<T> {}
unsafe impl<T: Send + Sync> Send for Weak<T> {}

impl<T> Weak<T> {
    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut strong_count = unsafe { self.ptr.as_ref().strong.load(Relaxed) };
        loop {
            if strong_count == 0 {
                return None;
            }

            // Check overflow
            // If it's overflowed, then wait until available
            if strong_count == usize::MAX {
                std::hint::spin_loop();
                strong_count = unsafe { self.ptr.as_ref().strong.load(Relaxed) };
                continue;
            }

            // Check if count is already taken
            if let Err(last_strong_count) = unsafe {
                self.ptr.as_ref().strong.compare_exchange_weak(
                    strong_count,
                    strong_count + 1,
                    Relaxed,
                    Relaxed,
                )
            } {
                strong_count = last_strong_count;
                continue;
            }

            return Some(Arc { ptr: self.ptr });
        }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        unsafe {
            if self.ptr.as_ref().weak.fetch_sub(1, Release) == 1 {
                fence(Acquire);
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chapter6_build_arc::weak_pointer::Arc;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering::Relaxed;

    #[test]
    fn test_std_sync_weak() {
        use std::sync::Arc;

        let a = Arc::new(3);
        let b = Arc::downgrade(&a);
        let c = b.as_ptr();

        use std::rc::{Rc, Weak};
        let a = Rc::new(3);
        let b = Rc::downgrade(&a);
        let c = b.upgrade();
    }

    #[test]
    fn test() {
        static NUM_DROPS: AtomicUsize = AtomicUsize::new(0);
        struct DetectDrop;
        impl Drop for DetectDrop {
            fn drop(&mut self) {
                NUM_DROPS.fetch_add(1, Relaxed);
            }
        }
        // Create an Arc with two weak pointers.
        let x = Arc::new(("hello", DetectDrop));
        let y = Arc::downgrade(&x);
        let z = Arc::downgrade(&x);
        let t = std::thread::spawn(move || {
            // Weak pointer should be upgradable at this point.
            let y = y.upgrade().unwrap();
            assert_eq!(y.0, "hello");
        });
        assert_eq!(x.0, "hello");
        t.join().unwrap();
        // The data shouldn't be dropped yet,
        // and the weak pointer should be upgradable.
        assert_eq!(NUM_DROPS.load(Relaxed), 0);
        assert!(z.upgrade().is_some());
        drop(x);
        // Now, the data should be dropped, and the
        // weak pointer should no longer be upgradable.
        assert_eq!(NUM_DROPS.load(Relaxed), 1);
        assert!(z.upgrade().is_none());
    }
}
