
    struct Foo {
        inner: u64
    }

    pub fn bar<let N: Foo>() {
        let _ = Foo { inner: 1 }; // silence Foo never constructed warning
    }
    