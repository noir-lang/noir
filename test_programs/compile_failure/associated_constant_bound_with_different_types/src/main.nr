trait TraitWithAssociatedConstant {
    let N: u32;

    fn foo(_: Self) -> bool {
        true
    }

    fn make_array(self) -> [Field; Self::N];
}

struct Foo {}

impl TraitWithAssociatedConstant for Foo {
    let N: u32 = 5;

    fn make_array(self) -> [Field; Self::N] {
        std::mem::zeroed()
    }
}

struct Wrapper<T> {
    inner: T,
}

impl<U> std::cmp::Eq for Wrapper<U>
where
    U: TraitWithAssociatedConstant,
{

    fn eq(self, _other: Self) -> bool {
        let _array1: [Field; 5] = self.inner.make_array();
        let _array2: [Field; 6] = self.inner.make_array(); // get the same method to return an array of a different size
        self.inner.foo()
    }
}

fn main() {
    let wrapper = Wrapper { inner: Foo {} };
    assert_eq(wrapper, wrapper);
}
