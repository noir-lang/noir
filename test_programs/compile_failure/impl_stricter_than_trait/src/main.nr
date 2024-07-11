trait Serialize<let N: u32> {
    fn serialize(self) -> [Field; N];
}

trait ToField {
    fn to_field(self) -> Field;
}

fn process_array<let N: u32>(array: [Field; N]) -> Field {
    array[0]
}

fn serialize_thing<A, let N: u32>(thing: A) -> [Field; N] where A: Serialize<N> {
    thing.serialize()
}

struct MyType<T> {
    a: T,
    b: T,
}

impl<T> Serialize<2> for MyType<T> {
    fn serialize(self) -> [Field; 2] where T: ToField {
        [ self.a.to_field(), self.b.to_field() ]
    }
}

impl<T> MyType<T> {
    fn do_thing_with_serialization_with_extra_steps(self) -> Field {
        process_array(serialize_thing(self))
    }
}

trait MyTrait { }

// // Object type differs
// trait Foo<T> {
//     fn foo<U>() where T: MyTrait;
//     // fn foo<U>();
// }

// impl<A> Foo<A> for () {
//     fn foo<B>() where B: MyTrait {}
//     // fn foo<B>() where A: Default {}
//     // fn foo<B>() {}
// }

struct MyStruct<T> {
    inner: T,
}

trait OtherTrait {}

trait Bar<T> {
    fn bar<U>() where MyStruct<T>: MyTrait;
}

impl<A> Bar<A> for () {
    fn bar<B>() where MyStruct<A>: OtherTrait {}
}