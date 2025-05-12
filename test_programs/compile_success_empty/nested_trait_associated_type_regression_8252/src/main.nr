trait TraitWithAssociatedConstant {
    let N: u32;

    fn foo(_: Self) -> bool {
        true
    }
}

struct Foo {}
struct Bar {}

impl TraitWithAssociatedConstant for Foo {
    let N: u32 = 42;
}

impl<V> TraitWithAssociatedConstant for Wrapper<V>
where
    V: TraitWithAssociatedConstant,
{
    let N: u32 = <V as TraitWithAssociatedConstant>::N;
}
impl TraitWithAssociatedConstant for Bar {
    let N: u32 = 43;
}

struct Wrapper<T> {
    inner: T,
}

impl<U> std::cmp::Eq for Wrapper<U>
where
    U: TraitWithAssociatedConstant,
{
    fn eq(self, _other: Self) -> bool {
        self.inner.foo()
    }
}

fn main() {
    let wrapper = Wrapper { inner: Foo {} };
    assert(wrapper == wrapper);

    // Ensure dispatch works with a few additional variants
    assert(wrapper.eq(wrapper));
    assert(Eq::eq(wrapper, wrapper));
    assert_eq(wrapper, wrapper);

    // Ensure dispatch works with another type
    let wrapper = Wrapper { inner: Bar {} };
    assert_eq(wrapper, wrapper);

    let wrapper = Wrapper { inner: Wrapper { inner: Foo {} } };
    assert_eq(wrapper, wrapper);

    let wrapper = Wrapper {
        inner: Wrapper {
            inner: Wrapper {
                inner: Wrapper { inner: Wrapper { inner: Wrapper { inner: wrapper } } },
            },
        },
    };
    assert_eq(wrapper, wrapper);
}
