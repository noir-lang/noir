contract Foo {
    use crate::times_10;

    fn double(x: field) -> pub field {
        x * 2
    }
    fn triple(x: field) -> pub field {
        x * 3
    }
    fn times_40(x: field) -> pub field {
        times_10(x) * 4
    }
}

#[fold]
fn times_10(x: field) -> field {
    x * 10
}
