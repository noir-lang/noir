use acvm::{
    acir::{brillig::BitSize, AcirField},
    brillig_vm::brillig::{HeapValueType, MemoryAddress},
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
}

/// The representation of a noir slice in the Brillig IR
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

    pub(crate) fn extract_register(self) -> MemoryAddress {
        match self {
            BrilligVariable::SingleAddr(single_addr) => single_addr.address,
            BrilligVariable::BrilligArray(array) => array.pointer,
            BrilligVariable::BrilligVector(vector) => vector.pointer,
        }
    }
}

pub(crate) fn type_to_heap_value_type(typ: &Type) -> HeapValueType {
    match typ {
        Type::Numeric(_) | Type::Reference(_) | Type::Function => HeapValueType::Simple(
            BitSize::try_from_u32::<FieldElement>(get_bit_size_from_ssa_type(typ)).unwrap(),
        ),
        Type::Array(elem_type, size) => HeapValueType::Array {
            value_types: elem_type.as_ref().iter().map(type_to_heap_value_type).collect(),
            size: typ.element_size() * *size as usize,
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
