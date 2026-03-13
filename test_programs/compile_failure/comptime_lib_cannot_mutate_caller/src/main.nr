use dependency::make_unconstrained;

// Applying a library attribute that tries to set_unconstrained on a local function
// should fail because the library cannot modify items in the caller's crate.
#[make_unconstrained]
fn foo() {}

fn main() {}
