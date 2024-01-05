use dep::std::println;

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

    regression_side_effectual_array_ops(x, y);

    let q = x.push_back(foo_four);
    let foo_parent_one = FooParent { parent_arr: [0, 1, 2], foos: x };
    let foo_parent_two = FooParent { parent_arr: [3, 4, 5], foos: q };
    let mut foo_parents = [foo_parent_one];
    foo_parents = foo_parents.push_back(foo_parent_two);

    merge_nested_slices(foo_parents, y);
}

fn merge_nested_slices(mut foo_parents: [FooParent], y: Field) {
    let mut foo = Foo { a: 13, b: [14, 15, 16], bar: Bar { inner: [112, 113, 114] } };
    foo.a = 40;

    merge_nested_slice_if_no_push_front_asserts(foo_parents, y, foo);
    merge_nested_slice_if(foo_parents, y, foo);
    merge_nested_slice_else(foo_parents, y, foo);
    merge_nested_if_else_else(foo_parents, y, foo);
    merge_nested_if_else_if(foo_parents, y);
    merge_nested_two_ifs(foo_parents, y, foo);
    merge_nested_mutate_between_ifs(foo_parents, y, foo);
    merge_nested_push_then_pop(foo_parents, y, foo);
}

// This is a regression test as nested slices could possibly not be accurately tracked when certain
// followup asserts on the nested slice are missing.
// This method purely checks that we accurately track the slice of foo even when it is not asserted upon.
fn merge_nested_slice_if_no_push_front_asserts(mut foo_parents: [FooParent], y: Field, foo: Foo) {
    if y as u32 < 10 {
        assert(foo_parents[y - 2].foos.len() == 5);

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
        assert(foo_parents[y - 2].foos.len() == 6);
        assert(foo_parents[1].foos[5].a == 40);
        assert(foo_parents[1].foos[5].b[0] == 14);
        assert(foo_parents[1].foos[5].b[1] == 15);
        assert(foo_parents[1].foos[5].b[2] == 16);
        assert(foo_parents[1].foos[5].bar.inner == [112, 113, 114]);

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_front(foo);
        assert(foo_parents[y - 2].foos.len() == 7);
        // assert(foo_parents[y - 2].foos[0].a == 40);
    }
}

