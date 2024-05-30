fn main() {
    let a = id([1, 2]);
    let b = id([1, 2, 3]);

    let itWorks1 = MyStruct { data: a };
    assert(itWorks1.data[1] == 2);
    let itWorks2 = MyStruct { data: b };
    assert(itWorks2.data[1] == 2);

    let c = [1, 2];
    let itAlsoWorks = MyStruct { data: c };
    assert(itAlsoWorks.data[1] == 2);

    assert(foo(itWorks2).data[0] == itWorks2.data[0] + 1);

    double_numeric_generics_test();

    let my_type: MyType<20> = PublicStorage::read();
    assert(my_type.fields.len() == 20);
    for field in my_type.fields {
        assert(field == 1);
    }
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

fn double<let N: u64>() -> u64 {
    N * 2
}

fn double_numeric_generics_test() {
    // Example usage of a numeric generic arguments.
    assert(double::<9>() == 18);
    assert(double::<123>() == 246);
    assert(double::<7 + 8>() == 30);
}

struct MyType<N> {
    fields: [Field; N]
}

impl<N> Deserialize<N> for MyType<N> {
    fn deserialize(fields: [Field; N]) -> Self {
        let mut new_fields = [0; N];
        for i in 0..N {
            new_fields[i] = fields[i];
        }
        MyType { fields: new_fields }
    }
}

trait Deserialize<N> {
    fn deserialize(fields: [Field; N]) -> Self;
}

fn storage_read() -> Field {
    1
}

struct PublicStorage {}

impl PublicStorage {
    fn read<T, let N: u64>() -> T where T: Deserialize<N> {
        let mut fields = [0; N];
        for i in 0..N {
            fields[i] = storage_read();
        }
        T::deserialize(fields)
    }
}
