
        trait From2<T> {
        fn from2(input: T) -> Self;
    }
    struct U60Repr {}
    impl From2<[i32; 3]> for U60Repr {
        fn from2(_: [i32; 3]) -> Self {
            U60Repr {}
        }
    }
    impl From2<i32> for U60Repr {
        fn from2(_: i32) -> Self {
            U60Repr {}
        }
    }

    fn main() {
        let _ = U60Repr::<i32>::from2([1, 2, 3]);
    }
    