
        fn main() {
            match Opt::Some(Opt::Some(3)) {
                Opt::Some(_) => (),
                Opt::None => (),
                Opt::Some(Opt::Some(_)) => (),
            }
        }

        enum Opt<T> {
            None,
            Some(T),
        }
    