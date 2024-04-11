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
    let mut serialized_points = &[];
    for point in points {
        serialized_points = serialized_points.append(point.serialize().as_slice());
    }
    assert_eq(sum(serialized_points), 21);
}
