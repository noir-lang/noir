// Duplicate functions should not compile
fn hello(x : Field) -> Field {
    x
}

fn hello(x : Field) -> Field {
    x
}