use dep::std::option::Option;

fn main() {
    let none = Option::none();
    let some = Option::some(3);

    assert(none.is_none());
    assert(some.is_some());

    assert(some.unwrap() == 3);

    assert(none.unwrap_or(2) == 2);
    assert(some.unwrap_or(2) == 3);

    assert(none.unwrap_or_else(|| 5) == 5);
    assert(some.unwrap_or_else(|| 5) == 3);

    assert(none.map(|x| x * 2).is_none());
    assert(some.map(|x| x * 2).unwrap() == 6);

    assert(none.map_or(0, |x| x * 2) == 0);
    assert(some.map_or(0, |x| x * 2) == 6);

    assert(none.map_or_else(|| 0, |x| x * 2) == 0);
    assert(some.map_or_else(|| 0, |x| x * 2) == 6);

    assert(none.and(none).is_none());
    assert(none.and(some).is_none());
    assert(some.and(none).is_none());
    assert(some.and(some).is_some());

    let add1_u64 = |value: Field| Option::some(value as u64 + 1);

    assert(none.and_then(|_value| Option::none()).is_none());
    assert(none.and_then(add1_u64).is_none());
    assert(some.and_then(|_value| Option::none()).is_none());
    assert(some.and_then(add1_u64).unwrap() == 4);

    assert(none.or(none).is_none());
    assert(none.or(some).is_some());
    assert(some.or(none).is_some());
    assert(some.or(some).is_some());

    assert(none.or_else(|| Option::none()).is_none());
    assert(none.or_else(|| Option::some(5)).is_some());
    assert(some.or_else(|| Option::none()).is_some());
    assert(some.or_else(|| Option::some(5)).is_some());

    assert(none.xor(none).is_none());
    assert(none.xor(some).is_some());
    assert(some.xor(none).is_some());
    assert(some.xor(some).is_none());
}
