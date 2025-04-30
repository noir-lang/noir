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

impl<T> std::cmp::Eq for Wrapper<T>
where
    T: TraitWithAssociatedConstant,
{

    fn eq(self, _: Self) -> bool {
        self.inner.foo()
    }
}

fn main() {
    let wrapper = Wrapper { inner: Foo {} };
    assert_eq(wrapper, wrapper);
}
