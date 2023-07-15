#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut numbers = vec![1, 3, 100];
        std::thread::scope(|s| {
            s.spawn(|| {
                numbers.extend([2, 4, 5]);
            });
        });

        println!("{numbers:?}");
    }
}