fn merge_nested_slice_if(mut foo_parents: [FooParent], y: Field, foo: Foo) {
    if y as u32 < 10 {
        assert(foo_parents[y - 2].foos.len() == 5);
        assert(foo_parents[1].foos[3].a == 10);
        assert(foo_parents[1].foos[3].b[0] == 11);
        assert(foo_parents[1].foos[3].b[1] == 12);
        assert(foo_parents[1].foos[3].b[2] == 23);
        assert(foo_parents[1].foos[3].bar.inner == [109, 110, 111]);

        foo_parents[y - 2].foos[y] = foo;
        assert(foo_parents[1].foos[3].a == 40);
        assert(foo_parents[1].foos[3].b[y - 3] == 14);
        assert(foo_parents[1].foos[3].b[y - 2] == 15);
        assert(foo_parents[1].foos[3].b[y - 1] == 16);
        assert(foo_parents[1].foos[3].bar.inner == [112, 113, 114]);

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
        assert(foo_parents[y - 2].foos.len() == 6);
        assert(foo_parents[1].foos[5].a == 40);
        assert(foo_parents[1].foos[5].b[0] == 14);
        assert(foo_parents[1].foos[5].b[1] == 15);
        assert(foo_parents[1].foos[5].b[2] == 16);
        assert(foo_parents[1].foos[5].bar.inner == [112, 113, 114]);

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_front(foo);
        assert(foo_parents[y - 2].foos.len() == 7);
        assert(foo_parents[1].foos[0].a == 40);
        assert(foo_parents[1].foos[0].b[0] == 14);
        assert(foo_parents[1].foos[0].b[1] == 15);
        assert(foo_parents[1].foos[0].b[2] == 16);
        assert(foo_parents[1].foos[0].bar.inner == [112, 113, 114]);

        let (popped_slice, last_foo) = foo_parents[y - 2].foos.pop_back();
        foo_parents[y - 2].foos = popped_slice;
        assert(foo_parents[y - 2].foos.len() == 6);
        assert(last_foo.a == 40);
        assert(last_foo.b[0] == 14);
        assert(last_foo.b[1] == 15);
        assert(last_foo.b[2] == 16);
        assert(last_foo.bar.inner == [112, 113, 114]);

        let (first_elem, rest_of_slice) = foo_parents[y - 2].foos.pop_front();
        foo_parents[y - 2].foos = rest_of_slice;
        assert(rest_of_slice.len() == 5);
        assert(foo_parents[y - 2].foos.len() == 5);
        assert(first_elem.a == 40);
        assert(first_elem.b[0] == 14);
        assert(first_elem.b[1] == 15);
        assert(first_elem.b[2] == 16);
        assert(first_elem.bar.inner == [112, 113, 114]);

        assert(foo_parents[1].foos[0].a == 1);
        assert(foo_parents[1].foos[0].b[0] == 2);
        assert(foo_parents[1].foos[0].b[1] == 3);
        assert(foo_parents[1].foos[0].b[2] == 20);
        assert(foo_parents[1].foos[0].bar.inner == [100, 101, 102]);

        assert(foo_parents[1].foos[1].a == 4);
        assert(foo_parents[1].foos[1].b[0] == 5);
        assert(foo_parents[1].foos[1].b[1] == 6);
        assert(foo_parents[1].foos[1].b[2] == 21);
        assert(foo_parents[1].foos[1].bar.inner == [103, 104, 105]);

        assert(foo_parents[1].foos[2].a == 7);
        assert(foo_parents[1].foos[2].b[0] == 8);
        assert(foo_parents[1].foos[2].b[1] == 9);
        assert(foo_parents[1].foos[2].b[2] == 22);
        assert(foo_parents[1].foos[2].bar.inner == [106, 107, 108]);

        assert(foo_parents[1].foos[3].a == 40);
        assert(foo_parents[1].foos[3].b[y - 3] == 14);
        assert(foo_parents[1].foos[3].b[y - 2] == 15);
        assert(foo_parents[1].foos[3].b[y - 1] == 16);
        assert(foo_parents[1].foos[3].bar.inner == [112, 113, 114]);

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.insert(y - 1, foo);
        assert(foo_parents[y - 2].foos.len() == 6);

        // Check that the value before the insert is unchanged
        assert(foo_parents[1].foos[1].a == 4);
        assert(foo_parents[1].foos[1].b[0] == 5);
        assert(foo_parents[1].foos[1].b[1] == 6);
        assert(foo_parents[1].foos[1].b[2] == 21);
        assert(foo_parents[1].foos[1].bar.inner == [103, 104, 105]);
        // Check that we have inserted the correct nested slice
        assert(foo_parents[1].foos[2].a == 40);
        assert(foo_parents[1].foos[2].b[0] == 14);
        assert(foo_parents[1].foos[2].b[1] == 15);
        assert(foo_parents[1].foos[2].b[2] == 16);
        assert(foo_parents[1].foos[2].bar.inner == [112, 113, 114]);
        // Check that tne nested slice has shifted following an insert
        assert(foo_parents[1].foos[3].a == 7);
        assert(foo_parents[1].foos[3].b[0] == 8);
        assert(foo_parents[1].foos[3].b[1] == 9);
        assert(foo_parents[1].foos[3].b[2] == 22);
        assert(foo_parents[1].foos[3].bar.inner == [106, 107, 108]);

        assert(foo_parents[1].foos[4].a == 40);
        assert(foo_parents[1].foos[4].b[y - 3] == 14);
        assert(foo_parents[1].foos[4].b[y - 2] == 15);
        assert(foo_parents[1].foos[4].b[y - 1] == 16);
        assert(foo_parents[1].foos[4].bar.inner == [112, 113, 114]);

        let (removed_slice, removed_elem) = foo_parents[y - 2].foos.remove(y);
        foo_parents[y - 2].foos = removed_slice;
        assert(foo_parents[y - 2].foos.len() == 5);
        // Check that the removed slice is what we expect
        assert(removed_elem.a == 7);
        assert(removed_elem.b[0] == 8);
        assert(removed_elem.b[1] == 9);
        assert(removed_elem.b[2] == 22);
        assert(removed_elem.bar.inner == [106, 107, 108]);
        // Check that the value before the remove is unchanged
        assert(foo_parents[1].foos[2].a == 40);
        assert(foo_parents[1].foos[2].b[0] == 14);
        assert(foo_parents[1].foos[2].b[1] == 15);
        assert(foo_parents[1].foos[2].b[2] == 16);
        assert(foo_parents[1].foos[2].bar.inner == [112, 113, 114]);
        // Check that the nested slice has shifted following a removal
        assert(foo_parents[1].foos[3].a == 40);
        assert(foo_parents[1].foos[3].b[y - 3] == 14);
        assert(foo_parents[1].foos[3].b[y - 2] == 15);
        assert(foo_parents[1].foos[3].b[y - 1] == 16);
        assert(foo_parents[1].foos[3].bar.inner == [112, 113, 114]);

        assert(foo_parents[1].foos[4].a == 10);
        assert(foo_parents[1].foos[4].b[y - 3] == 11);
        assert(foo_parents[1].foos[4].b[y - 2] == 12);
        assert(foo_parents[1].foos[4].b[y - 1] == 23);
        assert(foo_parents[1].foos[4].bar.inner == [109, 110, 111]);
    } else {
        foo_parents[y - 2].foos[y].a = 1000;

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
        assert(foo_parents[y - 2].foos.len() == 6);
        assert(foo_parents[1].foos[5].a == 40);
        assert(foo_parents[1].foos[5].b[0] == 14);
        assert(foo_parents[1].foos[5].b[2] == 16);
        assert(foo_parents[1].foos[5].b.len() == 3);
        assert(foo_parents[1].foos[5].bar.inner == [109, 110, 111]);
    }
    // Check that we end with what we expect from the if case
    assert(foo_parents[1].foos[3].a == 40);
    assert(foo_parents[1].foos[3].b[y - 3] == 14);
    assert(foo_parents[1].foos[3].b[y - 2] == 15);
    assert(foo_parents[1].foos[3].b[y - 1] == 16);
    assert(foo_parents[1].foos[3].bar.inner == [112, 113, 114]);

    assert(foo_parents[y - 2].foos.len() == 5);
    assert(foo_parents[1].foos[4].a == 10);
    assert(foo_parents[1].foos[4].b[y - 3] == 11);
    assert(foo_parents[1].foos[4].b[y - 2] == 12);
    assert(foo_parents[1].foos[4].b[y - 1] == 23);
    assert(foo_parents[1].foos[4].bar.inner == [109, 110, 111]);
}

