// TODO: A global of the same name with the const generic does not work 
// at the moment. This will work once we switch over to the "elaborator"
// in the noirc_frontend
// global N = 1000;

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

    let foo = baz::<10>();
    assert(foo.inner == [1; 10]);

    let new_arr = update_arr([0, 1, 2, 3]);
    assert(new_arr[0] == 5);
}

fn id<let I: Field>(x: [Field; I]) -> [Field; I] {
    x
}

struct MyStruct<let S: u64> {
    data: [Field; S],
}

impl<let S: u64> MyStruct<S> {
    fn insert(mut self: Self, index: Field, elem: Field) -> Self {
        // Regression test for numeric generics on impls
        assert(index as u64 < S);

        self.data[index] = elem;
        self
    }
}

fn foo(mut s: MyStruct<2+1>) -> MyStruct<10/2-2> {
    s.data[0] = s.data[0] + 1;
    s
}

fn double<let N: u64>() -> u64 {
    // Used as an expression
    N * 2
}

fn double_numeric_generics_test() {
    // Example usage of a numeric generic arguments.
    assert(double::<9>() == 18);
    assert(double::<123>() == 246);
    assert(double::<7 + 8>() == 30);
}

// Used as a field of a struct
struct Foo<let N: u64> {
    inner: [u64; N],
}
fn baz<let N: u64>() -> Foo<N> {
    Foo { inner: [1; N] }
}
// Used in an impl
impl<let N: u64> Foo<N> {
    fn bar(self) -> u64 {
        N * self.inner[0]
    }
}

struct MyType {
    a: Field,
    b: Field,
    c: Field,
}

impl<let N: u64> Deserialize<N> for MyType {
    fn deserialize(fields: [Field; N]) -> Self {
        MyType { a: fields[0], b: fields[1], c: fields[2] }
    }
}

trait Deserialize<let N: u64> {
    fn deserialize(fields: [Field; N]) -> Self;
}

struct PublicStorage {}

impl PublicStorage {
    fn read<T, let N: u64>() -> T where T: Deserialize<N> {
        // Used as a type within a function body
        let mut fields: [Field; N] = [0; N];
        // Used a loop bound
        for i in 0..N {
            fields[i] = i as Field + 1;
        }
        T::deserialize(fields)
    }
}

// Used in the signature of a function
fn update_arr<let N: u8>(mut arr: [Field; N]) -> [Field; N] {
    arr[0] = 5;
    arr
}
