//! This file holds convenient methods and essentially a DSL
//! to generate the ACIR IR.
//!
//! The notion of a Witness is abstracted away behind what is known
//! as an ACIR variable.

mod acir_variable;
mod errors;
pub(crate) mod generated_acir;
mod memory;
