struct NullifierLeafPreimage {
    leaf_value : Field,
    next_value :Field,
    next_index : u32,
}

impl NullifierLeafPreimage {
    pub fn default() -> Self {
        Self {
            leaf_value : 0,
            next_value : 0,
            next_index : 0,
        }
    }

    pub fn is_empty(self) -> bool {
        (self.leaf_value == 0) & (self.next_index == 0) & (self.next_value == 0)
    }

    pub fn hash(self) -> Field {
        if self.is_empty() {
            0
        } else {
            dep::std::hash::pedersen_hash([self.leaf_value, (self.next_index as Field), self.next_value])
        }
    }
}