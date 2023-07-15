use std::cell::{Cell, RefCell};

fn f(a: &Cell<i32>, b: &Cell<i32>) {
    let before = a.get();
    b.set(b.get() + 1);
    let after = a.get();
    if before != after {
        // No guarantee that a and b reference to different variables any more
        // -> code inside this block can be executed if a and b reference to the same variable
        println!("{} {}", before, after);
    }
}

fn set_vec_cell(nums: &Cell<Vec<i32>>, num: i32) {
    let mut temp_nums = nums.take();
    temp_nums.push(num);
    nums.set(temp_nums);
}

fn set_vec_refcell(nums: &RefCell<Vec<i32>>, num: i32) {
    let mut mut_borrowed_nums = nums.borrow_mut();
    let borrowed_nums = nums.borrow();

    mut_borrowed_nums.push(num);
    println!("{borrowed_nums:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let num = Cell::new(13);
        f(&num, &num);
    }
}
