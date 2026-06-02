fn main(mut a: [(bool, bool); 3], b: u32, c: bool) -> pub bool {
    for _ in 0..1 {
        let _ = if c { a[b] } else { a[0] };
    }
    false
}
