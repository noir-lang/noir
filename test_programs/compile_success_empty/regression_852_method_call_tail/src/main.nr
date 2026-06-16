// Regression for noir-claude#852: when a tail is spliced onto an interned lvalue
// (`$lv.method()`, `$lv[i]`, `$lv += rhs`, ...), `Quoted::as_expr` must preserve it
// rather than silently dropping it down to the bare lvalue.
fn main() {
    comptime {
        // `lv` is an interned lvalue extracted from the lhs of an assignment.
        let lv = quote { x = 1 }.as_expr().unwrap().as_assign().unwrap().0;

        // Method-call tail (the originally reported case).
        let method_call = quote { $lv.foo() }.as_expr().unwrap();
        assert(method_call.as_method_call().is_some(), "method-call tail dropped");

        // Member-access tail.
        let member_access = quote { $lv.bar }.as_expr().unwrap();
        assert(member_access.as_member_access().is_some(), "member-access tail dropped");

        // Assignment whose lhs is an indexed interned lvalue (exercises the `[i]` tail).
        let index_assign = quote { $lv[0] = 1 }.as_expr().unwrap();
        assert(index_assign.as_assign().is_some(), "indexed assignment dropped");

        // Compound assignment (`+=`). There is no dedicated accessor, so observe that the
        // parsed result no longer collapses to the bare lvalue `$lv`.
        let compound_assign = quote { $lv += 1 }.as_expr().unwrap();
        assert(compound_assign.quoted() != quote { $lv }, "compound-assignment tail dropped");
    }
}
