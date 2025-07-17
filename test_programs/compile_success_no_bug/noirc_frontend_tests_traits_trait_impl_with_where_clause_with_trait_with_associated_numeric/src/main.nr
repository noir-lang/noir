
    trait Bar {
        let N: Field;
    }

    impl Bar for Field {
        let N: Field = 42;
    }

    trait Foo {
        fn foo<B>(b: B) where B: Bar; 
    }

    impl Foo for Field{
        fn foo<B>(_: B) where B: Bar {} 
    }

    fn main() {}
    