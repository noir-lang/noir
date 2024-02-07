use acvm::brillig_vm::brillig::{
    HeapArray, HeapValueType, HeapVector, MemoryAddress, ValueOrArray,
};
use serde::{Deserialize, Serialize};

use crate::ssa::ir::types::Type;

/// The representation of a noir array in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) struct BrilligArray {
    pub(crate) pointer: MemoryAddress,
    pub(crate) size: usize,
    pub(crate) rc: MemoryAddress,
}

impl BrilligArray {
    pub(crate) fn to_heap_array(self) -> HeapArray {
        HeapArray { pointer: self.pointer, size: self.size }
    }

    pub(crate) fn registers_count() -> usize {
        2
    }

    pub(crate) fn extract_registers(self) -> Vec<MemoryAddress> {
        vec![self.pointer, self.rc]
    }
}

/// The representation of a noir slice in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) struct BrilligVector {
    pub(crate) pointer: MemoryAddress,
    pub(crate) size: MemoryAddress,
    pub(crate) rc: MemoryAddress,
}

impl BrilligVector {
    pub(crate) fn to_heap_vector(self) -> HeapVector {
        HeapVector { pointer: self.pointer, size: self.size }
    }

    pub(crate) fn registers_count() -> usize {
        3
    }

    pub(crate) fn extract_registers(self) -> Vec<MemoryAddress> {
        vec![self.pointer, self.size, self.rc]
    }
}

/// The representation of a noir value in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) enum BrilligVariable {
    Simple(MemoryAddress),
    BrilligArray(BrilligArray),
    BrilligVector(BrilligVector),
}

impl BrilligVariable {
    pub(crate) fn extract_register(self) -> MemoryAddress {
        match self {
            BrilligVariable::Simple(register_index) => register_index,
            _ => unreachable!("ICE: Expected register, got {self:?}"),
        }
    }

    pub(crate) fn extract_array(self) -> BrilligArray {
        match self {
            BrilligVariable::BrilligArray(array) => array,
            _ => unreachable!("ICE: Expected array, got {self:?}"),
        }
    }

    pub(crate) fn extract_vector(self) -> BrilligVector {
        match self {
            BrilligVariable::BrilligVector(vector) => vector,
            _ => unreachable!("ICE: Expected vector, got {self:?}"),
        }
    }

    pub(crate) fn extract_registers(self) -> Vec<MemoryAddress> {
        match self {
            BrilligVariable::Simple(register_index) => vec![register_index],
            BrilligVariable::BrilligArray(array) => array.extract_registers(),
            BrilligVariable::BrilligVector(vector) => vector.extract_registers(),
        }
    }

    pub(crate) fn to_register_or_memory(self) -> ValueOrArray {
        match self {
            BrilligVariable::Simple(register_index) => ValueOrArray::MemoryAddress(register_index),
            BrilligVariable::BrilligArray(array) => ValueOrArray::HeapArray(array.to_heap_array()),
            BrilligVariable::BrilligVector(vector) => {
                ValueOrArray::HeapVector(vector.to_heap_vector())
            }
        }
    }
}

pub(crate) fn type_to_heap_value_type(typ: &Type) -> HeapValueType {
    match typ {
        Type::Numeric(_) | Type::Reference(_) | Type::Function => HeapValueType::Simple,
        Type::Array(elem_type, size) => HeapValueType::Array {
            value_types: elem_type.as_ref().iter().map(type_to_heap_value_type).collect(),
            size: typ.element_size() * size,
        },
        Type::Slice(elem_type) => HeapValueType::Vector {
            value_types: elem_type.as_ref().iter().map(type_to_heap_value_type).collect(),
        },
    }
}
