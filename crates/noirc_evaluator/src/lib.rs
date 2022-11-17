mod binary_op;

mod builtin;
mod environment;
mod errors;
mod interpreter;
mod low_level_function_impl;
mod object;
mod ssa;

use acvm::acir::circuit::{
    gate::{AndGate, Gate, XorGate},
    Circuit, PublicInputs,
};
use acvm::acir::native_types::{Expression, Linear, Witness};
use acvm::FieldElement;
use acvm::Language;
use environment::{Environment, FuncContext};
use errors::{RuntimeError, RuntimeErrorKind};
use noirc_abi::{AbiFEType, AbiType};
use noirc_frontend::monomorphisation::ast::*;

use object::{Array, Integer, Object};
use ssa::{code_gen::IRGenerator, node};

pub struct Evaluator {
    // Why is this not u64?
    //
    // At the moment, wasm32 is being used in the default backend
    // so it is safer to use a u64, at least until clang is changed
    // to compile wasm64.
    current_witness_index: u32,
    public_inputs: Vec<Witness>,
    gates: Vec<Gate>,
}

/// Compiles the Program into ACIR and applies optimisations to the arithmetic gates
// XXX: We return the num_witnesses, but this is the max number of witnesses
// Some of these could have been removed due to optimisations. We need this number because the
// Standard format requires the number of witnesses. The max number is also fine.
// If we had a composer object, we would not need it
pub fn create_circuit(
    program: Program,
    np_language: Language,
    enable_logging: bool,
) -> Result<Circuit, RuntimeError> {
    let mut evaluator = Evaluator::new();

    // create a new environment for the main context
    let mut env = Environment::new(FuncContext::Main);

    // First evaluate the main function
    evaluator.evaluate_main_alt(&mut env, program, enable_logging)?;

    let witness_index = evaluator.current_witness_index();

    let optimised_circuit = acvm::compiler::compile(
        Circuit {
            current_witness_index: witness_index,
            gates: evaluator.gates,
            public_inputs: PublicInputs(evaluator.public_inputs),
        },
        np_language,
    );

    Ok(optimised_circuit)
}

impl Evaluator {
    fn new() -> Self {
        Evaluator {
            public_inputs: Vec::new(),
            // XXX: Barretenberg, reserves the first index to have value 0.
            // When we increment, we do not use this index at all.
            // This means that every constraint system at the moment, will either need
            // to decrease each index by 1, or create a dummy witness.
            //
            // We ideally want to not have this and have Barretenberg apply the
            // following transformation to the witness index : f(i) = i + 1
            //
            current_witness_index: 0,
            gates: Vec::new(),
        }
    }

    // Creates a new Witness index
    fn add_witness_to_cs(&mut self) -> Witness {
        self.current_witness_index += 1;
        Witness(self.current_witness_index)
    }

    // Maps a variable name to a witness index
    fn add_witness_to_env(
        &mut self,
        variable_name: String,
        witness: Witness,
        env: &mut Environment,
    ) -> Object {
        let value = Object::from_witness(witness);
        env.store(variable_name, value.clone());
        value
    }

    pub fn current_witness_index(&self) -> u32 {
        self.current_witness_index
    }

    /// Compiles the AST into the intermediate format by evaluating the main function
    pub fn evaluate_main_alt(
        &mut self,
        env: &mut Environment,
        program: Program,
        enable_logging: bool,
    ) -> Result<(), RuntimeError> {
        let mut igen = IRGenerator::new(program);
        self.parse_abi_alt(&mut igen);

        // Now call the main function
        igen.codegen_main(env)?;

        //Generates ACIR representation:
        igen.context.ir_to_acir(self, enable_logging)?;
        Ok(())
    }

    // When we are multiplying arithmetic gates by each other, if one gate has too many terms
    // It is better to create an intermediate variable which links to the gate and then multiply by that intermediate variable
    // instead
    pub fn create_intermediate_variable(
        &mut self,
        arithmetic_gate: Expression,
    ) -> (Object, Witness) {
        // Create a unique witness name and add witness to the constraint system
        let inter_var_witness = self.add_witness_to_cs();
        let inter_var_object = Object::from_witness(inter_var_witness);

        // Link that witness to the arithmetic gate
        let constraint = &arithmetic_gate - &inter_var_witness;
        self.gates.push(Gate::Arithmetic(constraint));
        (inter_var_object, inter_var_witness)
    }

    fn param_to_var(
        &mut self,
        name: &str,
        def: DefinitionId,
        param_type: &AbiType,
        igen: &mut IRGenerator,
    ) -> Result<(), RuntimeErrorKind> {
        dbg!("inside param_to_var, {:?}, {:?}", name, def);
        dbg!(param_type);
        match param_type {
            AbiType::Field(visibility) => {
                let witness = self.add_witness_to_cs();
                if *visibility == AbiFEType::Public {
                    self.public_inputs.push(witness);
                }
                igen.abi_var(name, def, node::ObjectType::NativeField, witness);
            }
            AbiType::Array { visibility, length, typ } => {
                let mut witnesses = Vec::new();
                let mut element_width = None;
                if let AbiType::Integer { width, .. } = typ.as_ref() {
                    element_width = Some(*width);
                }
                for _ in 0..*length {
                    let witness = self.add_witness_to_cs();
                    witnesses.push(witness);
                    if let Some(ww) = element_width {
                        ssa::acir_gen::range_constraint(witness, ww, self)?;
                    }
                    if *visibility == AbiFEType::Public {
                        self.public_inputs.push(witness);
                    }
                }
                igen.abi_array(name, def, typ.as_ref(), *length, witnesses);
            }
            AbiType::Integer { visibility, sign, width } => {
                let witness = self.add_witness_to_cs();
                ssa::acir_gen::range_constraint(witness, *width, self)?;
                if *visibility == AbiFEType::Public {
                    self.public_inputs.push(witness);
                }
                match sign {
                    noirc_abi::Sign::Unsigned => {
                        igen.abi_var(name, def, node::ObjectType::Unsigned(*width), witness)
                    }
                    noirc_abi::Sign::Signed => {
                        igen.abi_var(name, def, node::ObjectType::Signed(*width), witness)
                    }
                }
            }
            AbiType::Struct { visibility, num_fields, fields } => {
                let mut witnesses = Vec::new();
                for (key, val) in fields {
                    let witness = self.add_witness_to_cs();
                    witnesses.push(witness);
                    // self.param_to_var(key, def, val, igen);
                    // igen.create_new_value(field, &name, None);
                    if *visibility == AbiFEType::Public {
                        self.public_inputs.push(witness);
                    }
                }
                // TODO: possible add an abi_struct method that recursively calls all the others igen.abi_* funcs
                igen.abi_struct(name, def, fields, witnesses);
            }
            _ => todo!(), // TODO: this is still used by parse_abi_alt
        }
        Ok(())
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
        assert_eq!(main_params.len(), abi_params.len());

        for ((param_id, _, param_name1, _), (param_name2, param_type)) in
            main_params.iter().zip(abi_params)
        {
            assert_eq!(param_name1, &param_name2);
            self.param_to_var(param_name1, *param_id, &param_type, igen).unwrap();
        }
    }
}
