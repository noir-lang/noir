// An primary attribute should not be able to be added to a struct defintion
#[oracle(some_oracle)]
struct SomeStruct{
    x: Field,
    y: Field
}

fn main() {}
