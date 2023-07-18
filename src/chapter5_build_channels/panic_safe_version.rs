use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering::{Relaxed, Release};

const EMPTY: u8 = 0;
const READY: u8 = 1;
const READING: u8 = 2;
const WRITING: u8 = 3;

struct Channcel<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    state: AtomicU8,
}

unsafe impl<T> Sync for Channcel<T> {}

impl<T> Channcel<T> {
    pub const fn new() -> Channcel<T> {
        Channcel {
            msg: UnsafeCell::new(MaybeUninit::uninit()),
            state: AtomicU8::new(EMPTY),
        }
    }

    pub unsafe fn send(&self, msg: T) {
        if self
            .state
            .compare_exchange(EMPTY, WRITING, Relaxed, Relaxed)
            .is_err()
        {
            panic!("Message has already been sent.");
        }
        (*self.msg.get()).write(msg);
        self.state.store(READY, Release);
    }

    pub fn is_ready(&self) -> bool {
        self.state.load(Relaxed) == READY
    }

    pub unsafe fn receive(&self) -> T {
        if self
            .state
            .compare_exchange(READY, READING, Relaxed, Relaxed)
            .is_err()
        {
            panic!("Message has already been received.");
        }
        let msg = (*self.msg.get()).assume_init_read();
        self.state.store(EMPTY, Release);
        msg
    }
}

impl<T> Drop for Channcel<T> {
    fn drop(&mut self) {
        if self.state.load(Relaxed) == READY {
            unsafe {
                (*self.msg.get()).assume_init_drop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chapter5_build_channels::panic_safe_version::Channcel;
    use std::thread;

    #[test]
    fn test() {
        let channel = Channcel::new();
        let t = thread::current();
        thread::scope(|s| {
            s.spawn(|| {
                unsafe {
                    channel.send("hello world");
                }
                t.unpark();
            });
            while !channel.is_ready() {
                thread::park();
            }
        });

        assert_eq!(unsafe { channel.receive() }, "hello world");
    }
}
