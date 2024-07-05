use std::append::Append;

fn main() {
    let slice = &[1, 2, 3, 4, 5];

    let odds = slice.filter(|x| x % 2 == 1);
    assert_eq(odds, &[1, 3, 5]);

    let odds_and_evens = append_three(odds, &[100], &[2, 4]);
    assert_eq(odds_and_evens, &[1, 3, 5, 100, 2, 4]);
}

fn append_three<T>(one: T, two: T, three: T) -> T where T: Append {
    // The last .append(T::empty()) should do nothing
    one.append(two).append(three).append(T::empty())
}
