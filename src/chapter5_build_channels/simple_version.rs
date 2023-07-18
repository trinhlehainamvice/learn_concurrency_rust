use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};

pub(crate) struct Channel<T> {
    msg_queue: Mutex<VecDeque<T>>,
    ready: Condvar,
}

impl<T> Channel<T> {
    pub const fn new() -> Channel<T> {
        Channel {
            msg_queue: Mutex::new(VecDeque::new()),
            ready: Condvar::new(),
        }
    }

    pub fn send(&self, msg: T) {
        self.msg_queue.lock().unwrap().push_back(msg);
        self.ready.notify_one();
    }

    pub fn receive(&self) -> T {
        let mut guard = self.msg_queue.lock().unwrap();
        loop {
            if let Some(msg) = guard.pop_front() {
                return msg;
            }
            guard = self.ready.wait(guard).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chapter5_build_channels::simple_version::Channel;
    use std::thread;

    #[test]
    fn test() {
        let channel = Channel::new();
        thread::scope(|s| {
            s.spawn(|| {
                channel.send(1);
            });
            s.spawn(|| {
                let msg = channel.receive();
                assert_eq!(msg, 1);
            });
        });
        println!("Done");
    }
}
