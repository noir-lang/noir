unconstrained fn main(start: i64, end: i64) -> pub u32 {
    let start = (start % 5);
    let end = (end % 5);
    let mut sum = 0;
    for i in start..end {
        sum += 1;
    }
    sum
}
