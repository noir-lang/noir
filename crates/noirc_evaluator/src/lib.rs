mod errors;
mod ssa;

use acvm::acir::circuit::{opcodes::Opcode as AcirOpcode, Circuit, PublicInputs};
use acvm::acir::native_types::{Expression, Witness};
use acvm::compiler::fallback::IsBlackBoxSupported;
use acvm::Language;
use errors::{RuntimeError, RuntimeErrorKind};
use iter_extended::btree_map;
use noirc_abi::{AbiType, AbiVisibility};
use noirc_frontend::monomorphisation::ast::*;
use std::collections::BTreeMap;

use ssa::{code_gen::IRGenerator, node};

pub struct Evaluator {
    // Why is this not u64?
    //
    // At the moment, wasm32 is being used in the default backend
    // so it is safer to use a u32, at least until clang is changed
    // to compile wasm64.
    current_witness_index: u32,
    // This is the number of witnesses indices used when
    // creating the private/public inputs of the ABI.
    num_witnesses_abi_len: usize,
    public_inputs: Vec<Witness>,
    opcodes: Vec<AcirOpcode>,
}

/// Compiles the Program into ACIR and applies optimisations to the arithmetic gates
// XXX: We return the num_witnesses, but this is the max number of witnesses
// Some of these could have been removed due to optimisations. We need this number because the
// Standard format requires the number of witnesses. The max number is also fine.
// If we had a composer object, we would not need it
pub fn create_circuit(
    program: Program,
    np_language: Language,
    is_blackbox_supported: IsBlackBoxSupported,
    enable_logging: bool,
) -> Result<Circuit, RuntimeError> {
    let mut evaluator = Evaluator::new();

    // First evaluate the main function
    evaluator.evaluate_main_alt(program, enable_logging)?;

    let witness_index = evaluator.current_witness_index();

    let optimised_circuit = acvm::compiler::compile(
        Circuit {
            current_witness_index: witness_index,
            opcodes: evaluator.opcodes,
            public_inputs: PublicInputs(evaluator.public_inputs),
        },
        np_language,
        is_blackbox_supported,
    )
    .map_err(|_| RuntimeErrorKind::Spanless(String::from("produced an acvm compile error")))?;

    Ok(optimised_circuit)
}

impl Evaluator {
    fn new() -> Self {
        Evaluator {
            public_inputs: Vec::new(),
            num_witnesses_abi_len: 0,
            // XXX: Barretenberg, reserves the first index to have value 0.
            // When we increment, we do not use this index at all.
            // This means that every constraint system at the moment, will either need
            // to decrease each index by 1, or create a dummy witness.
            //
            // We ideally want to not have this and have Barretenberg apply the
            // following transformation to the witness index : f(i) = i + 1
            //
            current_witness_index: 0,
            opcodes: Vec::new(),
        }
    }

    // Returns true if the `witness_index`
    // was created in the ABI as a private input.
    //
    // Note: This method is used so that we don't convert private
    // ABI inputs into public outputs.
    fn is_private_abi_input(&self, witness_index: Witness) -> bool {
        // If the `witness_index` is more than the `num_witnesses_abi_len`
        // then it was created after the ABI was processed and is therefore
        // an intermediate variable.
        let is_intermediate_variable = witness_index.as_usize() > self.num_witnesses_abi_len;

        let is_public_input = self.public_inputs.contains(&witness_index);

        !is_intermediate_variable && !is_public_input
    }

    // Creates a new Witness index
    fn add_witness_to_cs(&mut self) -> Witness {
        self.current_witness_index += 1;
        Witness(self.current_witness_index)
    }

    pub fn current_witness_index(&self) -> u32 {
        self.current_witness_index
    }

    /// Compiles the AST into the intermediate format by evaluating the main function
    pub fn evaluate_main_alt(
        &mut self,
        program: Program,
        enable_logging: bool,
    ) -> Result<(), RuntimeError> {
        let mut igen = IRGenerator::new(program);
        self.parse_abi_alt(&mut igen);

        // Now call the main function
        igen.codegen_main()?;

        //Generates ACIR representation:
        igen.context.ir_to_acir(self, enable_logging)?;
        Ok(())
    }

    // When we are multiplying arithmetic gates by each other, if one gate has too many terms
    // It is better to create an intermediate variable which links to the gate and then multiply by that intermediate variable
    // instead
    pub fn create_intermediate_variable(&mut self, arithmetic_gate: Expression) -> Witness {
        // Create a unique witness name and add witness to the constraint system
        let inter_var_witness = self.add_witness_to_cs();

        // Link that witness to the arithmetic gate
        let constraint = &arithmetic_gate - &inter_var_witness;
        self.opcodes.push(AcirOpcode::Arithmetic(constraint));
        inter_var_witness
    }

