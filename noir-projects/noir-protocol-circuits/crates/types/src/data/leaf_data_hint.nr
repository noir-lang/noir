trait LeafDataHint {
    fn leaf_slot(self) -> Field;
    fn value(self) -> Field;
    fn override_counter(self) -> u32;
}

