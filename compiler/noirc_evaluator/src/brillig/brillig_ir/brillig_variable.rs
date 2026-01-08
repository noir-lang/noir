use std::ops::Deref;

use acvm::{
    FieldElement,
    acir::{
        AcirField,
        brillig::{BitSize, lengths::SemanticLength},
    },
    brillig_vm::brillig::{HeapValueType, MemoryAddress},
};
use serde::{Deserialize, Serialize};

use crate::{
    brillig::brillig_ir::registers::{Allocated, RegisterAllocator},
    ssa::ir::types::Type,
};

use super::BRILLIG_MEMORY_ADDRESSING_BIT_SIZE;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) struct SingleAddrVariable {
    pub(crate) address: MemoryAddress,
    pub(crate) bit_size: u32,
}

impl SingleAddrVariable {
    pub(crate) fn new(address: MemoryAddress, bit_size: u32) -> Self {
        SingleAddrVariable { address, bit_size }
    }

    pub(crate) fn new_usize(address: MemoryAddress) -> Self {
        SingleAddrVariable { address, bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE }
    }

    pub(crate) fn new_field(address: MemoryAddress) -> Self {
        SingleAddrVariable { address, bit_size: FieldElement::max_num_bits() }
    }
}

/// The representation of a Noir array in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) struct BrilligArray {
    pub(crate) pointer: MemoryAddress,
    /// The number of memory slots the array occupies.
    ///
    /// This is the flattened size of the array, where complex types
    /// take up more than one slot.
    pub(crate) size: usize,
    // TODO(lengths): use SemiFlattenedLength here instead of usize
}

/// The representation of a noir vector in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) struct BrilligVector {
    pub(crate) pointer: MemoryAddress,
}

/// The representation of a noir value in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) enum BrilligVariable {
    SingleAddr(SingleAddrVariable),
    BrilligArray(BrilligArray),
    BrilligVector(BrilligVector),
}

impl BrilligVariable {
    /// Extract a [SingleAddrVariable].
    ///
    /// Panics if the variable is an array or vector.
    pub(crate) fn extract_single_addr(self) -> SingleAddrVariable {
        match self {
            BrilligVariable::SingleAddr(single_addr) => single_addr,
            _ => unreachable!("ICE: Expected single address, got {self:?}"),
        }
    }

    /// Extract a [BrilligArray].
    ///
    /// Panics if it's a single address variable or a vector.
    pub(crate) fn extract_array(self) -> BrilligArray {
        match self {
            BrilligVariable::BrilligArray(array) => array,
            _ => unreachable!("ICE: Expected array, got {self:?}"),
        }
    }

    /// Extract a [BrilligVector].
    ///
    /// Panics if it's a single address variable or an array.
    pub(crate) fn extract_vector(self) -> BrilligVector {
        match self {
            BrilligVariable::BrilligVector(vector) => vector,
            _ => unreachable!("ICE: Expected vector, got {self:?}"),
        }
    }

    /// Extract the [MemoryAddress] out of any [BrilligVariable].
    ///
    /// This can be deallocated to make the memory available for reuse.
    ///
    /// Note that this is a single address even for vectors, because this is a `BrilligVector`, not a `HeapVector`.
    pub(crate) fn extract_register(self) -> MemoryAddress {
        match self {
            BrilligVariable::SingleAddr(single_addr) => single_addr.address,
            BrilligVariable::BrilligArray(array) => array.pointer,
            BrilligVariable::BrilligVector(vector) => vector.pointer,
        }
    }
}

impl From<SingleAddrVariable> for BrilligVariable {
    fn from(value: SingleAddrVariable) -> Self {
        Self::SingleAddr(value)
    }
}

impl From<BrilligArray> for BrilligVariable {
    fn from(value: BrilligArray) -> Self {
        Self::BrilligArray(value)
    }
}

impl From<BrilligVector> for BrilligVariable {
    fn from(value: BrilligVector) -> Self {
        Self::BrilligVector(value)
    }
}

impl<T, R: RegisterAllocator> From<&Allocated<T, R>> for BrilligVariable
where
    BrilligVariable: From<T>,
    T: Copy,
{
    fn from(value: &Allocated<T, R>) -> Self {
        Self::from(*value.deref())
    }
}

/// Convenience method to convert e.g. an `Allocated<BrilligArray, _>` to a `BrilligVariable`.
#[cfg(test)]
impl<T, R: RegisterAllocator> Allocated<T, R>
where
    BrilligVariable: From<T>,
    T: Copy,
{
    /// Convert the allocated value into a [BrilligVariable].
    pub(crate) fn to_var(&self) -> BrilligVariable {
        BrilligVariable::from(**self)
    }
}

/// Convert an SSA [Type] to [HeapValueType] for passing values to foreign calls.
pub(crate) fn type_to_heap_value_type(typ: &Type) -> HeapValueType {
    match typ {
        Type::Numeric(_) | Type::Reference(_) | Type::Function => HeapValueType::Simple(
            BitSize::try_from_u32::<FieldElement>(get_bit_size_from_ssa_type(typ)).unwrap(),
        ),
        Type::Array(elem_type, size) => HeapValueType::Array {
            value_types: elem_type.as_ref().iter().map(type_to_heap_value_type).collect(),
            size: SemanticLength(*size as usize),
        },
        Type::Vector(elem_type) => HeapValueType::Vector {
            value_types: elem_type.as_ref().iter().map(type_to_heap_value_type).collect(),
        },
    }
}

pub(crate) fn get_bit_size_from_ssa_type(typ: &Type) -> u32 {
    match typ {
        Type::Reference(_) => BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
        // NB. function references are converted to a constant when
        // translating from SSA to Brillig (to allow for debugger
        // instrumentation to work properly)
        Type::Function => 32,
        typ => typ.bit_size(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use acvm::acir::brillig::{HeapValueType, lengths::SemanticLength};

    use crate::{
        brillig::brillig_ir::brillig_variable::type_to_heap_value_type, ssa::ir::types::Type,
    };

    #[test]
    fn type_to_heap_value_type_flattened_size() {
        // typ = [(u32, bool); 3]
        let typ = Type::Array(Arc::new(vec![Type::unsigned(32), Type::bool()]), 3);
        let typ = type_to_heap_value_type(&typ);
        assert_eq!(typ.flattened_size(), Some(6));

        let HeapValueType::Array { value_types: _, size } = typ else {
            panic!("Expected array type");
        };
        assert_eq!(size, SemanticLength(3));

        // typ = [[u32; 4]; 2]
        let arr = Type::Array(Arc::new(vec![Type::unsigned(32)]), 4);
        let typ = Type::Array(Arc::new(vec![arr]), 2);
        let typ = type_to_heap_value_type(&typ);
        assert_eq!(typ.flattened_size(), Some(8));

        let HeapValueType::Array { value_types: _, size } = typ else {
            panic!("Expected array type");
        };
        assert_eq!(size, SemanticLength(2));

        // typ = [([u32; 4], bool); 2]
        let arr = Type::Array(Arc::new(vec![Type::unsigned(32)]), 4);
        let typ = Type::Array(Arc::new(vec![arr, Type::bool()]), 2);
        let typ = type_to_heap_value_type(&typ);
        assert_eq!(typ.flattened_size(), Some(10));

        let HeapValueType::Array { value_types: _, size } = typ else {
            panic!("Expected array type");
        };
        assert_eq!(size, SemanticLength(2));
    }
}
