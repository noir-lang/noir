
fn main(x: u64) {
    let a = std::hint::black_box(|| {
        @[1, 2, 3]
    });

    let vector = if x > 2 {
        a()
    } else {
        @[4, 5]
    };

    assert_eq(vector[0], 1);
}
