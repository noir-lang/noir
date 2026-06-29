// Entry-point inputs are validated automatically, so calling `main` with an invalid value is
// rejected by the injected validation — even though `main` itself ignores the input. The test is
// expected to fail with the invariant's message, confirming the validation runs: `main` stays an
// entry point under `nargo test`, and the validation lives in its body.
use std::validate::validate;

#[derive(Validate)]
#[validate(Percent::in_range)]
struct Percent {
    value: u8,
}

impl Percent {
    fn in_range(&self) {
        assert(self.value <= 100, "percent out of range");
    }
}

fn main(_p: Percent) {}

#[test(should_fail_with = "percent out of range")]
fn main_rejects_invalid_input() {
    main(Percent { value: 200 });
}
