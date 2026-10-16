// The out-of-bounds `values[1]` reads sit inside a branch that `flag = false` never takes,
// so the program must solve and return 0.
fn main(flag: bool) -> pub Field {
    let mut out = 0;

    if flag {
        let mut values: [(Field, Field)] = @[(1, 2)];
        if flag {
            values = values.push_back(values[1]);
        }
        if flag {
            values = values.push_back(values[1]);
        }
        out = values[0].0;
    }

    out
}
