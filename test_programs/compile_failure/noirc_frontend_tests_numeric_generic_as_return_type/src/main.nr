
    // std::mem::zeroed() without stdlib
    trait Zeroed {
        fn zeroed<T>(self) -> T;
    }

    fn foo<T, let I: Field>(x: T) -> I where T: Zeroed {
        x.zeroed()
    }

    fn main() {}
    