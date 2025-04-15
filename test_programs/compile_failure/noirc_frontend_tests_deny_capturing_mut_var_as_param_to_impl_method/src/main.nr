
    struct Foo {
        value: Field,
    }

    impl Foo {
        fn mutate(&mut self) {
            self.value = 2;
        }
    }

    fn main() {
        let mut foo = Foo { value: 1 };
        let f = || foo.mutate();
        f();
        assert(foo.value == 2);
    }
    