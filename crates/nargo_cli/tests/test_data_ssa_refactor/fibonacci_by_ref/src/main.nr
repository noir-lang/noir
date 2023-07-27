fn fib_fn(a: Field, b: Field, res: &mut Field) {
    *res = a + b;
}

fn main(mut prev: Field, mut cur: Field) {

    let mut fib = prev + cur;
    for i in 1..10 {
        prev = cur;
        cur = fib;
        fib_fn(prev, cur, &mut fib);
        assert(prev + cur == fib);
    }

}
