// Test that try_optimize_array_get_from_previous_set does not 
// optimizes away an array_get when the preceding array_set
// might be disabled by enable_side_effects_if

fn main(condition: bool) {
    let mut arr = [1, 2, 3];

    // This array_set should be disabled when condition is false.
    if condition {
        arr[0] = 10;
    }

    // And then prevent to optimize the array-get with 10.
    let val = arr[0];
    assert(val == 1); 

}
