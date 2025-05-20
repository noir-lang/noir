// Returns whether to validate the output of the circuit. Eventually we'll want to change this
// to `!dep::std::runtime::is_unconstrained`, so we skip unneeded validations when we
// do not need to generate a proof. But for now, we always validate to make sure we catch
// errors earlier, as most of our tests run with real proofs disabled, which means validation
// checks get skipped.
pub fn should_validate_output() -> bool {
    true
}