fn merge_nested_slice_else(mut foo_parents: [FooParent], y: Field, foo: Foo) {
    if y as u32 > 10 {
        assert(foo_parents[y - 2].foos.len() == 5);
        foo_parents[y - 2].foos[y].a = 1000;
    } else {
        assert(foo_parents[y - 2].foos.len() == 5);
        assert(foo_parents[1].foos[3].a == 10);
        assert(foo_parents[1].foos[3].b[0] == 11);
        assert(foo_parents[1].foos[3].b[1] == 12);
        assert(foo_parents[1].foos[3].b[2] == 23);
        assert(foo_parents[1].foos[3].bar.inner == [109, 110, 111]);

        foo_parents[y - 2].foos[y] = foo;
        assert(foo_parents[1].foos[3].a == 40);
        assert(foo_parents[1].foos[3].b[y - 3] == 14);
        assert(foo_parents[1].foos[3].b[y - 2] == 15);
        assert(foo_parents[1].foos[3].b[y - 1] == 16);
        assert(foo_parents[1].foos[3].bar.inner == [112, 113, 114]);

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
        assert(foo_parents[y - 2].foos.len() == 6);
        assert(foo_parents[1].foos[5].a == 40);
        assert(foo_parents[1].foos[5].b[0] == 14);
        assert(foo_parents[1].foos[5].b[1] == 15);
        assert(foo_parents[1].foos[5].b[2] == 16);
        assert(foo_parents[1].foos[5].bar.inner == [112, 113, 114]);

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_front(foo);
        assert(foo_parents[y - 2].foos.len() == 7);
        assert(foo_parents[1].foos[0].a == 40);
        assert(foo_parents[1].foos[0].b[0] == 14);
        assert(foo_parents[1].foos[0].b[1] == 15);
        assert(foo_parents[1].foos[0].b[2] == 16);
        assert(foo_parents[1].foos[0].bar.inner == [112, 113, 114]);

        let (popped_slice, last_foo) = foo_parents[y - 2].foos.pop_back();
        foo_parents[y - 2].foos = popped_slice;
        assert(foo_parents[y - 2].foos.len() == 6);
        assert(last_foo.a == 40);
        assert(last_foo.b[0] == 14);
        assert(last_foo.b[1] == 15);
        assert(last_foo.b[2] == 16);
        assert(last_foo.bar.inner == [112, 113, 114]);

        let (first_elem, rest_of_slice) = foo_parents[y - 2].foos.pop_front();
        foo_parents[y - 2].foos = rest_of_slice;
        assert(rest_of_slice.len() == 5);
        assert(foo_parents[y - 2].foos.len() == 5);
        assert(first_elem.a == 40);
        assert(first_elem.b[0] == 14);
        assert(first_elem.b[1] == 15);
        assert(first_elem.b[2] == 16);
        assert(first_elem.bar.inner == [112, 113, 114]);

        assert(foo_parents[1].foos[0].a == 1);
        assert(foo_parents[1].foos[0].b[0] == 2);
        assert(foo_parents[1].foos[0].b[1] == 3);
        assert(foo_parents[1].foos[0].b[2] == 20);
        assert(foo_parents[1].foos[0].bar.inner == [100, 101, 102]);

        assert(foo_parents[1].foos[1].a == 4);
        assert(foo_parents[1].foos[1].b[0] == 5);
        assert(foo_parents[1].foos[1].b[1] == 6);
        assert(foo_parents[1].foos[1].b[2] == 21);
        assert(foo_parents[1].foos[1].bar.inner == [103, 104, 105]);

        assert(foo_parents[1].foos[2].a == 7);
        assert(foo_parents[1].foos[2].b[0] == 8);
        assert(foo_parents[1].foos[2].b[1] == 9);
        assert(foo_parents[1].foos[2].b[2] == 22);
        assert(foo_parents[1].foos[2].bar.inner == [106, 107, 108]);

        assert(foo_parents[1].foos[3].a == 40);
        assert(foo_parents[1].foos[3].b[y - 3] == 14);
        assert(foo_parents[1].foos[3].b[y - 2] == 15);
        assert(foo_parents[1].foos[3].b[y - 1] == 16);
        assert(foo_parents[1].foos[3].bar.inner == [112, 113, 114]);

        foo_parents[y - 2].foos = foo_parents[y - 2].foos.insert(y - 1, foo);
        assert(foo_parents[y - 2].foos.len() == 6);

        // Check that the value before the insert is unchanged
        assert(foo_parents[1].foos[1].a == 4);
        assert(foo_parents[1].foos[1].b[0] == 5);
        assert(foo_parents[1].foos[1].b[1] == 6);
        assert(foo_parents[1].foos[1].b[2] == 21);
        assert(foo_parents[1].foos[1].bar.inner == [103, 104, 105]);
        // Check that we have inserted the correct nested slice
        assert(foo_parents[1].foos[2].a == 40);
        assert(foo_parents[1].foos[2].b[0] == 14);
        assert(foo_parents[1].foos[2].b[1] == 15);
        assert(foo_parents[1].foos[2].b[2] == 16);
        assert(foo_parents[1].foos[2].bar.inner == [112, 113, 114]);
        // Check that tne nested slice has shifted following an insert
        assert(foo_parents[1].foos[3].a == 7);
        assert(foo_parents[1].foos[3].b[0] == 8);
        assert(foo_parents[1].foos[3].b[1] == 9);
        assert(foo_parents[1].foos[3].b[2] == 22);
        assert(foo_parents[1].foos[3].bar.inner == [106, 107, 108]);

        assert(foo_parents[1].foos[4].a == 40);
        assert(foo_parents[1].foos[4].b[y - 3] == 14);
        assert(foo_parents[1].foos[4].b[y - 2] == 15);
        assert(foo_parents[1].foos[4].b[y - 1] == 16);
        assert(foo_parents[1].foos[4].bar.inner == [112, 113, 114]);

        let (removed_slice, removed_elem) = foo_parents[y - 2].foos.remove(y);
        foo_parents[y - 2].foos = removed_slice;
        assert(foo_parents[y - 2].foos.len() == 5);
        // Check that the removed slice is what we expect
        assert(removed_elem.a == 7);
        assert(removed_elem.b[0] == 8);
        assert(removed_elem.b[1] == 9);
        assert(removed_elem.b[2] == 22);
        assert(removed_elem.bar.inner == [106, 107, 108]);
        // Check that the value before the remove is unchanged
        assert(foo_parents[1].foos[2].a == 40);
        assert(foo_parents[1].foos[2].b[0] == 14);
        assert(foo_parents[1].foos[2].b[1] == 15);
        assert(foo_parents[1].foos[2].b[2] == 16);
        assert(foo_parents[1].foos[2].bar.inner == [112, 113, 114]);
        // Check that the nested slice has shifted following a removal
        assert(foo_parents[1].foos[3].a == 40);
        assert(foo_parents[1].foos[3].b[y - 3] == 14);
        assert(foo_parents[1].foos[3].b[y - 2] == 15);
        assert(foo_parents[1].foos[3].b[y - 1] == 16);
        assert(foo_parents[1].foos[3].bar.inner == [112, 113, 114]);

        assert(foo_parents[1].foos[4].a == 10);
        assert(foo_parents[1].foos[4].b[y - 3] == 11);
        assert(foo_parents[1].foos[4].b[y - 2] == 12);
        assert(foo_parents[1].foos[4].b[y - 1] == 23);
        assert(foo_parents[1].foos[4].bar.inner == [109, 110, 111]);
    }
    // Check that we end with what we expect from the else case
    assert(foo_parents[1].foos[3].a == 40);
    assert(foo_parents[1].foos[3].b[y - 3] == 14);
    assert(foo_parents[1].foos[3].b[y - 2] == 15);
    assert(foo_parents[1].foos[3].b[y - 1] == 16);
    assert(foo_parents[1].foos[3].bar.inner == [112, 113, 114]);

    assert(foo_parents[y - 2].foos.len() == 5);
    assert(foo_parents[1].foos[4].a == 10);
    assert(foo_parents[1].foos[4].b[y - 3] == 11);
    assert(foo_parents[1].foos[4].b[y - 2] == 12);
    assert(foo_parents[1].foos[4].b[y - 1] == 23);
    assert(foo_parents[1].foos[4].bar.inner == [109, 110, 111]);
}

