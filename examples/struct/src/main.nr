use dep::std;

struct Pair {
    first: Field,
    second: Field,
}

fn main(x : Field, y : Field) {
    constrain Pair { first: x, second: y }.x == 1;
    constrain Pair { first: x, second: y }.y == 0;
}
    
