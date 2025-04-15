
        fn main() {
            match 2 {
                Foo::One(_) => (),
            }
        }

        enum Foo {
            One(i32),
        }
    