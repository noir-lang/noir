fn main(x: u8, y: u8, z: u8) -> pub u8 {
    let p = y - z;
    if p != 0 {
        let a = x / z;
        let b = a - z;
        if b != 0 {
            let _ = a / b;
        }
    }

    let u = x - z;
    let v = y - u;
    v
}
