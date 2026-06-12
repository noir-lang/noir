global ADD_ONE: fn[(Field,)](Field) -> Field = {
    let offset = 1;
    |x: Field| x + offset
};

fn main(x: Field) -> pub Field {
    ADD_ONE(x)
}
