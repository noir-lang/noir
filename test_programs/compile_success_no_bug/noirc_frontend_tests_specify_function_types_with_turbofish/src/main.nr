
        trait Default2 {
            fn default2() -> Self;
        }

        impl Default2 for Field {
            fn default2() -> Self { 0 }
        }

        impl Default2 for u64 {
            fn default2() -> Self { 0 }
        }

        // Need the above as we don't have access to the stdlib here.
        // We also need to construct a concrete value of `U` without giving away its type
        // as otherwise the unspecified type is ignored.

        fn generic_func<T, U>() -> (T, U) where T: Default2, U: Default2 {
            (T::default2(), U::default2())
        }

        fn main() {
            let _ = generic_func::<u64, Field>();
        }
    