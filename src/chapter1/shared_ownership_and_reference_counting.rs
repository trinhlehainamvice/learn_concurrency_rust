static X: [i32; 5] = [1, 2, 3, 4, 5];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let t1 = std::thread::spawn(|| dbg!(&X));
        let t2 = std::thread::spawn(|| dbg!(&X));
        t1.join().unwrap();
        t2.join().unwrap();

        let x: &'static [i32; 3] = Box::leak(Box::new([1, 5, 3]));
        let t1 = std::thread::spawn(move || dbg!(x));
        let t2 = std::thread::spawn(move || dbg!(x));
        t1.join().unwrap();
        t2.join().unwrap();

        let x = std::rc::Rc::new(vec![1, 5, 3]);

        let a = std::sync::Arc::new(vec![1, 5, 3]);
    }
}
