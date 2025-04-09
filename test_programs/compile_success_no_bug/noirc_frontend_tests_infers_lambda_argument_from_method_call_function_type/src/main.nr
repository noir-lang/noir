
    struct Foo {
        value: Field,
    }

    impl Foo {
        fn foo(self) -> Field {
            self.value
        }
    }

    struct Box<T> {
        value: T,
    }

    impl<T> Box<T> {
        fn map<U>(self, f: fn(T) -> U) -> Box<U> {
            Box { value: f(self.value) }
        }
    }

    fn main() {
        let box = Box { value: Foo { value: 1 } };
        let _ = box.map(|foo| foo.foo());
    }
    