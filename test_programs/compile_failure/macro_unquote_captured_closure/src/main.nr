comptime fn make_adder() -> Quoted {
    let offset = 1;
    let f = |x: Field| x + offset;
    quote { $f }
}

fn main(x: Field) -> pub Field {
    let f = make_adder!();
    f(x)
}
