use crate::ssa::{
    acir_gen::InternalVar,
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
pub struct Memory {
    arrays: Vec<MemArray>,
    pub last_adr: u32,                    //last address in 'memory'
    pub memory_map: HashMap<u32, NodeId>, //maps memory address to expression
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayId(u32);

impl ArrayId {
    pub fn dummy() -> ArrayId {
        ArrayId(std::u32::MAX)
    }
}

#[derive(Debug, Clone)]
pub struct MemArray {
    pub id: ArrayId,
    pub element_type: node::ObjectType, //type of elements
    pub values: Vec<InternalVar>,
    pub name: String,
    pub def: Definition,
    pub len: u32,     //number of elements
    pub adr: u32,     //base address of the array
    pub max: BigUint, //Max possible value of array elements
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
            values: Vec::new(),
            def: definition,
            len,
            adr: 0,
            max: of.max_size(),
        }
    }

    pub fn absolute_adr(&self, idx: u32) -> u32 {
        self.adr + idx
    }
}

impl Memory {
    /// Retrieves the ArrayId of the last array in Memory.
    /// Panics if self does not contain at least 1 array.
    pub fn last_id(&self) -> ArrayId {
        ArrayId(self.arrays.len() as u32 - 1)
    }

    //dereference a pointer
    pub fn deref(ctx: &SsaContext, id: NodeId) -> Option<ArrayId> {
        ctx.try_get_node(id).and_then(|var| match var.get_type() {
            node::ObjectType::Pointer(a) => Some(a),
            _ => None,
        })
    }

    pub fn create_new_array(
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

    pub fn as_u32(value: FieldElement) -> u32 {
        let big_v = BigUint::from_bytes_be(&value.to_be_bytes());
        let mut modulus = BigUint::from(2_u32);
        modulus = modulus.pow(32);
        let result = big_v % modulus;
        result.to_u32().unwrap()
    }

    pub fn to_u32(ctx: &SsaContext, id: NodeId) -> Option<u32> {
        if let Some(index_as_constant) = ctx.get_as_constant(id) {
            if let Ok(address) = index_as_constant.to_u128().try_into() {
                return Some(address);
            }
            //Invalid memory address
        }
        None //Not a constant object
    }

    //returns the value of the element array[index], if it exists in the memory_map
    pub fn get_value_from_map(&self, array_id: ArrayId, index: u32) -> Option<&NodeId> {
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
