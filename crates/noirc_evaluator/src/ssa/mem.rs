use super::context::SsaContext;
use super::node::{self, Node, NodeId};
use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use noirc_frontend::node_interner::DefinitionId;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::Array;
use std::convert::TryInto;

#[derive(Default)]
pub struct Memory {
    pub arrays: Vec<MemArray>,
    pub last_adr: u32,                    //last address in 'memory'
    pub memory_map: HashMap<u32, NodeId>, //maps memory adress to expression
}

#[derive(Debug)]
pub struct MemArray {
    pub element_type: node::ObjectType, //type of elements
    pub witness: Vec<Witness>,
    pub name: String,
    pub def: DefinitionId,
    pub len: u32,     //number of elements
    pub adr: u32,     //base address of the array
    pub max: BigUint, //Max possible value of array elements
}

impl MemArray {
    pub fn set_witness(&mut self, array: &Array) {
        for object in &array.contents {
            if let Some(w) = node::get_witness_from_object(object) {
                self.witness.push(w);
            }
        }
        assert!(self.witness.is_empty() || self.witness.len() == self.len.try_into().unwrap());
    }

    pub fn new(definition: DefinitionId, name: &str, of: node::ObjectType, len: u32) -> MemArray {
        assert!(len > 0);
        MemArray {
            element_type: of,
            name: name.to_string(),
            witness: Vec::new(),
            def: definition,
            len,
            adr: 0,
            max: of.max_size(),
        }
    }
}

impl Memory {
    pub fn find_array(&self, definition: &Option<DefinitionId>) -> Option<&MemArray> {
        definition.and_then(|def| self.arrays.iter().find(|a| a.def == def))
    }

    pub fn get_array_index(&self, array: &MemArray) -> Option<u32> {
        self.arrays
            .iter()
            .position(|x| x.def == array.def)
            .map(|p| p as u32)
    }

    //dereference a pointer
    pub fn deref(ctx: &SsaContext, id: NodeId) -> Option<u32> {
        ctx.try_get_node(id).and_then(|var| match var.get_type() {
            node::ObjectType::Pointer(a) => Some(a),
            _ => None,
        })
    }

    pub fn create_array_from_object(
        &mut self,
        array: &Array,
        definition: DefinitionId,
        el_type: node::ObjectType,
        arr_name: &str,
    ) -> &MemArray {
        let len = u32::try_from(array.length).unwrap();
        self.create_new_array(len, el_type, arr_name);
        let mem_array = self.arrays.last_mut().unwrap();
        mem_array.set_witness(array);
        mem_array.def = definition;
        self.arrays.last().unwrap()
    }

    pub fn create_new_array(&mut self, len: u32, el_type: node::ObjectType, arr_name: &str) -> u32 {
        let mut new_array = MemArray::new(DefinitionId::dummy_id(), arr_name, el_type, len);
        new_array.adr = self.last_adr;
        self.arrays.push(new_array);
        self.last_adr += len;
        (self.arrays.len() - 1) as u32
    }

    pub fn as_u32(value: FieldElement) -> u32 {
        let big_v = BigUint::from_bytes_be(&value.to_bytes());
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
}
