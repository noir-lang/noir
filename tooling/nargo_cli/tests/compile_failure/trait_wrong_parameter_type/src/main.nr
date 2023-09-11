trait Default {
    fn default(x: Field, y: NotAType) -> Field;
}

fn main(x: Field, y: Field) {
    assert(y == x);
}
