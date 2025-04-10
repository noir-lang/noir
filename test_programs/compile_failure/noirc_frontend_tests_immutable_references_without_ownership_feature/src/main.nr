
        fn main() {
            let mut array = [1, 2, 3];
            borrow(&array);
        }

        fn borrow(_array: &[Field; 3]) {}
    