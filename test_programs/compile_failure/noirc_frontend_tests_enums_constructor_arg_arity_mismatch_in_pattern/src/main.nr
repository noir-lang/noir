
        fn main() {
            match Foo::One(1) {
                Foo::One(_, _) => (),
                Foo::Two(_) => (),
            }
        }

        enum Foo {
            One(i32),
            Two(i32, i32),
        }
    