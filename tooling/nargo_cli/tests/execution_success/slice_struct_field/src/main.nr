use dep::std::collections::vec::Vec;

struct Bar {
    inner: [Field; 3],
}

struct Foo {
    a: Field,
    b: Vec<Field>,
    bar: Bar,
}



fn main(y : pub Field) {
    // let mut b_one = [2, 3];
    // b_one = b_one.push_back(20);
    let b_one = Vec::from_slice([2, 3, 20]);
    let foo_one = Foo { a: 1, b: b_one, bar: Bar { inner: [100, 101, 102] } };
    // let mut b_two = [5, 6];
    // b_two = b_two.push_back(21);
    let b_two = Vec::from_slice([5, 6, 21]);
    let foo_two = Foo { a: 14, b: b_two, bar: Bar { inner: [103, 104, 105] } };

    let mut x = [foo_one];
    x = x.push_back(foo_two);

    assert(x[y - 3].a == 1);
    assert(x[y - 3].b.slice == b_one.slice);
}