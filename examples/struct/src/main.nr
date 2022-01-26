use dep::std;

struct Pair {
    first: Field,
    second: Field,
}

fn main(x : Field, y : Field) {
    constrain Pair { first: x, second: y }.first == 1;
    constrain Pair { first: x, second: y }.second == 0;
}
    
