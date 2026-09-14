// A `#[fold]` function whose only call site is statically unreachable: the call is
// eliminated while `main`'s SSA is generated, but the function is still an entry point
// and must still get its own circuit, matching the signature count derived from the AST.
#[fold]
fn folded() -> Field {
    1
}

fn main() -> pub Field {
    if false {
        folded()
    } else {
        0
    }
}
