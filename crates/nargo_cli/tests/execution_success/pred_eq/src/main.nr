use dep::std;

fn main(x: Field, y: Field) {
    let p = x == y;
    std::assert_eq(p, true);
}
