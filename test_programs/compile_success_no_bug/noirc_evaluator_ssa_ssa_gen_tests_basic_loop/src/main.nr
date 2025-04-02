fn main(sum_to_check: u32) {
    let mut sum = 0;
    for i in 0..4 {
        sum = sum + i;
    }
    assert(sum_to_check == sum);
}
