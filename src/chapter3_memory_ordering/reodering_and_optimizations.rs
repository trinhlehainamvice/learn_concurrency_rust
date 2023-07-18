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
