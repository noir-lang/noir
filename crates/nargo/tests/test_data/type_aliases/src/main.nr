use dep::std;

type foo = u8;

fn main(x : Field, y : pub Field) {
    let a: foo = 1;
    let b: foo = 2;
    constrain x != y;
    constrain a != b;
}