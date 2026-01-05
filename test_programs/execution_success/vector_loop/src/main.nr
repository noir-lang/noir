struct Point {
    x: Field,
    y: Field,
}

impl Point {
    fn serialize(self) -> [Field; 2] {
        [self.x, self.y]
    }
}

fn sum(values: [Field]) -> Field {
    let mut sum = 0;
    for value in values {
        sum = sum + value;
    }
    sum
}

fn main(points: [Point; 3]) {
    let mut serialized_points = #[];
    for point in points {
        serialized_points = serialized_points.append(point.serialize().as_vector());
    }
    // Do a compile-time check that needs the previous loop to be unrolled
    if serialized_points.len() > 5 {
        let empty_point = Point { x: 0, y: 0 };
        serialized_points = serialized_points.append(empty_point.serialize().as_vector());
    }
    // Do a sum that needs both the previous loop and the previous if to have been simplified
    assert_eq(sum(serialized_points), 21);
}
