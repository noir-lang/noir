
    mod trait_mod {
        pub trait Foo {
            fn foo(_: Self) {}
        }

        impl Foo for Field {}
    }

    use trait_mod::Foo as FooTrait;

    fn main(x: Field) {
        x.foo();
    }
    