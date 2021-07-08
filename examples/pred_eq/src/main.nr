use dep::std;
fn main(x : Field, y : Field) {
    priv p = std::predicate_eq(x, y);
    constrain p == 1;
}
    