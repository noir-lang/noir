fn main(x: u128, y: u128) -> pub u128 {
    let a = x + y;
    let b = a * y;
    let c = b / y;
    let d = c - y;
    let e = d % y;
    let f = e % !x;
    let g = f ^ y;
    let h = g >> 1;
    let i = h << 2;
    let j = i & x;
    let k = j | y;
    k
}
