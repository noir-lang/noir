use dep::protocol_types::traits::{Eq, Serialize, Deserialize};

struct MockStruct {
    a: Field,
    b: Field,
}

impl MockStruct {
    fn new(a: Field, b: Field) -> Self {
        Self { a, b }
    }
}

impl Eq for MockStruct {
    fn eq(self, other: Self) -> bool {
        (self.a == other.a) & (self.b == other.b)
    }
}

impl Serialize<2> for MockStruct {
    fn serialize(self) -> [Field; 2] {
        [self.a, self.b]
    }
}

impl Deserialize<2> for MockStruct {
    fn deserialize(fields: [Field; 2]) -> Self {
        Self { a: fields[0], b: fields[1] }
    }
}

#[test]
fn test_serde() {
    let val = MockStruct::new(5, 6);
    assert_eq(val, MockStruct::deserialize(val.serialize()));
}
