
        fn main() {
            match Opt::Some(3) {
                Opt::None => (),
                Opt::Some(_) => {},
                Opt::None => (),
                3 => (),
            }
        }

        enum Opt<T> {
            None,
            Some(T),
        }
    