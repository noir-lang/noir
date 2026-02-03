trait Serialize<let N: u32> {
    fn serialize(self) -> [Field; N];
}

trait ToField {
    fn to_field(self) -> Field;
}

fn process_array<let N: u32>(array: [Field; N]) -> Field {
    array[0]
}

fn serialize_thing<A, let N: u32>(thing: A) -> [Field; N]
where
    A: Serialize<N>,
{
    thing.serialize()
}

struct MyType<T> {
    a: T,
    b: T,
}

impl<T> Serialize<2> for MyType<T>
where
    T: ToField,
{
    fn serialize(self) -> [Field; 2] {
        [self.a.to_field(), self.b.to_field()]
    }
}

impl<T> MyType<T> {
    fn do_thing_with_serialization_with_extra_steps(self) -> Field {
        process_array(serialize_thing(self))
    }
}

fn main() {
    let _ = MyType { a: 1, b: 1 }; // silence MyType never constructed warning
}
