fn main(x : Field, y : pub Field) {
    let q = foo(x);
    constrain q != y;
}

fn foo(x : pub Field) -> Field {
    x
}
    