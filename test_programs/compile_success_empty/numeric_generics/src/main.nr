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

    let my_type = PublicStorage::read::<MyType, 3>();
    assert(my_type.a == 1);
    assert(my_type.b == 2);
    assert(my_type.c == 3);
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

struct MyType {
    a: Field,
    b: Field,
    c: Field,
}

impl<N> Deserialize<N> for MyType {
    fn deserialize(fields: [Field; N]) -> Self {
        MyType { a: fields[0], b: fields[1], c: fields[2] }
    }
}

trait Deserialize<N> {
    fn deserialize(fields: [Field; N]) -> Self;
}

struct PublicStorage {}

impl PublicStorage {
    fn read<T, let N: u64>() -> T where T: Deserialize<N> {
        let mut fields = [0; N];
        for i in 0..N {
            // TODO: Removing this cast I get an error as expected as deserialzie
            // expects an array of fields not u64, but the error could be improved.
            // Getting the following: `Expected type [Field; u64], found type [u64; u64]`
            fields[i] = i as Field + 1;
        }
        T::deserialize(fields)
    }
}


