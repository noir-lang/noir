// Tests a very simple Brillig function.
// 
// The features being tested is not instruction on brillig
fn main(x: Field, y : Field)  {
    assert(!(x as bool) == not_operator(x as bool));
    assert((x as bool)== not_operator(!(x as bool)));
    assert(!(y as u8) == not_operator_8bit(y as u8));
}

unconstrained fn not_operator(x : bool) -> bool {
    !x
}

unconstrained fn not_operator_8bit(x : u8) -> u8 {
    !x
}