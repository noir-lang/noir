fn main(arr: [u32; 1], cond: bool) {
    let vector = arr.as_vector();
    let (empty, _) = vector.remove(0);

    if cond {
        let inserted = empty.insert(0, 5);
        assert_eq(inserted.len(), 1);
    }
}
