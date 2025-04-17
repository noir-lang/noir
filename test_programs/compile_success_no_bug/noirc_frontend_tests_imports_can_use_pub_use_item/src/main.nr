
    mod foo {
        mod bar {
            pub fn baz() {}
        }

        pub use bar::baz;
    }

    fn main() {
        foo::baz();
    }
    