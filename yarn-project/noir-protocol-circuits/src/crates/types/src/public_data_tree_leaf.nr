use crate::traits::Empty;
use dep::std::cmp::Eq;

struct PublicDataTreeLeaf {
    slot: Field,
    value: Field,
}

impl Eq for PublicDataTreeLeaf {
    fn eq(self, other: Self) -> bool {
        (self.slot == other.slot) & (self.value == other.value)
    }
}

impl Empty for PublicDataTreeLeaf  {    
    fn empty() -> Self {
        Self {
            slot: 0,
            value: 0,
        }
    }
}

impl PublicDataTreeLeaf {
    pub fn is_empty(self) -> bool {
        (self.slot == 0) & (self.value == 0)
    }
}
