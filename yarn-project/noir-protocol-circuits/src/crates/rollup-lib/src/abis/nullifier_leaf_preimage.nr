struct NullifierLeafPreimage {
    leaf_value : Field,
    next_value :Field,
    next_index : u32,
}

impl NullifierLeafPreimage {
    // TODO(Kev): Check if this is needed.
    fn is_empty(self) -> bool {
        (self.leaf_value == 0) & (self.next_index == 0) & (self.next_value == 0)
    }

    fn hash(self) -> Field {
        if self.is_empty() {
            0
        } else {
            dep::std::hash::pedersen_hash([self.leaf_value, (self.next_index as Field), self.next_value])
        }
    }
}