use dep::std::assert_constant;

fn main(x: field) {
    foo(5, x);
}

fn foo(constant: field, non_constant: field) {
    assert_constant(constant);
    assert_constant(non_constant);
}
