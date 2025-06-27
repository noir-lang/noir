
    mod trait_mod {
        pub trait Foo {
            fn foo(self) {}
        }

        impl Foo for Field {}
    }

    use trait_mod::Foo as FooTrait;

    fn main(x: Field) {
        x.foo();
    }
    