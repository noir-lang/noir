use std::meta::error;

#[reject]
fn forbidden() {}

comptime fn reject(f: FunctionDefinition) {
    error(
        f"I am the error test on `{f}`",
        Option::some(f"I am the secondary on `{f}`"),
        f.location(),
    );
}

struct IGiveLocations;

fn main() {
    forbidden();

    // Ensure further blocks are still executed
    comptime {
        let location = quote[IGiveLocations].as_type().as_data_type().unwrap().0.location();
        error(f"error 2", Option::none(), location);
    }
    comptime {
        let location = quote[IGiveLocations].as_type().as_data_type().unwrap().0.location();
        error(f"error 3", Option::none(), location);
        error(f"error 4", Option::none(), location);
    }
    // Ensure we do not stop early on error
    comptime {
        let code = quote[foo()];
        std::meta::unquote!(code);
        std::meta::unquote!(code);
    }
}

comptime fn foo() {
    let location = quote[IGiveLocations].as_type().as_data_type().unwrap().0.location();
    error::<_, _, 0, ()>(f"I am called twice", Option::none(), location);
}
