#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::thread;

    #[test]
    fn test() {
        let num = 3;
        let cell_num = Cell::new(3);

        thread::scope(|s| {
            s.spawn(|| {
                println!("{num:?}");
            });
            s.spawn(move || {
                println!("{cell_num:?}");
            });
            /* Error
            ** Cell can be shared safely between threads because Cell is not implemented Sync
            s.spawn(|| {
                println!("{cell_num:?}");
            });
            */
        });
    }
}
