trait LeafPreimage {
    fn get_key(self) -> Field;
    fn as_leaf(self) -> Field;
}
