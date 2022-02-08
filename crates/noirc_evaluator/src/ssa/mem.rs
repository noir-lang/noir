use acvm::acir::native_types::Witness;
use acvm::FieldElement;
use std::convert::TryFrom;
use std::collections::HashMap;
//use arena;
use super::super::environment::Environment;
use super::code_gen::IRGenerator;
use super::node;
use noirc_frontend::hir::Context;
use noirc_frontend::node_interner::IdentId;
use num_bigint::BigUint;
use num_traits::ToPrimitive;

use crate::Array;
use std::convert::TryInto;

pub struct Memory {
    pub arrays: Vec<MemArray>,
    pub last_adr: u32, //last address in 'memory'
    pub memory_map: HashMap<u32, arena::Index>, //maps memory adress to expression
}

pub struct MemArray {
    pub element_type: node::ObjectType, //type of elements
    pub witness: Vec<Witness>,
    pub name: String,
    pub def: IdentId,
    pub len: u32, //number of elements
    pub adr: u32, //base address of the array
    pub max: BigUint,      //Max possible value of array elements
}

impl MemArray {
    pub fn set_witness(&mut self, array: &Array) {
        for object in &array.contents {
            if let Some(w) = node::get_witness_from_object(object) {
                self.witness.push(w);
            }
        }
        assert!(self.witness.len() == 0 || self.witness.len() == self.len.try_into().unwrap());
    }

    pub fn new(definition: IdentId, name: String, of: node::ObjectType, len: u32) -> MemArray {
        assert!(len > 0);
        MemArray {
            element_type: of,
            name,
            witness: Vec::new(),
            def: definition,
            len,
            adr: 0,
            max: BigUint::from((1_u128 << of.bits()) - 1),
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

    pub fn create_array(
        &mut self,
        array: &Array,
        definition: IdentId,
        el_type: node::ObjectType,
        arr_name: String,
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
        &self.arrays.last().unwrap()
    }

    pub fn get_array_from_adr(&self, adr: u32) -> &MemArray {
        let mut cur_adr = 0;
        for a in &self.arrays {
            if cur_adr + a.len > adr {
                return a;
            }
            cur_adr += a.len;
        }
        unreachable!("Invalid memory");
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
        None   //Not a constant object
    }
}
