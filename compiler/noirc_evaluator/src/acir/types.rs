use std::fmt::Debug;

use acvm::{AcirField, acir::circuit::opcodes::BlockId};

use crate::{
    errors::InternalError,
    ssa::ir::{types::NumericType, types::Type as SsaType},
};
use noirc_errors::call_stack::CallStack;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// High level Type descriptor for Variables.
///
/// One can think of Expression/Witness/Const
/// as low level types which can represent high level types.
///
/// An Expression can represent a u32 for example.
/// We could store this information when we do a range constraint
/// but this information is readily available by the caller so
/// we allow the user to pass it in.
pub(crate) enum AcirType {
    NumericType(NumericType),
    Array(Vec<AcirType>, usize),
}

impl AcirType {
    pub(crate) fn new(typ: NumericType) -> Self {
        Self::NumericType(typ)
    }

    /// Returns the bit size of the underlying type
    pub(crate) fn bit_size<F: AcirField>(&self) -> u32 {
        match self {
            AcirType::NumericType(numeric_type) => match numeric_type {
                NumericType::Signed { bit_size } => *bit_size,
                NumericType::Unsigned { bit_size } => *bit_size,
                NumericType::NativeField => F::max_num_bits(),
            },
            AcirType::Array(_, _) => unreachable!("cannot fetch bit size of array type"),
        }
    }

    /// Returns a field type
    pub(crate) fn field() -> Self {
        AcirType::NumericType(NumericType::NativeField)
    }

    /// Returns an unsigned type of the specified bit size
    pub(crate) fn unsigned(bit_size: u32) -> Self {
        AcirType::NumericType(NumericType::Unsigned { bit_size })
    }

    pub(crate) fn to_numeric_type(&self) -> NumericType {
        match self {
            AcirType::NumericType(numeric_type) => *numeric_type,
            AcirType::Array(_, _) => unreachable!("cannot fetch a numeric type for an array type"),
        }
    }
}

impl From<SsaType> for AcirType {
    fn from(value: SsaType) -> Self {
        AcirType::from(&value)
    }
}

impl From<&SsaType> for AcirType {
    fn from(value: &SsaType) -> Self {
        match value {
            SsaType::Numeric(numeric_type) => AcirType::NumericType(*numeric_type),
            SsaType::Array(elements, size) => {
                let elements = elements.iter().map(|e| e.into()).collect();
                AcirType::Array(elements, *size as usize)
            }
            _ => unreachable!("The type {value} cannot be represented in ACIR"),
        }
    }
}

impl From<NumericType> for AcirType {
    fn from(value: NumericType) -> Self {
        AcirType::NumericType(value)
    }
}

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

    /// Generates an uninitialized `AcirValue` of the given type.
    pub(super) fn uninitialized(typ: AcirType) -> Self {
        match typ {
            AcirType::NumericType(_) => AcirValue::Var(AcirVar::uninitialized(), typ),
            AcirType::Array(acir_types, len) => {
                let values: im::Vector<AcirValue> =
                    acir_types.iter().map(|t| AcirValue::uninitialized(t.clone())).collect();
                let values = if values.len() == 1 {
                    values.into_iter().next().unwrap()
                } else {
                    AcirValue::Array(values)
                };
                AcirValue::Array(im::Vector::from(vec![values; len]))
            }
        }
    }
}

/// A Reference to an `AcirVarData`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(super) struct AcirVar(usize);

impl AcirVar {
    pub(super) fn new(var: usize) -> Self {
        AcirVar(var)
    }

    /// Returns the uninitialized `AcirVar`, corresponding to an unsolvable witness in ACIR.
    pub(super) fn uninitialized() -> Self {
        AcirVar(0)
    }
}
