use std::fmt::Debug;

use acvm::acir::circuit::opcodes::BlockId;

use crate::{
    errors::InternalError,
    ssa::ir::{call_stack::CallStack, types::NumericType},
};

use super::acir_context::{AcirType, AcirVar};

#[derive(Clone)]
pub(super) struct AcirDynamicArray {
    /// Identification for the Acir dynamic array
    /// This is essentially a ACIR pointer to the array
    pub(super) block_id: BlockId,
    /// Length of the array
    pub(super) len: usize,
    /// An ACIR dynamic array is a flat structure, so we use
    /// the inner structure of an `AcirType::NumericType` directly.
    /// Some usages of ACIR arrays (e.g. black box functions) require the bit size
    /// of every value to be known, thus we store the types as part of the dynamic
    /// array definition.
    ///
    /// A dynamic non-homogenous array can potentially have values of differing types.
    /// Thus, we store a vector of types rather than a single type, as a dynamic non-homogenous array
    /// is still represented in ACIR by a single `AcirDynamicArray` structure.
    ///
    /// The length of the value types vector must match the `len` field in this structure.
    pub(super) value_types: Vec<NumericType>,
    /// Identification for the ACIR dynamic array
    /// inner element type sizes array
    pub(super) element_type_sizes: Option<BlockId>,
}
impl Debug for AcirDynamicArray {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "id: {}, len: {}, element_type_sizes: {:?}",
            self.block_id.0,
            self.len,
            self.element_type_sizes.map(|block_id| block_id.0)
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AcirValue {
    Var(AcirVar, AcirType),
    Array(im::Vector<AcirValue>),
    DynamicArray(AcirDynamicArray),
}

impl AcirValue {
    pub(super) fn into_var(self) -> Result<AcirVar, InternalError> {
        match self {
            AcirValue::Var(var, _) => Ok(var),
            AcirValue::DynamicArray(_) | AcirValue::Array(_) => Err(InternalError::General {
                message: "Called AcirValue::into_var on an array".to_string(),
                call_stack: CallStack::new(),
            }),
        }
    }

    pub(super) fn borrow_var(&self) -> Result<AcirVar, InternalError> {
        match self {
            AcirValue::Var(var, _) => Ok(*var),
            AcirValue::DynamicArray(_) | AcirValue::Array(_) => Err(InternalError::General {
                message: "Called AcirValue::borrow_var on an array".to_string(),
                call_stack: CallStack::new(),
            }),
        }
    }

    pub(super) fn flatten(self) -> Vec<(AcirVar, AcirType)> {
        match self {
            AcirValue::Var(var, typ) => vec![(var, typ)],
            AcirValue::Array(array) => array.into_iter().flat_map(AcirValue::flatten).collect(),
            AcirValue::DynamicArray(_) => unimplemented!("Cannot flatten a dynamic array"),
        }
    }

    pub(super) fn flat_numeric_types(self) -> Vec<NumericType> {
        match self {
            AcirValue::Array(_) => {
                self.flatten().into_iter().map(|(_, typ)| typ.to_numeric_type()).collect()
            }
            AcirValue::DynamicArray(AcirDynamicArray { value_types, .. }) => value_types,
            _ => unreachable!("An AcirValue::Var cannot be used as an array value"),
        }
    }
}
