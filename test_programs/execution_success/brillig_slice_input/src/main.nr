struct Point {
    x: Field,
    y: Field,
}

unconstrained fn sum_slice(slice: [Point]) -> Field {
    let mut sum = 0;
    for i in 0..slice.len() {
        sum += slice[i].x + slice[i].y;
    }
    sum
}

fn main() {
    let mut slice = &[];
    slice = slice.push_back(Point {
        x: 13,
        y: 14,
    });
    slice = slice.push_front(Point {
        x: 20,
        y: 8,
    });
    let brillig_sum = sum_slice(slice);
    assert_eq(brillig_sum, 55);
}
