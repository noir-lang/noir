
    pub trait Other {
        fn other(self) {
            let _ = self;
        }
    }

    pub trait Trait<T: Other> {
        fn foo(x: T) {
            x.other();
        }
    }

    fn main() {}
    