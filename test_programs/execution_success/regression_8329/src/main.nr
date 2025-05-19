fn main(x: bool, y: bool, z: bool) -> pub bool {
    let p = y - z;
    if p {
        let a = x / z;
        let b = a - z;
        if b {
            let _ = a / b;
        }
    }

    let u = x - z;
    let v = y - u;
    v
}
