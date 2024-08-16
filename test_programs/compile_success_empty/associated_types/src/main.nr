trait Collection {
    type Elem;

    fn cget(self, index: u32) -> Option<Self::Elem>;

    fn ctake(self, index: u32) -> Self::Elem {
        self.cget(index).unwrap()
    }
}

impl<T, let N: u32> Collection for [T; N] {
    type Elem = T;

    fn cget(self, index: u32) -> Option<T> {
        if index < self.len() {
            Option::some(self[index])
        } else {
            Option::none()
        }
    }
}

fn main() {
    // Use zeroed here so that we don't help by adding another type constraint.
    // We should know Elem = Field from the associated type alone.
    let array = [1, 2, 3, 0, 5];
    assert_eq(array.ctake(3), std::mem::zeroed());
}
