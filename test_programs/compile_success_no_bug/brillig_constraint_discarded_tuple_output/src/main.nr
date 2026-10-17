struct Point {
    x: [Field; 4],
    y: [Field; 4],
    is_infinity: bool,
}

unconstrained fn compute_with_transcript(x: Field) -> (Point, [Field; 2]) {
    let point = Point { x: [x; 4], y: [x; 4], is_infinity: false };
    (point, [x, x * 2])
}

fn main(x: Field) -> pub Field {
    // Safety: the transcript is checked below; the point is discarded
    let transcript = unsafe { compute_with_transcript(x).1 };
    assert_eq(transcript[0], x);
    assert_eq(transcript[1], x * 2);
    transcript[1]
}
