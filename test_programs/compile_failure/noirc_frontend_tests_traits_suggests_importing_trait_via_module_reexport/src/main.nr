
    mod one {
        mod two {
            pub mod three {
                pub trait Trait {
                    fn method(self);
                }

                impl Trait for bool {
                    fn method(self) {}
                }
            }
        }

        pub use two::three;
    }

    fn main() {
        true.method()
    }
    