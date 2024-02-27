pub fn from_baz(x: [field; crate::foo::MAGIC_NUMBER]) {
    for i in 0..crate::foo::MAGIC_NUMBER {
        assert(x[i] == crate::foo::MAGIC_NUMBER as field);
    }
}
