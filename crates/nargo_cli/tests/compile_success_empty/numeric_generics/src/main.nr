
fn main() {
    let a = id([1, 2]);
    let b = id([1, 2, 3]);

    let itWorks1 = MyStruct { data: a };
    assert_eq(itWorks1.data[1], 2);
    let itWorks2 = MyStruct { data: b };
    assert_eq(itWorks2.data[1], 2);

    let c = [1, 2];
    let itAlsoWorks = MyStruct { data: c };
    assert_eq(itAlsoWorks.data[1], 2);

    assert_eq(foo(itWorks2).data[0], itWorks2.data[0] + 1);
}

fn id<I>(x: [Field; I]) -> [Field; I] {
    x
}

struct MyStruct<S> {
    data: [Field; S],
}

impl<S> MyStruct<S> {
    fn insert(mut self: Self, index: Field, elem: Field) -> Self {
        // Regression test for numeric generics on impls
        assert(index as u64 < S as u64);

        self.data[index] = elem;
        self
    }
}

fn foo(mut s: MyStruct<2+1>) -> MyStruct<10/2-2> {
    s.data[0] = s.data[0] + 1;
    s
}

