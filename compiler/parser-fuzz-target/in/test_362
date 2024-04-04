use dep::std;

fn main(x: u64, y: pub u64) -> pub u64 {
    // We include a println statement to show that noirJS will ignore this and continue execution
    std::println("foo");

    // A dynamic assertion message is used to show that noirJS will ignore the call and continue execution
    // The assertion passes and thus the foreign call for resolving an assertion message should not be called.
    assert(x < y, f"Expected x < y but got {x} < {y}");

    assert(x < y);
    x + y
}
