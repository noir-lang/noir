
        #[oracle(foo)]
        pub fn foo(x: Field, y: Field) {
            assert(x != y);
        }
    