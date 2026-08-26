fn main(arr: [u8; 2], c1: bool, c2: bool) -> pub u8 {
    let v: [u8] = arr.as_vector();
    let v1: [u8] = v.push_front(9);
    if c1 {
        let w: [u8] = v1.push_front(8);
        w[0]
    } else if c2 {
        let w: [u8] = v1.push_front(8);
        w[0]
    } else {
        v1[0]
    }
}
