// Regression from issue #7964 (https://github.com/noir-lang/noir/issues/7964)
global G_A: bool = false;
fn main(a: Field) -> pub Field {
    if (
        (
            if (!G_A) {
                if (!G_A) {
                    (a as u128)
                } else {
                    (a as u128)
                }
            } else {
                (a as u128)
            }
                <= (a as u128)
        ) as bool
    ) {
        (-(-a))
    } else {
        (-(-a))
    }
}
