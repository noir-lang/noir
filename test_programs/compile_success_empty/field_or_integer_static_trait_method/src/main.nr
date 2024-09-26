trait Read {
    fn read(data: [Field; 1]) -> Self;
}

impl Read for Field {
    fn read(data: [Field; 1]) -> Self {
        data[0] * 10
    }
}

impl Read for u32 {
    fn read(data: [Field; 1]) -> Self {
        data[0] as u32
    }
}

fn main() {
    let data = [1];

    let value: u32 = u32::read(data);
    assert_eq(value, 1);

    let value: Field = Field::read(data);
    assert_eq(value, 10);
}

#[attr]
pub fn foo() {}

comptime fn attr(_: FunctionDefinition) -> Quoted {
    quote { pub fn hello() {} }
}
