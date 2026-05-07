// Regression: the truncate simplification incorrectly treated field division
// as integer division when reasoning about bit sizes, removing a necessary
// truncation after `array_get` optimization replaced the operand.
fn main(inp: Field) {
    let mut x: Field = -1;
    for _i in 0..1 {
        x = -1;
    }

    let mut y = [inp / x];

    let expected = (((y[0]) as u128) as Field);
    let mut actual: Field = 0;

    if (inp.lt(42)) {
        actual = (((y[0]) as u128) as Field);
    }

    assert_eq(actual, expected);
}
