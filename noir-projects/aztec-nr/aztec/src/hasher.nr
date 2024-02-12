use dep::protocol_types::{hash::hash_args, traits::Hash};

struct Hasher {
    fields: [Field],
}

impl Hash for Hasher {
    fn hash(self) -> Field {
        hash_args(self.fields)
    }
}

impl Hasher {
    pub fn new() -> Self {
        Self { fields: [] }
    }

    pub fn add(&mut self, field: Field) {
        self.fields = self.fields.push_back(field);
    }

    pub fn add_multiple<N>(&mut self, fields: [Field; N]) {
        for i in 0..N {
            self.fields = self.fields.push_back(fields[i]);
        }
    }
}
