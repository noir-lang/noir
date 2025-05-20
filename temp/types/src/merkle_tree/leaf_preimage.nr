use crate::traits::Empty;

pub trait LeafPreimage {
    fn get_key(self) -> Field;
    fn as_leaf(self) -> Field;
}

pub trait IndexedTreeLeafPreimage<Value>: Eq + Empty + LeafPreimage {
    fn get_next_key(self) -> Field;

    fn points_to_infinity(self) -> bool;

    fn update_pointers(self, next_key: Field, next_index: u32) -> Self;

    fn update_value(self, value: Value) -> Self;

    fn build_insertion_leaf(value: Value, low_leaf: Self) -> Self;
}

pub trait IndexedTreeLeafValue: Eq + Empty {
    fn get_key(self) -> Field;
}

impl IndexedTreeLeafValue for Field {
    fn get_key(self) -> Field {
        self
    }
}
