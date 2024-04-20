struct Hasher {
    fields: [field],
}

impl Hasher {
    pub fn new() -> Self {
        Self { fields: [] }
    }

    pub fn add(&mut self, field: field) {
        self.fields = self.fields.push_back(field);
    }
}

fn main(expected: pub field, first: field) {
    let mut hasher = Hasher::new();
    hasher.add(first);
    assert(hasher.fields[0] == expected);
}
