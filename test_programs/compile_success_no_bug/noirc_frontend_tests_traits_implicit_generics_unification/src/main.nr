
    trait TraitWithAssociatedConstant {
        let N: u32;

        fn foo(_: Self) -> bool {
            true
        }
    }

    struct Foo {}

    impl TraitWithAssociatedConstant for Foo {
        let N: u32 = 42;
    }

    struct Wrapper<T> {
        inner: T,
    }

    trait Eq2 {
        fn eq(self, other: Self) -> bool;
    }

    impl<T> Eq2 for Wrapper<T>
    where
        T: TraitWithAssociatedConstant,
    {

        fn eq(self, _other: Self) -> bool {
            self.inner.foo()
        }
    }

    fn main() {
        let wrapper = Wrapper { inner: Foo {} };
        assert(wrapper.eq(wrapper));
    }
    