fn from_baz(x : [Field; crate::foo::MAGIC_NUMBER]) {
     for i in 0..crate::foo::MAGIC_NUMBER {
          assert_eq(x[i], crate::foo::MAGIC_NUMBER);
     };
}
