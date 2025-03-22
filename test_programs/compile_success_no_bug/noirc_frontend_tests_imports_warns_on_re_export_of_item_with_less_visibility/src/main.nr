
    mod foo {
        mod bar {
            pub(crate) fn baz() {}
        }

        pub use bar::baz;
    }

    fn main() {
        foo::baz();
    }
    