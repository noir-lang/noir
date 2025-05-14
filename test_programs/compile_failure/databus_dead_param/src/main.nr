// Regression for issue #8398 (https://github.com/noir-lang/noir/issues/8398)
fn main(
    _a: (i8, u32, i8),
    _b: call_data(0) [(i8, i8, bool, bool, str<0>); 2],
) -> pub [(bool, str<3>, str<0>, u32); 0] {
    []
}
