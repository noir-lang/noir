// An assertion whose condition is known to be false at compile time must abort
// compilation with a hard error rather than emit a warning and an unsatisfiable circuit.
// https://github.com/noir-lang/noir/issues/12922
fn check<let CONSTRAINED: u32, let HANDSHAKE: u32>(x: Field) -> Field {
    if CONSTRAINED == 1 {
        assert(HANDSHAKE == 1, "constrained delivery requires a handshake origin");
    }
    x
}

fn main(x: Field) -> pub Field {
    check::<1, 0>(x)
}
