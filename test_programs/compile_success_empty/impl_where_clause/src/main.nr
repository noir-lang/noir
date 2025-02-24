struct MyStruct<T> {
    a: u32,
    b: T,
}

struct InnerStruct {
    a: Field,
    b: Field,
}

trait MyEq {
    fn my_eq(self, other: Self) -> bool;
}

impl MyEq for InnerStruct {
    fn my_eq(self, other: InnerStruct) -> bool {
        (self.a == other.a) & (self.b == other.b)
    }
}

impl<T> MyStruct<T>
where
    T: MyEq,
{
    fn my_eq(self, other: Self) -> bool {
        (self.a == other.a) & self.b.my_eq(other.b)
    }
}

fn main() {
    let inner = InnerStruct { a: 1, b: 2 };
    let my_struct = MyStruct { a: 5, b: inner };
    assert(my_struct.my_eq(my_struct));

    let mut my_struct_new = MyStruct { a: 5, b: InnerStruct { a: 10, b: 15 } };
    assert(my_struct_new.my_eq(my_struct_new));
}
