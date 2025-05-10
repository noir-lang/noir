trait TraitWithAssociatedConstant {
    let N: u32;

    fn foo(_: Self) -> bool {
        true
    }
}

struct Foo {}

impl TraitWithAssociatedConstant for Foo {
    let N: u32 = 4;
}

struct Bar {}

impl TraitWithAssociatedConstant for Bar {
    let N: u32 = 5;
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

    let wrapper = Wrapper { inner: Bar {} };
    assert_eq(wrapper, wrapper);
}
