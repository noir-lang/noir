fn main(i: u32, a: bool, b: bool) -> pub u32 {
    let mut vec: [u32] = @[10, 20, 30];
    if a {
        vec = vec.remove(i).0;
    } else if b {
        vec = vec.insert(i, 40);
    }
    vec.len() as u32
}
