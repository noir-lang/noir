
        fn main() {
            match Opt::Some(3) {
                Opt::None => (),
            }
        }

        enum Opt<T> {
            None,
            Some(T),
        }
    