use dep::std;

fn main(x: Field, y: Field) {
    let mut z = x + y;
    std::assert_eq(z, 3);
    z = x * y;
    std::assert_eq(z, 2);
}
