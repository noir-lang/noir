
        type Id<T> = T;

        fn main() {
            let _: Id<Id<Field>> = 1;
        }
    