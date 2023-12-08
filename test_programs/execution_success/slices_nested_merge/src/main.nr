struct FooParent {
    parent_arr: [Field; 3],
    foos: [Foo],
}

struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: [Field],
    bar: Bar,
}

fn main(y: pub Field) {
    let mut b_one = [2, 3, 20];
    b_one = b_one.push_back(20);
    let foo_one = Foo { a: 1, b: b_one, bar: Bar { inner: [100, 101, 102] } };

    let mut b_two = [5, 6, 21];
    b_two = b_two.push_back(21);
    let foo_two = Foo { a: 4, b: b_two, bar: Bar { inner: [103, 104, 105] } };

    let foo_three = Foo { a: 7, b: [8, 9, 22], bar: Bar { inner: [106, 107, 108] } };
    let mut foo_four = Foo { a: 10, b: [11, 12, 23], bar: Bar { inner: [109, 110, 111] } };

    let mut x = [foo_one, foo_two];
    x = x.push_back(foo_three);
    x = x.push_back(foo_four);

    let q = x.push_back(foo_four);
    let foo_parent_one = FooParent { parent_arr: [0, 1, 2], foos: x };
    let foo_parent_two = FooParent { parent_arr: [3, 4, 5], foos: q };
    let mut foo_parents = [foo_parent_one];
    foo_parents = foo_parents.push_back(foo_parent_two);

    merge_nested_slices(foo_parents, y);
}

fn merge_nested_slices(mut foo_parents: [FooParent], y: Field) {
    let mut foo = Foo { a: 13, b: [14, 15, 16], bar: Bar { inner: [109, 110, 111] } };
    foo.a = 40;

    merge_nested_slice_if(foo_parents, y, foo);
    // let slice = 
}

fn merge_nested_slice_if(mut foo_parents: [FooParent], y: Field, foo: Foo) {
    if y as u32 < 10 {
        // dep::std::println(foo.a);
        assert(foo_parents[y - 2].foos.len() == 5);
        // TODO: this causes a panic and is an entirely separate bug
        assert(foo.bar.inner == [109, 110, 111]);
        // assert(foo_parents[1].foos[3].a == 10);
        // assert(foo_parents[1].foos[3].b[0] == 11);
        // assert(foo_parents[1].foos[3].b[1] == 12);
        // assert(foo_parents[1].foos[3].b[2] == 23);
        // assert(foo_parents[1].foos[3].bar.inner == [109, 110, 111]);

        // foo_parents[y - 2].foos[y] = foo;

        // assert(foo_parents[1].foos[3].a == 10);
        // assert(foo_parents[1].foos[3].b[0] == 11);
        // assert(foo_parents[1].foos[3].b[1] == 12);
        // assert(foo_parents[1].foos[3].b[2] == 23);
        // assert(foo_parents[1].foos[3].bar.inner == [109, 110, 111]);
    }   
}