fn merge_nested_if_else_else(mut foo_parents: [FooParent], y: Field, foo: Foo) {
    if y as u32 < 10 {
        foo_parents[y - 2].foos[y].a = 1000;

        if y != 3 {
            foo_parents[y - 2].foos[y].a = foo_parents[y - 2].foos[y].a + 20;
        } else if y == 5 {
            assert(foo_parents[1].foos[3].a == 1022);
        } else {
            assert(foo_parents[1].foos[3].a == 1000);

            foo_parents[y - 2].foos[y].a = 20;
            foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
            assert(foo_parents[y - 2].foos.len() == 6);
            assert(foo_parents[1].foos[5].a == 40);
            assert(foo_parents[1].foos[5].b[0] == 14);
            assert(foo_parents[1].foos[5].b[1] == 15);
            assert(foo_parents[1].foos[5].b[2] == 16);
            assert(foo_parents[1].foos[5].bar.inner == [112, 113, 114]);
        }
    } else {
        // Shadowing used to ignor warning lines from LSP
        let (_, _) = foo_parents[y - 2].foos.pop_back();
        foo_parents[y - 2].foos[y].a = 0;
    }
    assert(foo_parents[y - 2].foos[y].a == 20);
    assert(foo_parents[1].foos[3].a == 20);

    let foos_len = foo_parents[y - 2].foos.len();
    assert(foos_len == 6);
    assert(foo_parents[1].foos[foos_len - 1].a == 40);
    assert(foo_parents[1].foos[foos_len - 1].b[0] == 14);
    assert(foo_parents[1].foos[foos_len - 1].b[1] == 15);
    assert(foo_parents[1].foos[foos_len - 1].b[2] == 16);
    assert(foo_parents[1].foos[foos_len - 1].bar.inner == [112, 113, 114]);

    foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    let foos_len = foo_parents[y - 2].foos.len();
    assert(foos_len == 7);
    assert(foo_parents[1].foos[6].a == 40);
    assert(foo_parents[1].foos[6].b[0] == 14);
    assert(foo_parents[1].foos[6].b[1] == 15);
    assert(foo_parents[1].foos[6].b[2] == 16);
    assert(foo_parents[1].foos[6].bar.inner == [112, 113, 114]);
}

