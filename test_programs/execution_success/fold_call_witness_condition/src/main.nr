global NUM_RESULTS: u32 = 2;
fn main(x: Field, y: pub Field, enable: bool) -> pub [Field; NUM_RESULTS] {
    let mut result = [0; NUM_RESULTS];
    for i in 0..NUM_RESULTS {
        if enable {
            result[i] = return_value(x, y);
        }
    }
    result
}

#[fold]
fn return_value(x: Field, y: Field) -> Field {
    assert(x != y);
    x
}
