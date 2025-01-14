comptime mut global COUNTER: Field = 0;

fn main() {
    comptime { increment() };
    comptime { increment() };

    assert_eq(get_counter(), 2);
}

fn get_counter() -> Field {
    COUNTER
}

comptime fn increment() {
    COUNTER += 1;
    assert_eq(get_counter(), COUNTER);
}
