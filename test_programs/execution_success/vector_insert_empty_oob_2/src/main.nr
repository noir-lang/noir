fn main(arr: [u32; 1], cond: bool) {
    let slice = arr.as_vector();
    let (empty, _) = slice.remove(0);

    if cond {
        let inserted = empty.insert(0, 5);
        assert_eq(inserted.len(), 1);
    }
}
