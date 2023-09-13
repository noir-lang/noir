trait Default {
}

struct MyStruct {
}

type MyType = MyStruct;

// Duplicate trait implementations should not compile
impl Default for MyStruct {
}

// Duplicate trait implementations should not compile
impl Default for MyType {
}


fn main(x: Field, y: Field) {
}
