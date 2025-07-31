
    pub trait BarTrait {}

    pub trait Foo {
        type Bar;
    }

    pub trait Baz<T>
    where
        <T as Foo>::Bar: BarTrait,
        T: Foo,
    {}

    pub struct HasBarTrait1 {}
    impl BarTrait for HasBarTrait1 {}

    pub struct HasFoo1 {}
    impl Foo for HasFoo1 {
        type Bar = HasBarTrait1;
    }

    pub struct HasBaz1 {}
    impl Baz<HasFoo1> for HasBaz1 {}

    fn main() {}
    