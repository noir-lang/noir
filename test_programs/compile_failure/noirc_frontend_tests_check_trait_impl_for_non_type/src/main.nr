
    trait Default2 {
        fn default(x: Field, y: Field) -> Field;
    }

    impl Default2 for main {
        fn default(x: Field, y: Field) -> Field {
            x + y
        }
    }

    fn main() {}
    