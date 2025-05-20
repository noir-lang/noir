pub fn assert_array_prepended<T, let N: u32>(dest: [T; N], source: [T; N], num_source_items: u32)
where
    T: Eq,
{
    let mut proceed = true;
    for i in 0..source.len() {
        proceed &= i != num_source_items;
        if proceed {
            assert_eq(dest[i], source[i], "source item does not prepend to dest");
        }
    }
}
