fn main(a: bool, b: bool, c: u8, d: u8) -> pub u8 {
    let i = a & b;
    let j = a ^ b;
    let k = a | b;
    let x = c & d;
    let y = c ^ d;
    let z = c | d;
    (i as u8) + (j as u8) + (k as u8) + x + y + z
}
