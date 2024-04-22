use dep::std;
// It is expected that `y` must be equal to 0.
fn main(x: field, y: pub field) {
    let a: field = x / y;
    std::println(a);
}
