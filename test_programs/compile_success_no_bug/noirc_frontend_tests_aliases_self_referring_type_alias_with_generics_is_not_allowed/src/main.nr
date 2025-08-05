
        type Id<T> = T;

        fn main() {
            let x: Id<Id<Field>> = 1;
        }
    