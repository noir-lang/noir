// The type expression `(N - 1) + 1` simplifies to `N`, but its unsimplified
// form is kept so that intermediate steps are checked at monomorphization.
// With N = 0, the `(0 - 1)` subexpression underflows u32 even though the
// simplified form evaluates to 0 without error, so this must fail to compile.
// Unlike `arithmetic_generics_intermediate_underflow`, where the underflow is
// visible in the simplified form, here it only exists in an intermediate step.
fn intermediate_underflow<let N: u32>() -> [Field; (N - 1) + 1] {
    let result: [Field; N] = std::mem::zeroed();
    result
}

fn main() {
    let _x = intermediate_underflow::<0>();
}
