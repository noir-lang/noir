use dep::std;

struct Pair {
    first: Field,
    second: Field,
}

impl Pair {
    fn foo(p: Pair) -> Field {
        p.second
    }
}

fn main(x : Field, y : Field) {
    constrain Pair { first: x, second: y }.first == 1;
    constrain Pair { first: x, second: y }.foo() == 0;
}
    
