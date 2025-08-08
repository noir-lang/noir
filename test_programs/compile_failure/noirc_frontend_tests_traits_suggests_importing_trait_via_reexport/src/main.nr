
    mod one {
        mod two {
            pub trait Trait {
                fn method(self);
            }

            impl Trait for bool {
                fn method(self) {}
            }
        }

        pub use two::Trait;
    }

    fn main() {
        true.method()
    }
    