
    struct Bar {}

    pub trait Foo {
        fn foo();
    }

    impl Foo for Bar {
        fn foo() {}
    }

    pub fn main() {
        Bar::foo()
    }
    