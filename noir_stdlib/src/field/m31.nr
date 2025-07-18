use crate::field::field_less_than;
use crate::runtime::is_unconstrained;

pub fn lt(x: Field, y: Field) -> bool {
    if is_unconstrained() {
        // Safety: unsafe in unconstrained
        unsafe {
            field_less_than(x, y)
        }
    } else {
        // Safety: Take a hint of the comparison and verify it
        unsafe {
            if field_less_than(x, y) {
                assert_lt(x, y);
                true
            } else {
                assert_lt(y, x);
                false
            }
        }
    }
}

pub fn assert_lt(a: Field, b: Field) {
   // TODO: Implement this
}