fn merge_nested_if_else_if(mut foo_parents: [FooParent], y: Field) {
    if y as u32 < 10 {
        foo_parents[y - 2].foos[y].a = 1000;

        if y == 10 {
            foo_parents[y - 2].foos[y].a = foo_parents[y - 2].foos[y].a + 20;
        } else {
            assert(foo_parents[y - 2].foos[y].a == 1000);
            if y == 3 {
                assert(foo_parents[y - 2].foos[y].a == 1000);

                foo_parents[y - 2].foos[y].a = 5;
            }
        }
    } else {
        foo_parents[y - 2].foos[y].a = 999;
    }
    assert(foo_parents[y - 2].foos[y].a == 5);
}

// Simple regression for side effectual array operations
fn regression_side_effectual_array_ops(mut foos: [Foo], y: Field) {
    if y as u32 < 10 {
        foos[y].a = 1000;
        // If array ops (get/set) are labelled as pure instructions 
        // missing an assert here would cause the same assert in the
        // else-case of the inner if statement to fail
        // assert(foos[y].a == 1000);

        if y == 10 {
            foos[y].a = foos[y].a + 20;
        } else {
            assert(foos[y].a == 1000);
            foos[y].a = 5;
        }
    } else {
        foos[y].a = 999;
    }
    assert(foos[y].a == 5);
}

