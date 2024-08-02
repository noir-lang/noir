#[derive_via(derive_do_nothing)]
trait DoNothing {
    fn do_nothing(self);
}

#[derive(DoNothing)]
struct MyStruct { my_field: u32 }

comptime fn derive_do_nothing(s: StructDefinition) -> Quoted {
    let typ = s.as_type();
    let generics = s.generics().map(|g| quote { $g }).join(quote {,});
    quote {
        impl<$generics> DoNothing for $typ {
            fn do_nothing(_self: Self) {
                // Traits can't tell us what to do
                println("something");
            }
        }
    }
}

// Test stdlib derive fns & multiple traits
#[derive(Eq, Default)]
struct MyOtherStruct<A, B> {
    field1: A,
    field2: B,
    field3: MyOtherOtherStruct<B>,
}

#[derive(Eq, Default)]
struct MyOtherOtherStruct<T> {
    x: T,
}

fn main() {
    let s = MyStruct { my_field: 1 };
    s.do_nothing();

    let o: MyOtherStruct<Field, u32> = MyOtherStruct::default();
    assert_eq(o, o);

    let o: MyOtherStruct<u8, [str<2>]> = MyOtherStruct::default();
    assert_eq(o, o);
}
