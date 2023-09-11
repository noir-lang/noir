trait Default {
}

struct Foo {
    bar: Field,
}

// Duplicate trait implementations should not compile
impl Default for Foo {
}

// Duplicate trait implementations should not compile
impl Default for Foo {
}


fn main(x: Field, y: Field) {
}
