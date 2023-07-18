#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::sync::atomic::{fence, AtomicBool, AtomicPtr, AtomicU32};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test() {
        static DATA: AtomicU32 = AtomicU32::new(0);
        static FLAG: AtomicBool = AtomicBool::new(false);

        thread::spawn(|| {
            DATA.store(123, Relaxed);
            // FLAG.store(true, Release);
            FLAG.store(true, Relaxed);
        });

        // while !FLAG.load(Acquire) {}
        while !FLAG.load(Relaxed) {}

        assert_eq!(DATA.load(Relaxed), 123);
    }

    #[test]
    fn test2() {
        static mut DATA: u32 = 0;
        static READY: AtomicBool = AtomicBool::new(false);

        let producer = thread::spawn(|| {
            unsafe {
                DATA = 123;
            }
            READY.store(true, Release);
        });

        let consumer = thread::spawn(|| {
            while !READY.load(Acquire) {}

            assert_eq!(123, unsafe { DATA });
        });

        producer.join().unwrap();
        consumer.join().unwrap();
    }

    #[test]
    fn test_lock() {
        static mut DATA: i32 = 0;
        static LOCK: AtomicBool = AtomicBool::new(false);

        fn f() {
            // if LOCK.compare_exchange(false, true, Acquire, Relaxed)
            if !LOCK.swap(true, Acquire) {
                unsafe {
                    DATA += 1;
                }
                LOCK.store(false, Release);
            }
        }

        thread::scope(|s| {
            for _ in 0..100 {
                s.spawn(f);
            }
        });

        assert_eq!(100, unsafe { DATA });
    }

    #[test]
    fn test_lazy_init() {
        struct Data(i32);
        type Ptr = *mut Data;
        fn generate_data() -> Data {
            static mut DATA: i32 = 0;
            unsafe {
                DATA += 1;
                println!("DATA: {}", DATA);
                Data(DATA)
            }
        }

        static DATA: AtomicPtr<Data> = AtomicPtr::new(std::ptr::null_mut());
        fn get_data() -> &'static Data {
            let mut ptr = DATA.load(Acquire);
            if ptr.is_null() {
                ptr = Box::into_raw(Box::new(generate_data()));
                if let Err(generated_data_ptr) =
                    DATA.compare_exchange(std::ptr::null_mut(), ptr, Release, Acquire)
                {
                    unsafe {
                        let _ = Box::from_raw(ptr);
                    }
                    ptr = generated_data_ptr;
                }
            }

            unsafe { &*ptr }
        }

        thread::scope(|s| {
            let thread_a = s.spawn(get_data);
            let thread_b = s.spawn(get_data);
            let thread_c = s.spawn(get_data);

            let a = thread_a.join().unwrap();
            let b = thread_b.join().unwrap();
            let c = thread_c.join().unwrap();

            assert_eq!(a.0, b.0);
            assert_eq!(b.0, c.0);
            assert_eq!(a.0, c.0);
        });
    }

    #[test]
    fn test_fench() {
        fn generate_data() -> u64 {
            static mut DATA: u64 = 0;
            unsafe {
                DATA += 1;
                DATA
            }
        }

        static mut DATA: [u64; 10] = [0; 10];
        const READY_FALSE: AtomicBool = AtomicBool::new(false);
        static READY: [AtomicBool; 10] = [READY_FALSE; 10];

        for i in 0..10 {
            thread::spawn(move || {
                fence(Release);
                let data = generate_data();
                unsafe {
                    DATA[i] = data;
                }
                READY[i].store(true, Relaxed);
            });
        }

        thread::sleep(Duration::from_millis(100));

        let ready: [bool; 10] = std::array::from_fn(|i| READY[i].load(Relaxed));
        if ready.iter().all(|r| *r) {
            fence(Acquire);
            for data in unsafe { &DATA } {
                println!("DATA: {}", *data);
            }
        }
    }
}
