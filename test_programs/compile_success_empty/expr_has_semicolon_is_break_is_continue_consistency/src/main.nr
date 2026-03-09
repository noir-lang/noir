// This program checks that `is_break`, `is_continue` and `has_semicolon` work the same
// way for a quoted expression and recursive quoted values that eventually lead to the same expression.
fn main() {
    comptime {
        let a = quote { break }.as_expr().unwrap();
        let b = quote { $a }.as_expr().unwrap();
        let c = quote { $b }.as_expr().unwrap();

        assert(a.is_break());
        assert(b.is_break());
        assert(c.is_break());

        // c is still semantically identical to `break`.
        assert(a.quoted() == quote { break });
        assert(b.quoted() == quote { break });
        assert(c.quoted() == quote { break });

        let a = quote { continue }.as_expr().unwrap();
        let b = quote { $a }.as_expr().unwrap();
        let c = quote { $b }.as_expr().unwrap();

        assert(a.is_continue());
        assert(b.is_continue());
        assert(c.is_continue());

        // c is still semantically identical to `continue`.
        assert(a.quoted() == quote { continue });
        assert(b.quoted() == quote { continue });
        assert(c.quoted() == quote { continue });

        let block = quote { { 1; } }.as_expr().unwrap();
        let a = block.as_block().unwrap()[0];
        let b = quote { $a }.as_expr().unwrap();
        let c = quote { $b }.as_expr().unwrap();

        assert(a.has_semicolon());
        assert(b.has_semicolon());
        assert(c.has_semicolon()); // WRONG

        // c is still semantically identical to `1;`.
        assert(a.quoted() == c.quoted());
        assert(c.quoted() == quote { 1; });
    }
}
