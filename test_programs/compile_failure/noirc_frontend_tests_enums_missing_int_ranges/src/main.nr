
        fn main() {
            let x: i8 = 3;
            match Opt::Some(x) {
                Opt::Some(4) => (),
                Opt::Some(6) => (),
            }
        }

        enum Opt<T> {
            None,
            Some(T),
        }
    