//! Like Crane-lift all functions outside of the current function is seen as
//! external.
//! To reference external functions, one uses

use super::types::Typ;

#[derive(Debug, Default, Clone)]
pub(crate) struct Signature {
    pub(crate) params: Vec<Typ>,
    pub(crate) returns: Vec<Typ>,
}
/// Reference to a `Signature` in a map inside of
/// a functions DFG.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SigRef(pub(crate) u32);

#[test]
fn sign_smoke() {
    let mut signature = Signature::default();

    signature.params.push(Typ::Numeric(super::types::NumericType::NativeField));
    signature.returns.push(Typ::Numeric(super::types::NumericType::Unsigned { bit_size: 32 }));
}
