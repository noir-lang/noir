use acvm::brillig_vm::brillig::{
    HeapArray, HeapValueType, HeapVector, RegisterIndex, RegisterOrMemory,
};
use serde::{Deserialize, Serialize};

use crate::ssa::ir::types::Type;

/// The representation of a noir array in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) struct BrilligArray {
    pub(crate) pointer: RegisterIndex,
    pub(crate) size: usize,
    pub(crate) rc: RegisterIndex,
}

impl BrilligArray {
    pub(crate) fn to_heap_array(self, value_types: Vec<HeapValueType>) -> HeapArray {
        HeapArray { pointer: self.pointer, size: self.size, value_types }
    }

    pub(crate) fn registers_count() -> usize {
        2
    }

    pub(crate) fn extract_registers(self) -> Vec<RegisterIndex> {
        vec![self.pointer, self.rc]
    }
}

/// The representation of a noir slice in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) struct BrilligVector {
    pub(crate) pointer: RegisterIndex,
    pub(crate) size: RegisterIndex,
    pub(crate) rc: RegisterIndex,
}

impl BrilligVector {
    pub(crate) fn to_heap_vector(self, value_types: Vec<HeapValueType>) -> HeapVector {
        HeapVector { pointer: self.pointer, size: self.size, value_types }
    }

    pub(crate) fn registers_count() -> usize {
        3
    }

    pub(crate) fn extract_registers(self) -> Vec<RegisterIndex> {
        vec![self.pointer, self.size, self.rc]
    }
}

/// The representation of a noir value in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) enum BrilligVariable {
    Simple(RegisterIndex),
    BrilligArray(BrilligArray),
    BrilligVector(BrilligVector),
}

impl BrilligVariable {
    pub(crate) fn extract_register(self) -> RegisterIndex {
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

    pub(crate) fn extract_registers(self) -> Vec<RegisterIndex> {
        match self {
            BrilligVariable::Simple(register_index) => vec![register_index],
            BrilligVariable::BrilligArray(array) => array.extract_registers(),
            BrilligVariable::BrilligVector(vector) => vector.extract_registers(),
        }
    }

    pub(crate) fn to_register_or_memory(self, typ: &Type) -> RegisterOrMemory {
        match self {
            BrilligVariable::Simple(register_index) => {
                RegisterOrMemory::RegisterIndex(register_index)
            }
            BrilligVariable::BrilligArray(array) => {
                let value_types = heap_value_types_of_array(typ);
                RegisterOrMemory::HeapArray(array.to_heap_array(value_types))
            }
            BrilligVariable::BrilligVector(vector) => {
                let value_types = heap_value_types_of_slice(typ);
                RegisterOrMemory::HeapVector(vector.to_heap_vector(value_types))
            }
        }
    }
}

fn type_to_heap_value_type(typ: &Type) -> HeapValueType {
    match typ {
        Type::Numeric(_) | Type::Reference(_) | Type::Function => HeapValueType::Simple,
        Type::Array(elem_type, size) => HeapValueType::Array {
            value_types: elem_type.as_ref().iter().map(type_to_heap_value_type).collect(),
            size: *size,
        },
        Type::Slice(elem_type) => HeapValueType::Vector {
            value_types: elem_type.as_ref().iter().map(type_to_heap_value_type).collect(),
        },
    }
}

fn heap_value_types_of_array(typ: &Type) -> Vec<HeapValueType> {
    if let Type::Array(elem_type, _) = typ {
        elem_type.as_ref().iter().map(type_to_heap_value_type).collect()
    } else {
        unreachable!("value is not of type Array");
    }
}

fn heap_value_types_of_slice(typ: &Type) -> Vec<HeapValueType> {
    if let Type::Slice(elem_type) = typ {
        elem_type.as_ref().iter().map(type_to_heap_value_type).collect()
    } else {
        unreachable!("value is not of type Slice");
    }
}
