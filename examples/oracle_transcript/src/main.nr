use std::test::OracleMock;

struct Point {
    x: Field,
    y: Field,
}

impl Eq for Point {
    fn eq(self, other: Point) -> bool {
        (self.x == other.x) & (self.y == other.y)
    }
}

#[oracle(void_field)]
unconstrained fn void_field_oracle() -> Field {}

unconstrained fn void_field() -> Field {
    void_field_oracle()
}

#[oracle(field_field)]
unconstrained fn field_field_oracle(_x: Field) -> Field {}

unconstrained fn field_field(x: Field) -> Field {
    field_field_oracle(x)
}

#[oracle(struct_field)]
unconstrained fn struct_field_oracle(_point: Point, _array: [Field; 4]) -> Field {}

unconstrained fn struct_field(point: Point, array: [Field; 4]) -> Field {
    struct_field_oracle(point, array)
}

fn main(input: Point) -> pub Field {
    // Safety: testing context
    unsafe {
        let a = void_field();
        let b = void_field();
        let c = field_field(input.x);
        let d = field_field(input.y);
        let p = Point { x: a * c, y: b * d };
        struct_field(p, [a, b, c, d])
    }
}

/// This test is used to capture an oracle transcript, which can then be replayed
/// during execution.
#[test]
fn test_main() {
    // Safety: testing context
    unsafe {
        let _ = OracleMock::mock("void_field").returns(10).times(1);
        let _ = OracleMock::mock("void_field").returns(20).times(1);
        let _ = OracleMock::mock("field_field").with_params((2,)).returns(30);
        let _ = OracleMock::mock("field_field").with_params((3,)).returns(40);
        let _ = OracleMock::mock("struct_field")
            .with_params((Point { x: 300, y: 800 }, [10, 20, 30, 40]))
            .returns(100);
    }
    let output = main(Point { x: 2, y: 3 });
    assert_eq(output, 100)
}
