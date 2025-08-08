// This is an error as we can't fuzz if there are no arguments
#[fuzz]
fn fuzz_no_arguments() {}

// This is an error as `only_fail_with` implies fuzzing, but there are no arguments
#[test(only_fail_with = "error")]
fn test() {}

fn main() {}
