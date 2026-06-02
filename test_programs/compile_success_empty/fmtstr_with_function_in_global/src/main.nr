global function_in_fmtstr: fmtstr<16, (fn() -> (),)> = f"function: {func}";

fn func() {}

fn main() {
    let _ = function_in_fmtstr;
}
