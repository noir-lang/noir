
    trait Parent {}

    trait Child: Parent {
        fn child() {}
    }

    pub struct Struct<T> {}

    impl<T: Parent> Parent for Struct<T> {}
    impl<T: Child> Child for Struct<T> {}

    fn main() {}
    