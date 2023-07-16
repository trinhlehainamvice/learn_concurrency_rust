fn f() {
    println!("Started another thread");

    std::thread::spawn(g);

    let id = std::thread::current().id();
    println!("Current thread id: {id:?}");
}

fn g() {
    println!("Started another thread spawned from another thread");

    let id = std::thread::current().id();
    println!("Current thread id: {id:?}");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test() {
        println!("Say hello from main thread");
        let t1 = std::thread::spawn(f);
        let t2 = std::thread::spawn(f);
        let numbers = Vec::from_iter(0..=1000);

        // When a spawned thread borrow or reference a variable from another thread
        // Rust need us to ensure that variable lifetime need to outlive the spawned thread's lifetime
        // In this case, we capture numbers variable from main thread through a closure passed to spawned thread
        // There is possible that numbers can be dropped before the spawned thread is finished  
        // So we move numbers variable to the spawned thread
        let t3 = std::thread::spawn(move || {
            let len = numbers.len() as i32;
            let sum = numbers.into_iter().sum::<i32>();
            sum / len
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let average = t3.join().unwrap();
        println!("Average: {average:?}");

        let id = std::thread::current().id();
        println!("Say goodbye from main thread: {id:?}");
    }
}