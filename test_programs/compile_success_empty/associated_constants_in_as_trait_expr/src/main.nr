trait Serialize {
    let N: u32;
}

impl Serialize for Field {
    let N: u32 = 1;
}

fn main() {
    let x = <Field as Serialize>::N;
    assert_eq(x, 1);
}
