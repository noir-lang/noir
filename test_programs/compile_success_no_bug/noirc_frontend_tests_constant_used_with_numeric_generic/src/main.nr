struct ValueNote {
    value: Field,
}

trait Serialize<let N: u32> {
    fn serialize(self) -> [Field; N];
}

impl Serialize<1> for ValueNote {
    fn serialize(self) -> [Field; 1] {
        [self.value]
    }
}

fn main() {
    let _ = ValueNote { value: 1 }; // silence ValueNote never constructed warning
}
