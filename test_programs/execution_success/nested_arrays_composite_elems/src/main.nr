fn main(x: u32) {
    let mut parent_array = [Parent::default(); 2];

    parent_array[x].foo.c.a = 10;
    // println(parent_array);
    assert(parent_array[0].foo.c.a == 10);
}

struct Baz {
    a: u32,
    b: u32,
}

struct Bar {
    a: u8,
    b: u8,
}

struct InnerFoo {
    a: Field,
    // array: [u32; 2],
}

struct Foo {
    a: u32,
    b: u32,
    c: InnerFoo,
    d: u8 
}

struct Parent {
    foo: Foo,
    bar: Bar,
    baz: Baz,
}

impl Default for Baz {
    fn default() -> Self {
        Baz { a: 0, b: 0 }
    }
}

impl Default for Bar {
    fn default() -> Self {
        Bar { a: 0, b: 0 }
    }
}

impl Default for InnerFoo {
    fn default() -> Self {
        // InnerFoo { a: 0, array: [0; 2] }
        InnerFoo { a: 0 }
    }
}

impl Default for Foo {
    fn default() -> Self {
        Foo { a: 0, b: 0, c: InnerFoo::default(), d: 0 }
    }
}

impl Default for Parent {
    fn default() -> Self {
        Parent { foo: Foo::default(), bar: Bar::default(), baz: Baz::default() }
    }
}