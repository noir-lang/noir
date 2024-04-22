global NUM_RESULTS = 2;
fn main(x: field, y: pub field, enable: bool) -> pub [field; NUM_RESULTS] {
    let mut result = [0; NUM_RESULTS];
    for i in 0..NUM_RESULTS {
        if enable {
            result[i] = return_value(x, y);
        }
    }
    result
}

#[fold]
fn return_value(x: field, y: field) -> field {
    assert(x != y);
    x
}
