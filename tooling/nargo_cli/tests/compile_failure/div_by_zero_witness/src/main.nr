use dep::std;

// It is expected that `y` must be equal to 0.
fn main(x : Field, y : pub Field) {
    let a: Field = x / y;
    std::println(a);
}
