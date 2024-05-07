fn main(x: pub Field, y: pub Field) {
    foo(x, y);
}

fn foo(x: Field, y: Field) {
    bar(x, y);
}

fn bar(x: Field, y: Field) {
    assert(x == y + 1);
}
