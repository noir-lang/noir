struct Point {
    x: Field,
    y: Field,
}

unconstrained fn sum_vector(vector: [[Point; 2]]) -> Field {
    let mut sum = 0;
    for i in 0..vector.len() {
        for j in 0..vector[i].len() {
            sum += vector[i][j].x + vector[i][j].y;
        }
    }
    sum
}

fn main() {
    let mut vector = [].as_vector();
    vector = vector.push_back([Point { x: 13, y: 14 }, Point { x: 20, y: 8 }]);
    // Safety: testing context
    unsafe {
        let brillig_sum = sum_vector(vector);
        assert_eq(brillig_sum, 55);
    }

    vector = vector.push_back([Point { x: 15, y: 5 }, Point { x: 12, y: 13 }]);
    // Safety: testing context
    unsafe {
        let brillig_sum = sum_vector(vector);
        assert_eq(brillig_sum, 100);
    }
}
