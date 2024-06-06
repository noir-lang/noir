use acvm::{
    acir::AcirField,
    brillig_vm::brillig::{HeapArray, HeapValueType, HeapVector, MemoryAddress, ValueOrArray},
    FieldElement,
};
use serde::{Deserialize, Serialize};

use crate::ssa::ir::types::Type;

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
    SingleAddr(SingleAddrVariable),
    BrilligArray(BrilligArray),
    BrilligVector(BrilligVector),
}

impl BrilligVariable {
    pub(crate) fn extract_single_addr(self) -> SingleAddrVariable {
        match self {
            BrilligVariable::SingleAddr(single_addr) => single_addr,
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
            BrilligVariable::SingleAddr(single_addr) => vec![single_addr.address],
            BrilligVariable::BrilligArray(array) => array.extract_registers(),
            BrilligVariable::BrilligVector(vector) => vector.extract_registers(),
        }
    }

    pub(crate) fn to_value_or_array(self) -> ValueOrArray {
        match self {
            BrilligVariable::SingleAddr(single_addr) => {
                ValueOrArray::MemoryAddress(single_addr.address)
            }
            BrilligVariable::BrilligArray(array) => ValueOrArray::HeapArray(array.to_heap_array()),
            BrilligVariable::BrilligVector(vector) => {
                ValueOrArray::HeapVector(vector.to_heap_vector())
            }
        }
    }
}

pub(crate) fn type_to_heap_value_type(typ: &Type) -> HeapValueType {
    match typ {
        Type::Numeric(_) | Type::Reference(_) | Type::Function => {
            HeapValueType::Simple(get_bit_size_from_ssa_type(typ))
        }
        Type::Array(elem_type, size) => HeapValueType::Array {
            value_types: elem_type.as_ref().iter().map(type_to_heap_value_type).collect(),
            size: typ.element_size() * size,
        },
        Type::Slice(elem_type) => HeapValueType::Vector {
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
