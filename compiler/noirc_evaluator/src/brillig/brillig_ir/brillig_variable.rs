use acvm::brillig_vm::brillig::{HeapArray, HeapVector, RegisterIndex, RegisterOrMemory};
use serde::{Deserialize, Serialize};

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
}

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
}

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
            BrilligVariable::BrilligArray(array) => vec![array.pointer, array.rc],
            BrilligVariable::BrilligVector(vector) => vec![vector.pointer, vector.size, vector.rc],
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
