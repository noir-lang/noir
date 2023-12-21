use dep::std::ops::Eq;

struct FieldWrapper {
    inner: Field
}

impl Eq for FieldWrapper {
    fn eq(self, other: FieldWrapper) -> bool {
        self.inner == other.inner
    }
}

fn main(x: FieldWrapper) {
    let y = FieldWrapper { inner: 42 };
    assert(x == y);
}
