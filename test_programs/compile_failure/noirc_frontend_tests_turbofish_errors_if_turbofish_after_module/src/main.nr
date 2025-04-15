
    mod moo {
        pub fn foo() {}
    }

    fn main() {
        moo::<i32>::foo();
    }
    