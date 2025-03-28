use std::append::Append;

fn main() {
    assert_true!();
}

comptime fn assert_true() -> Quoted {
    let first = quote { assert( };
    let second = quote { true); };
    first.append(second)
}
