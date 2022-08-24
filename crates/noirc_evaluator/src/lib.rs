mod binary_op;

mod builtin;
mod environment;
mod errors;
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
use noirc_abi::AbiFEType;
use noirc_errors::Location;
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
    program: Functions,
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
        program: Functions,
        enable_logging: bool,
    ) -> Result<(), RuntimeError> {
        let mut igen = IRGenerator::new(program);
        self.parse_abi_alt(&mut igen)?;

        // Now call the main function
        igen.codegen_main(env)?;

        //Generates ACIR representation:
        igen.context.ir_to_acir(self, enable_logging)?;
        Ok(())
    }

    fn param_to_var(
        &mut self,
        name: &str,
        def: DefinitionId,
        param_visibility: AbiFEType,
        param_type: &Type,
        param_location: Location,
        igen: &mut IRGenerator,
    ) -> Result<(), RuntimeError> {
        match param_type {
            Type::Field => {
                let witness = self.add_witness_to_cs();
                if param_visibility == AbiFEType::Public {
                    self.public_inputs.push(witness);
                }
                igen.abi_var(name, def, node::ObjectType::NativeField, witness);
            }
            Type::Array(len, typ) => {
                let mut witnesses = Vec::new();
                let mut element_width = None;
                if let Type::Integer(_, width) = typ.as_ref() {
                    element_width = Some(*width);
                }

                for _ in 0..*len {
                    let witness = self.add_witness_to_cs();
                    witnesses.push(witness);
                    if let Some(ww) = element_width {
                        ssa::acir_gen::range_constraint(witness, ww, self)
                            .map_err(|e| e.add_location(param_location))?;
                    }
                    if param_visibility == AbiFEType::Public {
                        self.public_inputs.push(witness);
                    }
                }
                igen.abi_array(name, def, *typ.clone(), *len as u128, witnesses);
            }
            Type::Integer(sign, width) => {
                let witness = self.add_witness_to_cs();
                ssa::acir_gen::range_constraint(witness, *width, self)
                    .map_err(|e| e.add_location(param_location))?;
                if param_visibility == AbiFEType::Public {
                    self.public_inputs.push(witness);
                }
                match sign {
                    noirc_frontend::Signedness::Unsigned => {
                        igen.abi_var(name, def, node::ObjectType::Unsigned(*width), witness)
                    }
                    noirc_frontend::Signedness::Signed => {
                        igen.abi_var(name, def, node::ObjectType::Signed(*width), witness)
                    }
                }
            }

            Type::Bool => todo!(),
            Type::Unit => todo!(),
            Type::Tuple(_) => todo!(),
            _ => unreachable!(),
        }
        Ok(())
    }

    /// The ABI is the intermediate representation between Noir and types like Toml
    /// Noted in the noirc_abi, it is possible to convert Toml -> NoirTypes
    /// However, this intermediate representation is useful as it allows us to have
    /// intermediate Types which the core type system does not know about like Strings.
    fn parse_abi_alt(&mut self, igen: &mut IRGenerator) -> Result<(), RuntimeError> {
        // XXX: Currently, the syntax only supports public witnesses
        // u8 and arrays are assumed to be private
        // This is not a short-coming of the ABI, but of the grammar
        // The new grammar has been conceived, and will be implemented.
        for (param_name, param_type) in &igen.program.abi.parameters {
            let name = todo!();
            let vis = todo!();
            let location = todo!();
            self.param_to_var(name, *param_id, vis, param_type, location, igen)?;
        }

        Ok(())
    }
}
