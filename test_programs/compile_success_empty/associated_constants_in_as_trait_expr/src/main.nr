trait Serialize {
    let N: u32;
}

impl Serialize for Field {
    let N: u32 = 1;
}

fn main() {
    let x = <Field as Serialize>::N;
    assert_eq(x, 1);
    assert_eq(get_n_1::<Field>(), 1);
    assert_eq(get_n_2::<Field>(), 1);

    comptime {
        let x = <Field as Serialize>::N;
        assert_eq(x, 1);
        assert_eq(get_n_1::<Field>(), 1);
        assert_eq(get_n_2::<Field>(), 1);
    }
}

fn get_n_1<T: Serialize>() -> u32 {
    T::N
}

fn get_n_2<T: Serialize>() -> u32 {
    <T as Serialize>::N
}
