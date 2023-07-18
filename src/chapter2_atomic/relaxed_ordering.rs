use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

static X: AtomicU32 = AtomicU32::new(0);

fn a() {
    X.fetch_add(10, Relaxed);
    X.fetch_add(5, Relaxed);
}

fn a1() {
    X.fetch_add(10, Relaxed);
}

fn a2() {
    X.fetch_add(5, Relaxed);
}

fn b() {
    let a = X.load(Relaxed);
    let b = X.load(Relaxed);
    let c = X.load(Relaxed);
    let d = X.load(Relaxed);
    // If we run a() from another thread
    // The result will be guaranteed to be in order of [0->10->15]
    
    // But if we run a1() and a2() by its own thread
    // The order will not be guaranteed anymore
    // But the result will start from [0->...]
    println!("{a} {b} {c} {d}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        a();
        b();
    }
}
