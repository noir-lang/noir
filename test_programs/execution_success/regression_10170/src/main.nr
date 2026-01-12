struct foo {
    a: u32,
    b: u32,
    c: [Field; 3],
}
fn main(cond: bool) {
    // Issue 10170
    let mut vector = [0, 1].as_vector();
    if cond {
        vector = vector.push_back(10);
    }
    for i in 0..2 {
        vector = vector.push_back(i);
        assert_eq(vector[vector.len() - 1], i);
        let (new_vector, _) = vector.pop_back();
        vector = new_vector;
    }

    // Same, but with nested vectors
    let mut vector =
        [foo { a: 0, b: 0, c: [0, 0, 0] }, foo { a: 1, b: 1, c: [0, 0, 0] }].as_vector();
    if cond {
        vector = vector.push_back(foo { a: 10, b: 10, c: [0, 0, 0] });
    }
    for i in 0..2 {
        vector = vector.push_back(foo { a: i, b: i, c: [0, 0, 0] });
        assert_eq(vector[vector.len() - 1].a, i);
        let (new_vector, _) = vector.pop_back();
        vector = new_vector;
    }

    // push after pop
    let mut vector = [0, 1].as_vector();
    if !cond {
        vector = vector.push_back(10);
    }
    let (new_vector, _) = vector.pop_back();
    vector = new_vector.push_back(11);
    assert_eq(vector[vector.len() - 1], 11);
}
