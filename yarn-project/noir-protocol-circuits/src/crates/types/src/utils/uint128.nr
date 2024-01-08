// This is a diversion from the cpp code.
// The cpp code uses Fields for log_hashes
// whereas we are using u128 to make sure that it is really a u128.
struct U128 {
    inner : Field
}

impl U128 {
    pub fn to_field(self) -> Field {
        self.inner as Field
    }

    pub fn from_field(value : Field) -> U128 {
        // TODO(Kev): Apply 128 bit range constraint and fail if this is not the case.
        // We can expose a `apply_range_constraint` method from Noir which can take a field 
        // and return a Field.
        // It won't be type-safe, but thats fine. We may then be able to implement
        // u128 in the stdlib and have it be called automatically when a user 
        // does `let x :u128 = 0;` We will require traits to make operations nice.
        U128{inner : value}
    }

    pub fn to_be_bytes(self) -> [u8;16] {
        let slice = self.inner.to_be_bytes(16);
        let mut arr = [0;16];
        for i in 0..16 {
            arr[i] = slice[i];
        }
        arr
    }
}
