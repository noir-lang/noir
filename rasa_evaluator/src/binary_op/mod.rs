/// This module handles all of the binary operations between polynomials
pub mod add;
pub mod equal;
pub mod mul;
pub mod sub;
pub mod neq;

pub use add::handle_add_op;
pub use equal::handle_equal_op;
pub use mul::handle_mul_op;
pub use sub::handle_sub_op;
pub use neq::handle_neq_op;
