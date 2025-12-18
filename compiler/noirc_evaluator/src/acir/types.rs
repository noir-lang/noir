use std::fmt::Debug;

use acvm::acir::circuit::opcodes::BlockId;

use crate::{
    errors::InternalError,
    ssa::ir::{types::NumericType, types::Type as SsaType},
};
use noirc_errors::call_stack::CallStack;

/// High level Type descriptor for Variables.
///
/// One can think of Expression/Witness/Const
/// as low level types which can represent high level types.
///
/// An Expression can represent a u32 for example.
/// We could store this information when we do a range constraint
/// but this information is readily available by the caller so
/// we allow the user to pass it in.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AcirType {
    NumericType(NumericType),
    Array(Vec<AcirType>, usize),
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

/// Represents an array whose size is not known during compile time,
/// and which might be indexed with a dynamic index.
///
/// For example, in this Noir program:
///
/// ```noir
/// fn main(array: [Field; 3], index: u32) -> pub Field {
///     array[index]
/// }
/// ```
///
/// the array, first represented as an [AcirValue::Array], will be put in
/// an [AcirValue::DynamicArray]:
///
/// ```acir
/// private parameters: [w0, w1, w2, w3]
/// public parameters: []
/// return values: [w4]
/// INIT b0 = [w0, w1, w2]
/// READ w5 = b0[w3]
/// ASSERT w4 = w5
/// ```
///
/// Every dynamic array has an associated [BlockId] where its contents are stored,
/// in this case `b0`. Then the block can be read or written using dynamic indexes.
///
/// Dynamic arrays might result from other operations. For example:
/// - setting the value of an array element with a dynamic index
/// - pushing back to a vector where it's length is not known at compile time
/// - inserting to a vector with a dynamic index
/// - removing from a vector at a dynamic index
#[derive(Clone)]
pub(super) struct AcirDynamicArray {
    /// Identification for the Acir dynamic array
    /// This is essentially a ACIR pointer to the array
    pub(super) block_id: BlockId,
    /// Flattened length of the elements in the array
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
    /// This vector only holds the numeric types for a single dynamic array element.
    /// For example, if in Noir or SSA we have `[(u8, u32, Field); 3]` then `len` will be 3
    /// and `value_types` will be `[u8, u32, Field]`. To know the type of the element at index `i`
    /// we can fetch `value_types[i % value_types.len()]`.
    pub(super) value_types: Vec<NumericType>,
    /// Identification for the ACIR dynamic array
    /// inner element type sizes array
    pub(super) element_type_sizes: Option<BlockId>,
}

impl Debug for AcirDynamicArray {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "id: {}, len: {}, value_types: {:?}, element_type_sizes: {:?}",
            self.block_id.0,
            self.len,
            self.value_types,
            self.element_type_sizes.map(|block_id| block_id.0)
        )
    }
}

/// All possible variants of values in ACIR.
#[derive(Debug, Clone)]
pub(crate) enum AcirValue {
    /// Represents a single numeric value in ACIR.
    Var(AcirVar, NumericType),
    /// Represents a fixed-size array of ACIR values that are never indexed with
    /// a dynamic index.
    ///
    /// For example, in this Noir program:
    ///
    /// ```noir
    /// fn main(array: [Field; 3]) {
    ///     // Safety: example
    ///     unsafe { call(array) };
    /// }
    ///
    /// unconstrained fn call(array: [Field; 3]) {
    ///     println(array);
    /// }
    /// ```
    ///
    /// the array will be represented as an [AcirValue::Array]
    /// but we can see that it's simply a vector of variables:
    ///
    /// ```acir
    /// private parameters: [w0, w1, w2]
    /// public parameters: []
    /// return values: []
    /// BRILLIG CALL func: 0, inputs: [[w0, w1, w2]], outputs: []
    /// ```
    ///
    /// Compare this with `DynamicArray` below.
    Array(im::Vector<AcirValue>),
    /// Represents an array whose size is not known during compile time,
    /// and which might be indexed with a dynamic index.
    ///
    /// See [AcirDynamicArray] for more details and how this differs from
    /// [AcirValue::Array].
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

    /// Fetch a flat vector of ([AcirVar], [AcirType]).
    ///
    /// # Panics
    /// If [AcirValue::DynamicArray] is supplied or an inner element of an [AcirValue::Array].
    /// This is because an [AcirValue::DynamicArray] is simply a pointer to an array
    /// and fetching its internal [AcirValue::Var] would require laying down opcodes to read its content.
    /// This method should only be used where dynamic arrays are not a possible type.
    pub(super) fn flatten(self) -> Vec<(AcirVar, NumericType)> {
        match self {
            AcirValue::Var(var, typ) => vec![(var, typ)],
            AcirValue::Array(array) => array.into_iter().flat_map(AcirValue::flatten).collect(),
            AcirValue::DynamicArray(_) => unimplemented!("Cannot flatten a dynamic array"),
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
}

/// Assumes `typ` is an array or vector type with nested numeric types, arrays or vectors
/// (recursively) and returns a flat vector of all the contained numeric types.
/// Panics if `self` is not an array or vector type or if a function or reference type
/// is found along the way.
pub(crate) fn flat_numeric_types(typ: &SsaType) -> Vec<NumericType> {
    match typ {
        SsaType::Array(..) | SsaType::Vector(..) => {
            let mut flat_types = Vec::new();
            collect_flat_numeric_types(typ, &mut flat_types);
            flat_types
        }
        _ => panic!("Called flat_numeric_types on a non-array/vector type"),
    }
}

/// Returns the fully flattened numeric types for one element of a vector/array,
/// recursively flattening nested arrays.
/// For example, for Vector([(u32, u32, [Field; 3])]), this returns [u32, u32, Field, Field, Field].
pub(crate) fn flat_element_types(typ: &SsaType) -> Vec<NumericType> {
    match typ {
        SsaType::Vector(element_types) | SsaType::Array(element_types, _) => {
            let mut flat_types = Vec::new();
            for element_typ in element_types.iter() {
                collect_fully_flattened_numeric_types(element_typ, &mut flat_types);
            }
            flat_types
        }
        _ => panic!("Called flat_element_types on a non-array/vector type"),
    }
}

/// Helper function for `flat_element_types` that fully flattens arrays using the length.
/// This is different from `collect_flat_numeric_types` which returns just the first element.
fn collect_fully_flattened_numeric_types(typ: &SsaType, flat_types: &mut Vec<NumericType>) {
    match typ {
        SsaType::Numeric(numeric_type) => {
            flat_types.push(*numeric_type);
        }
        SsaType::Array(types, len) => {
            // For arrays, multiply by length to get the fully flattened representation
            for _ in 0..*len {
                for typ in types.iter() {
                    collect_fully_flattened_numeric_types(typ, flat_types);
                }
            }
        }
        SsaType::Vector(_) => {
            panic!("Cannot fully flatten a vector type - vectors have dynamic length")
        }
        _ => panic!("Called collect_fully_flattened_numeric_types on unsupported type"),
    }
}

/// Helper function for `flat_numeric_types` that recursively collects all numeric types
/// into `flat_types`.
fn collect_flat_numeric_types(typ: &SsaType, flat_types: &mut Vec<NumericType>) {
    match typ {
        SsaType::Numeric(numeric_type) => {
            flat_types.push(*numeric_type);
        }
        SsaType::Array(types, _) | SsaType::Vector(types) => {
            for typ in types.iter() {
                collect_flat_numeric_types(typ, flat_types);
            }
        }
        _ => panic!("Called collect_flat_numeric_types on non-array/vector/number type"),
    }
}
