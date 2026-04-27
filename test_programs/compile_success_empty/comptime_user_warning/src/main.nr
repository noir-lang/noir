use std::meta::warn;

#[notice]
fn target() {}

#[notice_with_hint]
fn other() {}

comptime fn notice(f: FunctionDefinition) {
    warn(
        f"This is a `warn` test with no secondary on `{f}`",
        Option::none(),
        f.location(),
    );
}

comptime fn notice_with_hint(f: FunctionDefinition) {
    warn(
        f"This is a `warn` test with a secondary on `{f}`",
        Option::some(f"I am the secondary on `{f}`"),
        f.location(),
    );
}

struct IGiveLocations;

fn main() {
    target();
    other();

    // Ensure further blocks are still executed
    comptime {
        let location = quote[IGiveLocations].as_type().as_data_type().unwrap().0.location();
        warn(f"warning 3", Option::none(), location);
    }
    comptime {
        let location = quote[IGiveLocations].as_type().as_data_type().unwrap().0.location();
        warn(f"warning 4", Option::none(), location);
    }
}
