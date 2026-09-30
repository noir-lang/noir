// The out-of-bounds write happens before both `if`s, so it must fail whichever branches are taken.
fn main(arr: [Field; 4], j: u32, c1: bool, c2: bool) -> pub [Field; 4] {
    let mut x = arr;
    x[j] = 7;
    let mut r = arr;
    if c1 {
        r = if c2 { x } else { arr };
    }
    r
}
