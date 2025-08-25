fn main(x: u32) {
    let mut parent_array = [Parent::default(); 2];

    parent_array[x].foo.c.a = 10;
    assert(parent_array[0].foo.c.a == 10);

    parent_array[x + 1].bar = Bar { a: 1, b: 2, array: [3, 4] };
    assert(parent_array[1].bar.a == 1);
    assert(parent_array[1].bar.b == 2);

    // Test checking the entire array
    assert(parent_array[1].bar.array == [3, 4]);
    // Test checking the array through indexing
    assert(parent_array[1].bar.array[0] == 3);
    assert(parent_array[1].bar.array[1] == 4);

    parent_array[x].foo.c.array[x] = 20;
    parent_array[x].foo.c.array[x + 1] = 30;
    assert(parent_array[0].foo.c.array[0] == 20);
    assert(parent_array[0].foo.c.array[1] == 30);

    parent_array[x + 1].qux = Qux {
        array: [
            InnerFoo {
                a: 100,
                array: [101, 102],
                bar: Bar::default(),
                bar_array: [Bar::default(); 2],
            },
            InnerFoo {
                a: 200,
                array: [201, 202],
                bar: Bar::default(),
                bar_array: [Bar::default(); 2],
            },
        ],
        a: 500,
    };
    assert(parent_array[1].qux.array[0].a == 100);
    assert(parent_array[1].qux.a == 500);

    // Test multiple array accesses between tuple accesses
    parent_array[x].qux.array[x].bar_array[x].a = 99;
    assert(parent_array[x].qux.array[x].bar_array[x].a == 99);

    let mut parent_array_wrapper = ParentsWrapper { inner: parent_array };
    parent_array_wrapper.inner[x].qux.a = 100;
    assert(parent_array_wrapper.inner[0].qux.a == 100);

    parent_array_wrapper.inner[x].qux.array[x].array[x] = 301;
    assert(parent_array_wrapper.inner[0].qux.array[0].array[0] == 301);

    // Test accessing a nested array where the value assignment
    // starts with a member access
    parent_array_wrapper.inner[x].foo.c.bar.array[x] = 1000;
    assert(parent_array_wrapper.inner[0].foo.c.bar.array[0] == 1000);

    parent_array_wrapper.set_foo(x);
}

struct Baz {
    a: u32,
    b: u32,
}

struct Bar {
    a: u8,
    b: u8,
    array: [Field; 2],
}

struct InnerFoo {
    a: Field,
    array: [u32; 2],
    bar: Bar,
    bar_array: [Bar; 2],
}

struct Foo {
    a: u32,
    b: u32,
    c: InnerFoo,
    d: u8,
}

// Struct with array and primitive fields where primitives come after
// the array fields. We want to index this inside of a composite array
// to test whether we are updating the index appropriately following an array.
struct Qux {
    array: [InnerFoo; 2],
    a: u32,
}

struct Parent {
    foo: Foo,
    bar: Bar,
    baz: Baz,
    qux: Qux,
}

struct ParentsWrapper {
    inner: [Parent; 2],
}

impl ParentsWrapper {
    fn set_foo(&mut self, index: u32) {
        self.inner[index].foo = Foo {
            a: 1,
            b: 2,
            c: InnerFoo {
                a: 3,
                array: [4, 5],
                bar: Bar::default(),
                bar_array: [Bar::default(); 2],
            },
            d: 6,
        }
    }
}

impl Default for Baz {
    fn default() -> Self {
        Baz { a: 0, b: 0 }
    }
}

impl Default for Bar {
    fn default() -> Self {
        Bar { a: 0, b: 0, array: [0; 2] }
    }
}

impl Default for InnerFoo {
    fn default() -> Self {
        InnerFoo { a: 0, array: [0; 2], bar: Bar::default(), bar_array: [Bar::default(); 2] }
    }
}

impl Default for Qux {
    fn default() -> Self {
        Qux { array: [InnerFoo::default(); 2], a: 0 }
    }
}

impl Default for Foo {
    fn default() -> Self {
        Foo { a: 0, b: 0, c: InnerFoo::default(), d: 0 }
    }
}

impl Default for Parent {
    fn default() -> Self {
        Parent {
            foo: Foo::default(),
            bar: Bar::default(),
            baz: Baz::default(),
            qux: Qux::default(),
        }
    }
}
