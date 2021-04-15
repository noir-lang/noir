pub mod bn254_ark;
pub mod field_trait;

pub use field_trait::FieldElement;
pub type Bn254Scalar = ark_bn254::Fr;
