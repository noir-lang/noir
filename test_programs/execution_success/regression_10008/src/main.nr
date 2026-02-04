fn main(v0: i16, v1: bool) -> pub i16 {
    let v4 = !(v0 - v0);
    if v1 {
        // b1
        let v6 = v4 / v4;
        let v7 = !v6;
        let v8 = v4 / v7;
        let v9 = v8 ^ v0;
        let v10 = !v9;
        let _ = v10 / v4;
    } else {
        let _ = v4 / v4;
    }
    v4 / v4
}
