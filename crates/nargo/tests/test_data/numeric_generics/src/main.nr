
struct Wrapper<M> {
    array: [Field; M]
}

fn id<N>(a: [Field; N]) -> [Field; N] {
    a
}

fn sum<N>(a: [Field; N]) -> Field {
    let mut acc = 0;
    for elem in a {
        acc = acc + elem;
    }
    acc
}

fn main() {
    let array = [1, 2, 3, 4];
    constrain id(array)[3] == 4;

    let w = Wrapper { array };
    constrain id(w.array) == array;

    let different_size = [5, 6, 7];
    constrain sum(id(different_size)) == 13;
}
