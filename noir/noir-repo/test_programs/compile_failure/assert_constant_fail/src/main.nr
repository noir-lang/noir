use dep::std::assert_constant;

fn main(x: Field) {
    foo(5, x);
}

fn foo(constant: Field, non_constant: Field) {
    assert_constant(constant);
    assert_constant(non_constant);
}
