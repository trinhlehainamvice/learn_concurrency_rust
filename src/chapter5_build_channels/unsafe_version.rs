use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

struct Channel<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool
}

unsafe impl<T> Sync for Channel<T> {}

impl<T> Channel<T> {
    pub const fn new() -> Channel<T> {
        Channel {
            msg: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
        }
    }
    
    pub unsafe fn send(&self, msg: T) {
        (*self.msg.get()).write(msg);
        self.ready.store(true, Release);
    }
    
    pub fn is_ready(&self) -> bool {
        self.ready.load(Acquire)
    }
    
    pub unsafe fn receive(&self) -> T {
        (*self.msg.get()).assume_init_read()
    }
}