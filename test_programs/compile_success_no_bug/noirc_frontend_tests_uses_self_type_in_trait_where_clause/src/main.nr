
    pub trait Trait {
        fn trait_func(self) -> bool;
    }

    pub trait Foo where Self: Trait {
        fn foo(self) -> bool {
            self.trait_func()
        }
    }

    struct Bar {}

    impl Foo for Bar {

    }

    fn main() {
        let _ = Bar {}; // silence Bar never constructed warning
    }
    