fn main(i: u32, cond: bool) -> pub u32 {
    let arr: [u32; 3] = [10, 20, 30];
    let mut s: [u32] = arr.as_vector();
    if cond {
        let (s2, _) = s.remove(i);
        s = s2;
    }
    s.len() as u32
}
