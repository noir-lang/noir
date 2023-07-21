use dep::std;

type foo = u8;

fn main(x : u8) {
    let a: foo = 1;
    assert(a != x);
}