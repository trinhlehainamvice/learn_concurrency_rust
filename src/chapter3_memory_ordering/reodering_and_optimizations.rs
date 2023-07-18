// Because there is no instruction that depend on each other, like a use b to calculate, etc...
// So each instruction can be executed in independent order
// Also Rust compiler will optimize the code to function g below
fn f(a: &mut i32, b: &mut i32) {
    *a += 1;
    *b += 1;
    *a += 1;
}

// Rust compiler see instructions on a and b are independent
// So compiler group all related to a or b instructions to one instruction
fn g(a: &mut i32, b: &mut i32) {
    *a += 2;
    *b += 1;
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
    use std::thread;

    #[test]
    fn test() {
        static LOCK: AtomicBool = AtomicBool::new(false);
        static mut DATA: [u32; 10] = [0; 10];

        thread::scope(|s| {
            // Produce threads
            for i in 0..10 {
                s.spawn(move || {
                    unsafe {
                        DATA[i] = i as u32;
                    }
                    LOCK.store(false, Release);
                });
            }
            // Consume threads
            for i in 0..10 {
                s.spawn(move || {
                    if !LOCK.swap(true, Acquire) {
                        println!("DATA is {}", unsafe { DATA[i] });
                        assert_eq!(unsafe { DATA[i] }, i as u32);
                    }
                });
            }
        });

        println!("Done");
    }
}
