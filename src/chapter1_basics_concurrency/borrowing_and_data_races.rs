fn f(a: &i32, b: &mut i32) {
    let before = *a;
    *b += 1;
    let after = *a;
    if before != after {
        // code inside this block will never be executed
        // because Rust compiler guarantee that a and b reference to different variables
        // let mut num = 3;
        // f(&num, &mut num); // error: A variable already borrowed as immutable was borrowed as mutable.
        println!("{} {}", before, after);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
