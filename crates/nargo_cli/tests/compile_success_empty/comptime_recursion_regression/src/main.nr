use dep::std;

fn main(x: Field, y: Field) {
    let flag = (x == 1) | (y == 2);
    std::assert_eq(flag | false, flag);
}
