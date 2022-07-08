use dep::std;

fn main(x : Field, y : Field) {
    let p = x == y;
    constrain p == true;
}
