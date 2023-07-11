use dep::std;

fn main(x : Field, y : pub Field) {

    std::println_new(x);
    assert(x != y);
}
