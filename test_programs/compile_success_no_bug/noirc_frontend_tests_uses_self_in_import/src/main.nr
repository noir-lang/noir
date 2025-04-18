
    mod moo {
        pub mod bar {
            pub fn foo() -> i32 {
                1
            }
        }
    }

    use moo::bar::{self};

    pub fn baz() -> i32 {
        bar::foo()
    }

    fn main() {}
    