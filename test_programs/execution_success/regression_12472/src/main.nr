fn main(a: [(u32, u64); 2]) -> pub u32 {
    let s = a.as_vector();
    let s2 = s.push_back((99, 100));
    s2.len() as u32
}
