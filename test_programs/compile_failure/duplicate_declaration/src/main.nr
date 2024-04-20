// Duplicate functions should not compile
fn hello(x: field) -> field {
    x
}

fn hello(x: field) -> field {
    x
}
