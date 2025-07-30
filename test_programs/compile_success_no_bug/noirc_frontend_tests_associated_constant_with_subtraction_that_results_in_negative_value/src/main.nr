
    trait Foo {
        let N: i32;

        fn n() -> i32 {
            Self::N
        }
    }

    impl Foo for Field {
        let N: i32 = 1 - 10;
    }

    fn main() {
        foo(Field::n());
    }

    fn foo(_: i32) {}
    