use dep::std;

type foo = u8;

fn main(x : u8) {
    let a: foo = 1;
    constrain a != x;
}