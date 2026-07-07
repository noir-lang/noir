// Regression test for https://github.com/noir-lang/noir/issues/1121.
// Dynamically reading from a non-homogenous vector initializes that vector's element-type-sizes
// helper block; a subsequent `insert` on the same vector must not reinitialize that block.
fn main(x: u32, y: u32) {
    let arr: [(Field, [Field; 2]); 3] = [(1, [2, 3]), (4, [5, 6]), (7, [8, 9])];
    let mut v: [(Field, [Field; 2])] = arr.as_vector();

    let elem = v[y];
    v = v.insert(x, (10, [11, 12]));

    assert_eq(v.len(), 4);
    assert(elem.0 != 999);
}
