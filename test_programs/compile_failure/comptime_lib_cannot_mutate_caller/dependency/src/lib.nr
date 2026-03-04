// This attribute function tries to modify a function from the caller's crate.
// This should be rejected: a library cannot mutate items in the calling crate.
pub comptime fn make_unconstrained(f: FunctionDefinition) {
    f.set_unconstrained(true);
}