fn merge_nested_two_ifs(mut foo_parents: [FooParent], y: Field, mut foo: Foo) {
    assert(foo_parents[1].foos.len() == 5);
    if y as u32 > 10 {
        assert(foo_parents[y - 2].foos[y].a == 999);
        foo_parents[y - 2].foos[y].a = 2;
    } else {
        assert(foo_parents[y - 2].foos[y].a == 10);
        foo_parents[y - 2].foos[y].a = foo_parents[y - 2].foos[y].a - 2;
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    }

    assert(foo_parents[1].foos.len() == 6);
    assert(foo_parents[1].foos[5].a == 40);
    assert(foo_parents[1].foos[5].b[0] == 14);
    assert(foo_parents[1].foos[5].b[1] == 15);
    assert(foo_parents[1].foos[5].b[2] == 16);
    assert(foo_parents[1].foos[5].bar.inner == [112, 113, 114]);

    if y == 20 {
        foo.a = 20;
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    }

    foo.a = 15;
    foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);

    assert(foo_parents[1].foos.len() == 7);
    assert(foo_parents[1].foos[6].a == 15);
    assert(foo_parents[1].foos[6].b[0] == 14);
    assert(foo_parents[1].foos[6].b[1] == 15);
    assert(foo_parents[1].foos[6].b[2] == 16);
    assert(foo_parents[1].foos[6].bar.inner == [112, 113, 114]);

    foo.a = 20;
    foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    assert(foo_parents[1].foos.len() == 8);
    assert(foo_parents[1].foos[7].a == 20);
}

fn merge_nested_mutate_between_ifs(mut foo_parents: [FooParent], y: Field, mut foo: Foo) {
    if y == 3 {
        foo_parents[y - 2].foos[y].a = 50;
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    } else {
        foo_parents[y - 2].foos[y].a = foo_parents[y - 2].foos[y].a - 2;
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    }

    foo.a = 30;
    foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    assert(foo_parents[1].foos.len() == 8);
    assert(foo_parents[1].foos[7].a == 30);

    if y == 20 {
        foo.a = 20;
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    }

    foo.a = 15;
    foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);

    if y != 20 {
        foo.a = 50;
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    }

    foo.a = 60;
    foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    let foos_len = foo_parents[1].foos.len();
    assert(foos_len == 11);
    assert(foo_parents[1].foos[3].a == 50);
    assert(foo_parents[1].foos[foos_len - 1].a == 60);
    assert(foo_parents[1].foos[foos_len - 2].a == 50);
    assert(foo_parents[1].foos[foos_len - 3].a == 15);
    assert(foo_parents[1].foos[foos_len - 4].a == 30);
}

fn merge_nested_push_then_pop(mut foo_parents: [FooParent], y: Field, mut foo: Foo) {
    if y == 3 {
        foo_parents[y - 2].foos[y].a = 50;
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
        assert(foo_parents[1].foos.len() == 7);

        let (popped_slice, elem) = foo_parents[y - 2].foos.pop_back();
        assert(foo_parents[1].foos.len() == 7);
        assert(elem.a == foo.a);
        assert(elem.b[0] == foo.b[0]);
        assert(elem.b[1] == foo.b[1]);
        assert(elem.b[2] == foo.b[2]);
        assert(elem.bar.inner == foo.bar.inner);
        foo_parents[y - 2].foos = popped_slice;
    } else {
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    }

    foo.a = 30;
    foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    assert(foo_parents[1].foos.len() == 7);
    assert(foo_parents[1].foos[6].a == 30);

    if y == 20 {
        foo_parents[y - 2].foos = foo_parents[y - 2].foos.push_back(foo);
    }

    let (popped_slice, elem) = foo_parents[y - 2].foos.pop_back();
    assert(elem.a == 30);

    let (_, elem) = popped_slice.pop_back();
    assert(elem.a == 40);
}

