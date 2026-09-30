// The out-of-bounds write happens before the `if`, so it must fail whichever branch is taken.
fn main(arr: [Field; 4], j: u32, c: bool) -> pub [Field; 4] {
    let mut x = arr;
    x[j] = 7;
    if c { x } else { arr }
}
