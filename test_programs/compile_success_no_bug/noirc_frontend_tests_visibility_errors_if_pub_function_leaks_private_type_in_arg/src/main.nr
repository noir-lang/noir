
    pub mod moo {
        struct Bar {}
        pub fn bar(_bar: Bar) {}

        pub fn no_unused_warnings() {
            let _ = Bar {};
        }
    }
    fn main() {}
    