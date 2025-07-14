
    trait Bar {
        type typ;
    }

    impl Bar for Field {
        type typ = Field;
    }

    trait Foo {
        fn foo<B>(b: B) where B: Bar; 
    }

    impl Foo for Field{
        fn foo<B>(_: B) where B: Bar {} 
    }

    fn main() {}
    