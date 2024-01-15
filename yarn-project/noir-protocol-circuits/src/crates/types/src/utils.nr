// general util packages/modules are usually bad practice
// because there is no criteria for what we should not put in here.
// Reducing the size of this package would be welcome.

mod arrays;
mod bounded_vec;
mod field;
mod uint256;

// if predicate == true then return lhs, else return rhs
pub fn conditional_assign(predicate: bool, lhs: Field, rhs: Field) -> Field {
    if predicate { lhs } else { rhs }
}
