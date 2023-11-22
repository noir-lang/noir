use acvm::brillig_vm::brillig::{HeapArray, HeapVector, RegisterIndex, RegisterOrMemory};
use serde::{Deserialize, Serialize};

/// The representation of a noir array in the Brillig IR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub(crate) struct BrilligArray {
    pub(crate) pointer: RegisterIndex,
    pub(crate) size: usize,
    pub(crate) rc: RegisterIndex,
}

impl BrilligArray {
    pub(crate) fn to_heap_array(self) -> HeapArray {
        HeapArray { pointer: self.pointer, size: self.size }
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
    pub(crate) fn to_heap_vector(self) -> HeapVector {
        HeapVector { pointer: self.pointer, size: self.size }
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

    pub(crate) fn to_register_or_memory(self) -> RegisterOrMemory {
        match self {
            BrilligVariable::Simple(register_index) => {
                RegisterOrMemory::RegisterIndex(register_index)
            }
            BrilligVariable::BrilligArray(array) => {
                RegisterOrMemory::HeapArray(array.to_heap_array())
            }
            BrilligVariable::BrilligVector(vector) => {
                RegisterOrMemory::HeapVector(vector.to_heap_vector())
            }
        }
    }
}
