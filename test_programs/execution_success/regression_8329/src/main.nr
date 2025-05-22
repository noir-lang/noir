fn main(x: u1, y: u1, z: u1) -> pub u1 {
    let p = y - z;
    if p as bool {
        let a = x / z;
        let b = a - z;
        if b as bool {
            let _ = a / b;
        }
    }

    let u = x - z;
    let v = y - u;
    v
}
