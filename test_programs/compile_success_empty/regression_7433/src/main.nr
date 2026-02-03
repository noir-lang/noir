// These two globals should cancel out.
global positive_global: Field = 17;
global negative_global: Field = -17; // Previously the negative sign here would be dropped so this became 17.

fn main() {
    assert_eq(positive_global + negative_global, 0);
}
