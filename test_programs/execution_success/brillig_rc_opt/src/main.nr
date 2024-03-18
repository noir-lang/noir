
unconstrained fn main() {
    let mut array = [1, 2, 3];

    let mut alias = get_alias(array);
    alias[0] = 0;

    assert(array[0] == 1);
    assert(alias[0] == 0);

    let alias2 = get_alias(&mut array);
    alias2[0] = 4;

    assert(array[0] == 4);
    assert(alias[0] == 0);
    assert((*alias2)[0] == 4);
}

unconstrained fn get_alias<T>(x: T) -> T {
    x
}
