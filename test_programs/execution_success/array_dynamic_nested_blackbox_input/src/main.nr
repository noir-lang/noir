struct Bar {
    inner: [u8; 3],
}

struct Foo {
    a: Field,
    b: [Field; 3],
    bar: Bar,
}

fn main(mut x: [Foo; 3], y: pub Field, hash_result: pub [u8; 32]) {
    // Simple dynamic array set for entire inner most array
    x[y - 1].bar.inner = [106, 107, 10];
    let mut hash_input = x[y - 1].bar.inner;
    // Make sure that we are passing a dynamic array to the black box function call
    // by setting the array using a dynamic index here
    hash_input[y - 1] = 0;
    let hash = dep::std::hash::sha256(hash_input);
    assert_eq(hash, hash_result);
}
