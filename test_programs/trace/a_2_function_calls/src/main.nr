fn bar(y: Field) {
    assert(y != 0);
}

fn foo(x: Field) {
    bar(x)
}

fn main(x: Field, y: pub Field) {
    foo(x);
    foo(y);
    assert(x != y);
}
