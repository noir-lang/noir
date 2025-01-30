struct Point {
    x: Field,
    y: Field,
}

unconstrained fn sum_slice(slice: [[Point; 2]]) -> Field {
    let mut sum = 0;
    for i in 0..slice.len() {
        for j in 0..slice[i].len() {
            sum += slice[i][j].x + slice[i][j].y;
        }
    }
    sum
}

fn main() {
    let mut slice = &[];
    slice = slice.push_back([Point { x: 13, y: 14 }, Point { x: 20, y: 8 }]);
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        let brillig_sum = sum_slice(slice);
        assert_eq(brillig_sum, 55);
    }

    slice = slice.push_back([Point { x: 15, y: 5 }, Point { x: 12, y: 13 }]);
    /// Safety: testing context
    unsafe {
        //@safety: testing context
        let brillig_sum = sum_slice(slice);
        assert_eq(brillig_sum, 100);
    }
}
