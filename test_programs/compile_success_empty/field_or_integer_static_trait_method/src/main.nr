trait Read {
    fn read(data: [Field; 1]) -> Self;
}

impl Read for Field {
    fn read(data: [Field; 1]) -> Self {
        data[0]
    }
}

impl Read for u32 {
    fn read(data: [Field; 1]) -> Self {
        data[0] as u32
    }
}

fn main() {
    let data = [1];
    let _: u32 = u32::read(data);
    let _: Field = Field::read(data);
}
