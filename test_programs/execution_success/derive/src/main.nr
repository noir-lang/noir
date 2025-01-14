use std::hash::Hash;

#[derive_via(derive_do_nothing)]
trait DoNothing {
    fn do_nothing(self);
}

#[derive(DoNothing)]
struct MyStruct {
    my_field: u32,
}

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
// - We can derive Ord and Hash even though std::cmp::Ordering and std::hash::Hasher aren't imported
// - We need to define MyOtherOtherStruct first since MyOtherStruct references it as a field and
//   attributes are run in reading order. If it were defined afterward, the derived Eq impl for MyOtherStruct
//   would error that MyOtherOtherStruct doesn't (yet) implement Eq.
#[derive(Eq, Default, Hash, Ord)]
struct MyOtherOtherStruct<T> {
    x: T,
}

#[derive(Eq, Default, Hash, Ord)]
struct MyOtherStruct<A, B> {
    field1: A,
    field2: B,
    field3: MyOtherOtherStruct<B>,
}

#[derive(Eq, Default, Hash, Ord)]
struct EmptyStruct {}

fn main() {
    let s = MyStruct { my_field: 1 };
    s.do_nothing();

    let o: MyOtherStruct<Field, u32> = MyOtherStruct::default();
    assert_eq(o, o);

    let o: MyOtherStruct<u8, [str<2>]> = MyOtherStruct::default();
    assert_eq(o, o);

    // Field & str<2> above don't implement Ord
    let o1 = MyOtherStruct {
        field1: 12 as u32,
        field2: 24 as i8,
        field3: MyOtherOtherStruct { x: 54 as i8 },
    };
    let o2 = MyOtherStruct {
        field1: 12 as u32,
        field2: 24 as i8,
        field3: MyOtherOtherStruct { x: 55 as i8 },
    };
    assert(o1 < o2);

    let mut hasher = TestHasher { result: 0 };
    o1.hash(&mut hasher);
    assert_eq(hasher.finish(), 12 + 24 + 54);

    let empty = EmptyStruct {};
    assert_eq(empty, empty);
}

struct TestHasher {
    result: Field,
}

impl std::hash::Hasher for TestHasher {
    fn finish(self) -> Field {
        self.result
    }

    fn write(&mut self, input: Field) {
        self.result += input;
    }
}
