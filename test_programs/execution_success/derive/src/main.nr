#[derive_via(derive_do_nothing)]
trait DoNothing {
    fn do_nothing(self);
}

#[derive(DoNothing)]
struct MyStruct { my_field: u32 }

comptime fn derive_do_nothing(s: StructDefinition) -> Quoted {
    let typ = s.as_type();
    let generics = s.generics().join(quote {,});
    quote {
        impl<$generics> DoNothing for $typ {
            fn do_nothing(_self: Self) {
                // Traits can't tell us what to do
                println("something");
            }
        }
    }
}

fn main() {
    let s = MyStruct { my_field: 1 };
    s.do_nothing();
}
