
        fn main() {
            let _tree: Opt<OptAlias<()>> = Opt::Some(OptAlias::None);
        }

        type OptAlias<T> = Opt<T>;

        enum Opt<T> {
            Some(T),
            None,
        }
        