fn main(x: u128, y: u128) -> pub u128 {
    let a = x + y;
    let b = a * y;
    let c = b / y;
    let d = c - y;
    let e = d % y;
    let f = e % !x;
    f
}
