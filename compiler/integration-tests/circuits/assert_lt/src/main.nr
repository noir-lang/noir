use dep::std;

fn main(x: u64, y: pub u64) -> pub u64 {
    // We include a println statement to show that noirJS will ignore this and continue execution
    std::println("foo");


    assert(x < y);
    x + y
}
