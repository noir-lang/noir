// Regression that a global of the same name does not trigger a duplicate definition error
global N: u32 = 1000;

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
    assert(foo.data == [1; 10]);
}

// Used in the signature of a function
fn id<let I: u32>(x: [Field; I]) -> [Field; I] {
    x
}

// Used as a field of a struct
struct MyStruct<let S: u32> {
    data: [Field; S],
}

// Used in an impl
impl<let S: u32> MyStruct<S> {
    fn insert(mut self: Self, index: Field, elem: Field) -> Self {
        // Regression test for numeric generics on impls
        assert(index as u32 < S);

        self.data[index] = elem;
        self
    }
}

fn foo(mut s: MyStruct<2 + 1>) -> MyStruct<10 / 2 - 2> {
    s.data[0] = s.data[0] + 1;
    s
}

fn baz<let N: u32>() -> MyStruct<N> {
    MyStruct { data: [1; N] }
}

fn double<let N: u32>() -> u32 {
    // Used as an expression
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

impl<let N: u32> Deserialize<N> for MyType {
    fn deserialize(fields: [Field; N]) -> Self {
        MyType { a: fields[0], b: fields[1], c: fields[2] }
    }
}

trait Deserialize<let N: u32> {
    fn deserialize(fields: [Field; N]) -> Self;
}

pub struct PublicStorage {}

impl PublicStorage {
    fn read<T, let N: u32>() -> T
    where
        T: Deserialize<N>,
    {
        // Used as a type within a function body
        let mut fields: [Field; N] = [0; N];
        // Used a loop bound
        for i in 0..N {
            fields[i] = i as Field + 1;
        }
        T::deserialize(fields)
    }
}

// Check that we can thread numeric generics into nested structs
// and also that we can handle nested structs with numeric generics
// which are declared after the parent struct
pub struct NestedNumeric<let N: u32> {
    a: Field,
    b: InnerNumeric<N>,
}
pub struct InnerNumeric<let N: u32> {
    inner: [u32; N],
}
