fn main(i: u32, cond: bool) -> pub u32 {
    let arr: [u32; 3] = [10, 20, 30];
    let mut s: [u32] = arr.as_vector();
    if cond {
        s = s.insert(i, 42);
    }
    s.len() as u32
}
