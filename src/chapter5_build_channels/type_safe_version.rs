use std::cell::{RefCell, UnsafeCell};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::thread;
use std::thread::Thread;

struct Channel<T> {
    msg: UnsafeCell<MaybeUninit<T>>,
    ready: AtomicBool,
    thread: RefCell<Option<Thread>>,
}

unsafe impl<T> Sync for Channel<T> {}

impl<T> Channel<T> {
    pub const fn new() -> Channel<T> {
        Channel {
            msg: UnsafeCell::new(MaybeUninit::uninit()),
            ready: AtomicBool::new(false),
            thread: RefCell::new(None),
        }
    }

    pub fn as_sender(&self) -> Sender<T> {
        Sender { inner: self }
    }

    pub fn as_receiver(&self) -> Receiver<T> {
        *self.thread.borrow_mut() = Some(thread::current());
        Receiver {
            inner: self,
            _no_send: PhantomData,
        }
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if self.ready.load(Relaxed) {
            unsafe {
                (*self.msg.get()).assume_init_drop();
            }
        }
    }
}

struct Sender<'a, T> {
    inner: &'a Channel<T>,
}

unsafe impl<T> Sync for Sender<'_, T> {}

impl<T> Sender<'_, T> {
    pub fn send(&self, msg: T) {
        unsafe {
            (*self.inner.msg.get()).write(msg);
        }
        if let Some(t) = self.inner.thread.borrow_mut().take() {
            t.unpark();
        }
        self.inner.ready.store(true, Release);
    }
}

struct Receiver<'a, T> {
    inner: &'a Channel<T>,
    // Prevent to be Send to other threads
    // Negative trait (!Send) is not fully implemented, Rust compiler suggests to use marker aka PhantomData
    _no_send: PhantomData<*const ()>,
}

impl<T> Receiver<'_, T> {
    pub fn receive(&self) -> Option<T> {
        if !self.inner.ready.swap(false, Acquire) {
            return None;
        }
        unsafe { Some((*self.inner.msg.get()).assume_init_read()) }
    }

    pub fn is_ready(&self) -> bool {
        self.inner.ready.load(Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use crate::chapter5_build_channels::type_safe_version::Channel;
    use std::thread;

    #[test]
    fn test() {
        let channel = Channel::new();
        thread::scope(|s| {
            s.spawn(|| {
                channel.as_sender().send(1);
            });
            s.spawn(|| {
                let receiver = channel.as_receiver();
                let msg = receiver.receive();
                assert_eq!(msg, Some(1));
            });
        });
        println!("Done");
    }
}
