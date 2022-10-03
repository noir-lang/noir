use dep::std;

fn main(x: Field, y: Field) {
    let pair = (x, y);
    constrain pair.0 == 1;
    constrain pair.1 == 0;

    let (a, b) = (0, 1);
    constrain a == 0;
    constrain b == 1;
}
