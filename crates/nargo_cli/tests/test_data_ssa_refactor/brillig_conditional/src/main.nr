// Tests a very simple program.
// 
// The features being tested is basic conditonal on brillig
fn main(x: Field)  {
    assert(5 == conditional(x as bool));
}

unconstrained fn conditional(x : bool) -> Field {
    // if x {
    //     4
    // }else {
    //     5
    // }
    let a = toto(x);
    a + 1
}

unconstrained fn toto(x : bool) -> Field {
    if x {
        4
    }else {
        5
    }
}