    fn param_to_var(
        &mut self,
        name: &str,
        def: Definition,
        param_type: &AbiType,
        visibility: &AbiVisibility,
        igen: &mut IRGenerator,
    ) -> Result<(), RuntimeErrorKind> {
        match param_type {
            AbiType::Field => {
                let witness = self.add_witness_to_cs();
                if *visibility == AbiVisibility::Public {
                    self.public_inputs.push(witness);
                }
                igen.create_new_variable(
                    name.to_owned(),
                    Some(def),
                    node::ObjectType::NativeField,
                    Some(witness),
                );
            }
            AbiType::Array { length, typ } => {
                let witnesses = self.generate_array_witnesses(length, typ)?;
                if *visibility == AbiVisibility::Public {
                    self.public_inputs.extend(witnesses.clone());
                }
                igen.abi_array(name, Some(def), typ.as_ref(), *length, witnesses);
            }
            AbiType::Integer { sign: _, width } => {
                let witness = self.add_witness_to_cs();
                ssa::acir_gen::range_constraint(witness, *width, self)?;
                if *visibility == AbiVisibility::Public {
                    self.public_inputs.push(witness);
                }
                let obj_type = igen.get_object_type_from_abi(param_type); // Fetch signedness of the integer
                igen.create_new_variable(name.to_owned(), Some(def), obj_type, Some(witness));
            }
            AbiType::Boolean => {
                let witness = self.add_witness_to_cs();
                ssa::acir_gen::range_constraint(witness, 1, self)?;
                if *visibility == AbiVisibility::Public {
                    self.public_inputs.push(witness);
                }
                let obj_type = node::ObjectType::Boolean;
                igen.create_new_variable(name.to_owned(), Some(def), obj_type, Some(witness));
            }
            AbiType::Struct { fields } => {
                let new_fields = btree_map(fields, |(inner_name, value)| {
                    let new_name = format!("{name}.{inner_name}");
                    (new_name, value.clone())
                });

                let mut struct_witnesses: BTreeMap<String, Vec<Witness>> = BTreeMap::new();
                self.generate_struct_witnesses(&mut struct_witnesses, &new_fields)?;
                if *visibility == AbiVisibility::Public {
                    let witnesses: Vec<Witness> =
                        struct_witnesses.values().flatten().cloned().collect();
                    self.public_inputs.extend(witnesses);
                }
                igen.abi_struct(name, Some(def), fields, struct_witnesses);
            }
            AbiType::String { length } => {
                let typ = AbiType::Integer { sign: noirc_abi::Sign::Unsigned, width: 8 };
                let witnesses = self.generate_array_witnesses(length, &typ)?;
                if *visibility == AbiVisibility::Public {
                    self.public_inputs.extend(witnesses.clone());
                }
                igen.abi_array(name, Some(def), &typ, *length, witnesses);
            }
        }
        Ok(())
    }

    fn generate_struct_witnesses(
        &mut self,
        struct_witnesses: &mut BTreeMap<String, Vec<Witness>>,
        fields: &BTreeMap<String, AbiType>,
    ) -> Result<(), RuntimeErrorKind> {
        for (name, typ) in fields {
            match typ {
                AbiType::Integer { width, .. } => {
                    let witness = self.add_witness_to_cs();
                    struct_witnesses.insert(name.clone(), vec![witness]);
                    ssa::acir_gen::range_constraint(witness, *width, self)?;
                }
                AbiType::Boolean => {
                    let witness = self.add_witness_to_cs();
                    struct_witnesses.insert(name.clone(), vec![witness]);
                    ssa::acir_gen::range_constraint(witness, 1, self)?;
                }
                AbiType::Field => {
                    let witness = self.add_witness_to_cs();
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
                    self.generate_struct_witnesses(struct_witnesses, &new_fields)?
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
            let witness = self.add_witness_to_cs();
            witnesses.push(witness);
            if let Some(ww) = element_width {
                ssa::acir_gen::range_constraint(witness, ww, self)?;
            }
        }
        Ok(witnesses)
    }

    /// The ABI is the intermediate representation between Noir and types like Toml
    /// Noted in the noirc_abi, it is possible to convert Toml -> NoirTypes
    /// However, this intermediate representation is useful as it allows us to have
    /// intermediate Types which the core type system does not know about like Strings.
    fn parse_abi_alt(&mut self, igen: &mut IRGenerator) {
        // XXX: Currently, the syntax only supports public witnesses
        // u8 and arrays are assumed to be private
        // This is not a short-coming of the ABI, but of the grammar
        // The new grammar has been conceived, and will be implemented.
        let main = igen.program.main();
        let main_params = std::mem::take(&mut main.parameters);
        let abi_params = std::mem::take(&mut igen.program.abi.parameters);

        // Remove the return type from the parameters
        // Since this is not in the main functions parameters.
        //
        // TODO(See Issue633) regarding adding a `return_type` field to the ABI struct
        let abi_params: Vec<_> = abi_params
            .into_iter()
            .filter(|param| param.name != noirc_abi::MAIN_RETURN_NAME)
            .collect();

        assert_eq!(main_params.len(), abi_params.len());

        for ((param_id, _, param_name, _), abi_param) in main_params.iter().zip(abi_params) {
            assert_eq!(param_name, &abi_param.name);
            let def = Definition::Local(*param_id);
            self.param_to_var(param_name, def, &abi_param.typ, &abi_param.visibility, igen)
                .unwrap();
        }

        // Store the number of witnesses used to represent the types
        // in the ABI
        self.num_witnesses_abi_len = self.current_witness_index as usize;
    }
}
