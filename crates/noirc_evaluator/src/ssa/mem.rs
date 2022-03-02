use super::code_gen::IRGenerator;
use super::node::{self, Node};
use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use noirc_frontend::node_interner::IdentId;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::convert::TryFrom;

use crate::Array;
use std::convert::TryInto;

pub struct Memory {
    pub arrays: Vec<MemArray>,
    pub last_adr: u32,                          //last address in 'memory'
    pub memory_map: HashMap<u32, arena::Index>, //maps memory adress to expression
}

#[derive(Debug)]
pub struct MemArray {
    pub element_type: node::ObjectType, //type of elements
    pub witness: Vec<Witness>,
    pub name: String,
    pub def: IdentId,
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

    pub fn new(definition: IdentId, name: &str, of: node::ObjectType, len: u32) -> MemArray {
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
    pub fn new() -> Memory {
        Memory {
            arrays: Vec::new(),
            last_adr: 0,
            memory_map: HashMap::new(),
        }
    }

    pub fn find_array(&self, definition: &Option<IdentId>) -> Option<&MemArray> {
        if let Some(def) = definition {
            return self.arrays.iter().find(|a| a.def == *def);
        }
        None
    }

    pub fn get_array_index(&self, array: &MemArray) -> Option<u32> {
        if let Some(p) = self.arrays.iter().position(|x| x.def == array.def) {
            return Some(p as u32);
        }
        None
    }

    //dereference a pointer
    pub fn deref(igen: &IRGenerator, idx: arena::Index) -> Option<u32> {
        if let Some(var) = igen.get_object(idx) {
            match var.get_type() {
                node::ObjectType::Pointer(a) => Some(a),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn create_array(
        &mut self,
        array: &Array,
        definition: IdentId,
        el_type: node::ObjectType,
        arr_name: &str,
    ) -> &MemArray {
        // let arr_name = context.def_interner.ident_name(collection);
        // let ident_span = context.def_interner.ident_span(collection);
        // let arr = env.get_array(&arr_name).map_err(|kind|kind.add_span(ident_span)).unwrap();
        // let arr_type = context.def_interner.id_type(arr_def.unwrap());
        // let o_type = node::ObjectType::from_type(arr_type);
        let len = u32::try_from(array.length).unwrap();
        let mut new_array = MemArray::new(definition, arr_name, el_type, len);
        new_array.adr = self.last_adr;
        new_array.set_witness(array);
        self.arrays.push(new_array);
        self.last_adr += len;
        self.arrays.last().unwrap()
    }

    pub fn create_new_array(
        &mut self,
        len: u32,
        el_type: node::ObjectType,
        arr_name: String,
    ) -> u32 {
        let mut new_array = MemArray::new(IdentId::dummy_id(), &arr_name, el_type, len);
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

    pub fn to_u32(eval: &IRGenerator, idx: arena::Index) -> Option<u32> {
        if let Some(index_as_constant) = eval.get_as_constant(idx) {
            if let Ok(address) = index_as_constant.to_u128().try_into() {
                return Some(address);
            }
            //Invalid memory address
        }
        None //Not a constant object
    }
}
