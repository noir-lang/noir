
    mod foo {
        mod bar {
            pub fn baz() {}
        }

        use bar::baz;

        pub fn qux() {
            baz();
        }
    }

    fn main() {
        foo::baz();
    }
    