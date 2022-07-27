use super::acir_gen::InternalVar;
use super::context::SsaContext;
use super::node::{self, NodeId};
use acvm::FieldElement;
use noirc_frontend::node_interner::DefinitionId;
use noirc_frontend::ArraySize;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;

use crate::Array;
use std::convert::TryInto;

#[derive(Default)]
pub struct Memory {
    arrays: Vec<MemArray>,
    pub last_adr: u32,                    //last address in 'memory'
    pub memory_map: HashMap<u32, NodeId>, //maps memory adress to expression
}

#[derive(Debug, Clone)]
pub struct MemArray {
    pub element_type: node::ObjectType, //type of elements
    pub values: Vec<InternalVar>,
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
                self.values.push(w.into());
            }
        }
        assert!(self.values.is_empty() || self.values.len() == self.len.try_into().unwrap());
    }

    fn new(definition: DefinitionId, name: &str, of: node::ObjectType, len: u32) -> MemArray {
        assert!(len > 0);
        MemArray {
            element_type: of,
            name: name.to_string(),
            values: Vec::new(),
            def: definition,
            len,
            adr: 0,
            max: of.max_size(),
        }
    }
}

impl Memory {
    pub fn find_array(&self, definition: DefinitionId) -> Option<&MemArray> {
        self.arrays.iter().find(|a| a.def == definition)
    }

    pub fn create_new_array(&mut self, len: u32, el_type: node::ObjectType, arr_name: &str) {
        let mut new_array = MemArray::new(DefinitionId::dummy_id(), arr_name, el_type, len);
        new_array.adr = self.last_adr;
        self.arrays.push(new_array);
        self.last_adr += len;
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

pub fn get_array_size(array_size: &ArraySize) -> u32 {
    match array_size {
        ArraySize::Fixed(l) => *l as u32,
        ArraySize::Variable => todo!(),
    }
}
