// Regression test for https://github.com/noir-lang/noir-claude/issues/1456
//
// Reflecting a vector literal `@[...]` through `Quoted::as_expr()`, interpolating it into a
// format string with `f"{expr}"`, and re-lexing via `.quoted_contents()` must preserve the
// vector spelling `@[...]` rather than lowering it to a fixed-array literal `[...]`. Otherwise
// the regenerated source takes `[Field; N]` trait dispatch instead of `[Field]` dispatch.

comptime fn unquote(code: Quoted) -> Quoted {
    code
}

comptime fn render_expr_to_code(code: Quoted) -> Quoted {
    let expr = code.as_expr().unwrap();
    f"{expr}".quoted_contents()
}

trait Kind {
    fn kind(self) -> Field;
}
impl Kind for [Field] {
    fn kind(self) -> Field {
        1
    }
}
impl<let N: u32> Kind for [Field; N] {
    fn kind(self) -> Field {
        2
    }
}

fn main() {
    let direct = unquote!(quote { @[1, 2, 3] }).kind();
    assert(direct == 1);

    let via_generated_source = render_expr_to_code!(quote { @[1, 2, 3] }).kind();
    assert(via_generated_source == 1);
}
