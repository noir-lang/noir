use dep::std;

fn main(x : Field, y : Field) {
    let p = std::predicate_eq(x, y);
    constrain p == 1;
}
