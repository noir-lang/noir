fn main(sum_limit: u64, desired_sum: pub u64) {
    let actual_sum = unsafe { 
       sum(sum_limit)
    };
    assert(actual_sum == desired_sum);
}

unconstrained fn sum(sum_limit: u64) -> u64 {
    let mut sum = 0;
    for x in 1..sum_limit {
        sum += x;
    }
    sum
}
