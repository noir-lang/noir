mod foo {
    mod bar {
        pub mod baz {
            pub fn func() {}

            pub mod qux {
                pub fn func() {}
            }

            mod one {
                pub mod two {
                    pub fn func() {}
                }
            }

            pub use one::two;
        }
    }

    pub use bar::baz;
}

mod test {
    mod nested {
        use super::utils;

        pub fn use_utils() {
            utils::func()
        }
    }

    mod utils {
        pub fn func() {}
    }
}

fn main() {
    foo::baz::func();
    foo::baz::qux::func();
    foo::baz::two::func();
}
