struct Hasher {
    fields: [field],
}

impl Hasher {
    pub fn new() -> Self {
        Self { fields: [] }
    }

    pub fn add(&mut self, field_element: field) {
        self.fields = self.fields.push_back(field_element);
    }
}

fn main(expected: pub field, first: field) {
    let mut hasher = Hasher::new();
    hasher.add(first);
    assert(hasher.fields[0] == expected);
}
