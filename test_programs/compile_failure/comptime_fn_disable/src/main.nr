#[use_disable_me_too_fn]
unconstrained fn main() {
    disable_me();
    disable_me_too();
}

#[disable]
fn disable_me() {}

#[disable]
fn disable_me_too() {}

comptime fn use_disable_me_too_fn(_: FunctionDefinition) {
    // Expect an error if a function has already been called before `disable` runs on it
    // `use_disable_me_too_fn` must be in an attr before `disable` is called on `disable_me_too`
    // for this to work.
    disable_me_too()
}

comptime fn disable(f: FunctionDefinition) {
    let name = f.name();
    f.disable(f"{name} is disabled");
}
