#[must_use = "This thingy must be used!"]
struct PleaseUseMe {}

fn main() {
    PleaseUseMe {};
    foo();
    let _ = foo();
}

fn foo() -> PleaseUseMe {
    PleaseUseMe {}
}
