use crate::ssa::{
    context::SsaContext,
    node,
    node::{Node, NodeId},
};
use acvm::FieldElement;
use noirc_frontend::monomorphization::ast::{Definition, LocalId};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct Memory {
    arrays: Vec<MemArray>,
    pub(crate) last_adr: u32,                    //last address in 'memory'
    pub(crate) memory_map: HashMap<u32, NodeId>, //maps memory address to expression
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ArrayId(u32);

impl ArrayId {
    pub(crate) fn dummy() -> ArrayId {
        ArrayId(std::u32::MAX)
    }

    pub(crate) fn as_u32(&self) -> u32 {
        self.0
    }
}

/// MemArray represents a contiguous array of elements of the same type.
#[derive(Debug, Clone)]
pub(crate) struct MemArray {
    /// The unique identifier of a `MemArray` instance.
    pub(crate) id: ArrayId,
    /// The type of each array element. All elements of a `MemArray` are of
    /// the same type.
    pub(crate) element_type: node::ObjectType,
    /// The name of the variable to which the array is assigned.
    pub(crate) name: String,
    /// A reference to where the array is defined.
    pub(crate) def: Definition,
    /// The number of elements in the array.
    pub(crate) len: u32,
    /// The base address of the array.
    pub(crate) adr: u32,
    /// The max possible value of each element.
    pub(crate) max: BigUint,
}

impl MemArray {
    fn new(
        id: ArrayId,
        definition: Definition,
        name: &str,
        of: node::ObjectType,
        len: u32,
    ) -> MemArray {
        assert!(len > 0);
        MemArray {
            id,
            element_type: of,
            name: name.to_string(),
            def: definition,
            len,
            adr: 0,
            max: of.max_size(),
        }
    }

    pub(super) fn absolute_adr(&self, idx: u32) -> u32 {
        self.adr + idx
    }
}

impl Memory {
    /// Retrieves the ArrayId of the last array in Memory.
    /// Panics if self does not contain at least 1 array.
    pub(super) fn last_id(&self) -> ArrayId {
        ArrayId(self.arrays.len() as u32 - 1)
    }

    //dereference a pointer
    pub(super) fn deref(ctx: &SsaContext, id: NodeId) -> Option<ArrayId> {
        ctx.try_get_node(id).and_then(|var| match var.get_type() {
            node::ObjectType::ArrayPointer(a) => Some(a),
            _ => None,
        })
    }

    pub(super) fn create_new_array(
        &mut self,
        len: u32,
        el_type: node::ObjectType,
        arr_name: &str,
    ) -> ArrayId {
        let id = ArrayId(self.arrays.len() as u32);
        let dummy_id = Definition::Local(LocalId(u32::MAX));
        let mut new_array = MemArray::new(id, dummy_id, arr_name, el_type, len);
        new_array.adr = self.last_adr;
        self.arrays.push(new_array);
        self.last_adr += len;
        id
    }
    /// Coerces a FieldElement into a u32
    /// By taking its value modulo 2^32
    ///
    /// See issue #785 on whether this is safe
    pub(super) fn as_u32(value: FieldElement) -> u32 {
        let big_v = BigUint::from_bytes_be(&value.to_be_bytes());
        let mut modulus = BigUint::from(2_u32);
        modulus = modulus.pow(32);
        let result = big_v % modulus;
        result.to_u32().unwrap()
    }

    pub(super) fn to_u32(ctx: &SsaContext, id: NodeId) -> Option<u32> {
        if let Some(index_as_constant) = ctx.get_as_constant(id) {
            if let Ok(address) = index_as_constant.to_u128().try_into() {
                return Some(address);
            }
            //Invalid memory address
        }
        None //Not a constant object
    }

    //returns the value of the element array[index], if it exists in the memory_map
    pub(super) fn get_value_from_map(&self, array_id: ArrayId, index: u32) -> Option<&NodeId> {
        let adr = self[array_id].absolute_adr(index);
        self.memory_map.get(&adr)
    }
}

impl std::ops::Index<ArrayId> for Memory {
    type Output = MemArray;

    fn index(&self, index: ArrayId) -> &Self::Output {
        &self.arrays[index.0 as usize]
    }
}

impl std::ops::IndexMut<ArrayId> for Memory {
    fn index_mut(&mut self, index: ArrayId) -> &mut Self::Output {
        &mut self.arrays[index.0 as usize]
    }
}
