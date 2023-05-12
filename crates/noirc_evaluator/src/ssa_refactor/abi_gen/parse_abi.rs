use std::collections::{BTreeMap, BTreeSet};

use acvm::acir::native_types::Witness;
use iter_extended::btree_map;
use noirc_abi::{AbiType, AbiVisibility};

use crate::errors::RuntimeErrorKind;

pub struct IrGen;

impl IrGen {}

pub struct WitnessStuff;

impl WitnessStuff {
    pub fn add_witness_to_cs(&mut self) -> Witness {
        Witness(0)
    }
}

pub enum AbiProcessed {
    Multi(Vec<AbiProcessed>),
    Item { name: String, witnesses: Vec<Witness>, range_constraint_size: u16, is_public: bool },
}

fn param_to_var(
    name: &str,
    param_type: &AbiType,
    param_visibility: &AbiVisibility,
    witness_stuff: &mut WitnessStuff,
    ir_gen: &mut IrGen,
    //
    param_witnesses: BTreeMap<String, Vec<Witness>>,
    // This is the list of witness indices which are linked to public parameters.
    // Witnesses below `num_witnesses_abi_len` and not included in this set
    // correspond to private parameters and must not be made public.
    public_parameters: BTreeSet<Witness>,
) -> Result<(), RuntimeErrorKind> {
    let witnesses = match param_type {
        AbiType::Field => {
            let witness = witness_stuff.add_witness_to_cs();
            // ir_gen.create_new_variable(
            //     name.to_owned(),
            //     Some(def),
            //     ObjectType::native_field(),
            //     Some(witness),
            // );
            vec![witness]
        }
        AbiType::Array { length, typ } => {
            let witnesses = generate_array_witnesses(length, typ)?;

            ir_gen.abi_array(name, Some(def), typ.as_ref(), *length, &witnesses);
            witnesses
        }
        AbiType::Integer { sign: _, width } => {
            let witness = witness_stuff.add_witness_to_cs();
            // ssa::acir_gen::range_constraint(witness, *width, self)?;
            // let obj_type = ir_gen.get_object_type_from_abi(param_type); // Fetch signedness of the integer
            // ir_gen.create_new_variable(name.to_owned(), Some(def), obj_type, Some(witness));

            vec![witness]
        }
        AbiType::Boolean => {
            let witness = witness_stuff.add_witness_to_cs();
            // ssa::acir_gen::range_constraint(witness, 1, self)?;
            // let obj_type = ObjectType::boolean();
            // ir_gen.create_new_variable(name.to_owned(), Some(def), obj_type, Some(witness));

            vec![witness]
        }
        AbiType::Struct { fields } => {
            let new_fields = btree_map(fields, |(inner_name, value)| {
                let new_name = format!("{name}.{inner_name}");
                (new_name, value.clone())
            });

            let mut struct_witnesses: BTreeMap<String, Vec<Witness>> = BTreeMap::new();
            self.generate_struct_witnesses(&mut struct_witnesses, &new_fields)?;

            ir_gen.abi_struct(name, Some(def), fields, &struct_witnesses);
            struct_witnesses.values().flatten().copied().collect()
        }
        AbiType::String { length } => {
            let typ = AbiType::Integer { sign: noirc_abi::Sign::Unsigned, width: 8 };
            let witnesses = self.generate_array_witnesses(length, &typ)?;
            ir_gen.abi_array(name, Some(def), &typ, *length, &witnesses);
            witnesses
        }
    };

    if param_visibility == &AbiVisibility::Public {
        self.public_parameters.extend(witnesses.clone());
    }
    self.param_witnesses.insert(name.to_owned(), witnesses);

    Ok(())
}

fn generate_struct_witnesses(
    witness_stuff: &mut WitnessStuff,
    struct_witnesses: &mut BTreeMap<String, Vec<Witness>>,
    fields: &BTreeMap<String, AbiType>,
) -> Result<(), RuntimeErrorKind> {
    for (name, typ) in fields {
        match typ {
            AbiType::Integer { width, .. } => {
                let witness = witness_stuff.add_witness_to_cs();
                struct_witnesses.insert(name.clone(), vec![witness]);
                // ssa::acir_gen::range_constraint(witness, *width, self)?;
            }
            AbiType::Boolean => {
                let witness = witness_stuff.add_witness_to_cs();
                struct_witnesses.insert(name.clone(), vec![witness]);
                // ssa::acir_gen::range_constraint(witness, 1, self)?;
            }
            AbiType::Field => {
                let witness = witness_stuff.add_witness_to_cs();
                struct_witnesses.insert(name.clone(), vec![witness]);
            }
            AbiType::Array { length, typ } => {
                let internal_arr_witnesses = self.generate_array_witnesses(length, typ)?;
                struct_witnesses.insert(name.clone(), internal_arr_witnesses);
            }
            AbiType::Struct { fields, .. } => {
                let mut new_fields: BTreeMap<String, AbiType> = BTreeMap::new();
                for (inner_name, value) in fields {
                    let new_name = format!("{name}.{inner_name}");
                    new_fields.insert(new_name, value.clone());
                }
                self.generate_struct_witnesses(struct_witnesses, &new_fields)?;
            }
            AbiType::String { length } => {
                let typ = AbiType::Integer { sign: noirc_abi::Sign::Unsigned, width: 8 };
                let internal_str_witnesses = self.generate_array_witnesses(length, &typ)?;
                struct_witnesses.insert(name.clone(), internal_str_witnesses);
            }
        }
    }
    Ok(())
}

fn generate_array_witnesses(
    &mut self,
    length: &u64,
    typ: &AbiType,
) -> Result<Vec<Witness>, RuntimeErrorKind> {
    let mut witnesses = Vec::new();
    let element_width = match typ {
        AbiType::Integer { width, .. } => Some(*width),
        _ => None,
    };
    for _ in 0..*length {
        let witness = witness_stuff.add_witness_to_cs();
        witnesses.push(witness);
        if let Some(ww) = element_width {
            // ssa::acir_gen::range_constraint(witness, ww, self)?;
        }
    }
    Ok(witnesses)
}

/// The ABI is the intermediate representation between Noir and types like Toml
/// Noted in the noirc_abi, it is possible to convert Toml -> NoirTypes
/// However, this intermediate representation is useful as it allows us to have
/// intermediate Types which the core type system does not know about like Strings.
fn parse_abi_alt(&mut self, ir_gen: &mut IrGenerator) {
    // XXX: Currently, the syntax only supports public witnesses
    // u8 and arrays are assumed to be private
    // This is not a short-coming of the ABI, but of the grammar
    // The new grammar has been conceived, and will be implemented.
    let main = ir_gen.program.main_mut();
    let main_params = std::mem::take(&mut main.parameters);
    let abi_params = std::mem::take(&mut ir_gen.program.main_function_signature.0);

    assert_eq!(main_params.len(), abi_params.len());

    for ((param_id, _, param_name, _), abi_param) in main_params.iter().zip(abi_params) {
        assert_eq!(param_name, &abi_param.name);
        let def = Definition::Local(*param_id);
        self.param_to_var(param_name, def, &abi_param.typ, &abi_param.visibility, ir_gen).unwrap();
    }

    // Store the number of witnesses used to represent the types
    // in the ABI
    self.num_witnesses_abi_len = self.current_witness_index as usize;
}
