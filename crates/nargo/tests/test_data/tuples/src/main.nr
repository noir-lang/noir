use dep::std;

fn main(x: Field, y: Field) {
    let pair = (x, y);
    constrain pair.0 == 1;
    constrain pair.1 == 0;
}
