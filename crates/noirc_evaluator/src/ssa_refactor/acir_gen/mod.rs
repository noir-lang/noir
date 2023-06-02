//! This file holds the pass to convert from Noir's SSA IR to ACIR.

use std::collections::HashMap;

use self::acir_ir::{
    acir_variable::{AcirContext, AcirVar},
    errors::AcirGenError,
    memory::ArrayId,
};
use super::{
    abi_gen::collate_array_info,
    ir::{
        dfg::DataFlowGraph,
        function::RuntimeType,
        instruction::{
            Binary, BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction,
        },
        map::Id,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use crate::brillig::{artefact::BrilligArtefact, Brillig};
use noirc_abi::{AbiType, FunctionSignature, Sign};

pub(crate) use acir_ir::generated_acir::GeneratedAcir;

mod acir_ir;

/// Context struct for the acir generation pass.
/// May be similar to the Evaluator struct in the current SSA IR.
#[derive(Default)]
struct Context {
    /// Maps SSA values to `AcirVar`.
    ///
    /// This is needed so that we only create a single
    /// AcirVar per SSA value. Before creating an `AcirVar`
    /// for an SSA value, we check this map. If an `AcirVar`
    /// already exists for this Value, we return the `AcirVar`.
    ssa_value_to_acir_var: HashMap<Id<Value>, AcirVar>,
    /// Maps SSA values to array addresses (including index offset).
    ///
    /// When converting parameters the of main, `ArrayId`s are gathered and stored with an offset
    /// of 0. When the use of these stored values are detected for address arithmetic, the results
    /// of such instructions are stored, in effect capturing any further values that refer to
    /// addresses.
    ssa_value_to_array_address: HashMap<ValueId, (ArrayId, usize)>,
    /// Manages and builds the `AcirVar`s to which the converted SSA values refer.
    acir_context: AcirContext,
}

impl Ssa {
    pub(crate) fn into_acir(
        self,
        main_function_signature: FunctionSignature,
        brillig: Brillig,
        allow_log_ops: bool,
    ) -> GeneratedAcir {
        let param_arrays_info: Vec<_> = collate_array_info(&main_function_signature.0)
            .iter()
            .map(|(size, abi_type)| (*size, numeric_type_for_abi_array_element_type(abi_type)))
            .collect();

        let context = Context::default();
        context.convert_ssa(self, &param_arrays_info, brillig, allow_log_ops)
    }
}

/// Gives the equivalent ssa numeric type for the given abi type. We are dealing in the context of
/// arrays - hence why only numerics are supported.
fn numeric_type_for_abi_array_element_type(abi_type: &AbiType) -> NumericType {
    match abi_type {
        AbiType::Boolean => NumericType::Unsigned { bit_size: 1 },
        AbiType::Integer { sign, width } => match sign {
            Sign::Signed => NumericType::Signed { bit_size: *width },
            Sign::Unsigned => NumericType::Unsigned { bit_size: *width },
        },
        AbiType::Field => NumericType::NativeField,
        _ => unreachable!("Non-numeric cannot be array element"),
    }
}

impl Context {
    /// Converts SSA into ACIR
    fn convert_ssa(
        mut self,
        ssa: Ssa,
        param_array_info: &[(usize, NumericType)],
        brillig: Brillig,
        allow_log_ops: bool,
    ) -> GeneratedAcir {
        assert_eq!(
            ssa.functions.len(),
            1,
            "expected only a single function to be present with all other functions being inlined."
        );
        let main_func = ssa.main();
        let dfg = &main_func.dfg;
        let entry_block = &dfg[main_func.entry_block()];

        self.convert_ssa_block_params(entry_block.parameters(), dfg, param_array_info);

        for instruction_id in entry_block.instructions() {
            self.convert_ssa_instruction(*instruction_id, dfg, &ssa, &brillig, allow_log_ops);
        }

        self.convert_ssa_return(entry_block.terminator().unwrap(), dfg);

        self.acir_context.finish()
    }

    /// Adds and binds `AcirVar`s for each numeric block parameter or block parameter array
    /// element. At the same time `ArrayId`s are bound for any references within the params.
    fn convert_ssa_block_params(
        &mut self,
        params: &[ValueId],
        dfg: &DataFlowGraph,
        param_arrays_info: &[(usize, NumericType)],
    ) {
        let mut param_arrays_info_iter = param_arrays_info.iter();
        for param_id in params {
            let value = &dfg[*param_id];
            let param_type = match value {
                Value::Param { typ, .. } => typ,
                _ => unreachable!("ICE: Only Param type values should appear in block parameters"),
            };
            match param_type {
                Type::Numeric(numeric_type) => {
                    let acir_var = self.add_numeric_input_var(numeric_type);
                    self.ssa_value_to_acir_var.insert(*param_id, acir_var);
                }
                Type::Reference => {
                    let (array_length, numeric_type) = param_arrays_info_iter
                        .next()
                        .expect("ICE: fewer arrays in abi than in block params");
                    let array_id = self.acir_context.allocate_array(*array_length);
                    self.ssa_value_to_array_address.insert(*param_id, (array_id, 0));
                    for index in 0..*array_length {
                        let acir_var = self.add_numeric_input_var(numeric_type);
                        self.acir_context
                            .array_store(array_id, index, acir_var)
                            .expect("invalid array store");
                    }
                }
                _ => {
                    unreachable!(
                        "ICE: Params to the program should only contains numerics and arrays"
                    )
                }
            }
        }
        assert_eq!(
            param_arrays_info_iter.next(),
            None,
            "ICE: more arrays in abi than in block params"
        );
    }

    /// Creates an `AcirVar` corresponding to a parameter witness to appears in the abi. A range
    /// constraint is added if the numeric type requires it.
    ///
    /// This function is used not only for adding numeric block parameters, but also for adding
    /// any array elements that belong to reference type block parameters.
    fn add_numeric_input_var(&mut self, numeric_type: &NumericType) -> AcirVar {
        let acir_var = self.acir_context.add_variable();
        if matches!(numeric_type, NumericType::Signed { .. } | NumericType::Unsigned { .. }) {
            self.acir_context
                .numeric_cast_var(acir_var, numeric_type)
                .expect("invalid range constraint was applied {numeric_type}");
        }
        acir_var
    }

    /// Converts an SSA instruction into its ACIR representation
    fn convert_ssa_instruction(
        &mut self,
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
        ssa: &Ssa,
        brillig: &Brillig,
        allow_log_ops: bool,
    ) {
        let instruction = &dfg[instruction_id];

        let (results_id, results_vars) = match instruction {
            Instruction::Binary(binary) => {
                let result_ids = dfg.instruction_results(instruction_id);
                if Self::value_is_array_address(result_ids[0], dfg) {
                    self.track_array_address(result_ids[0], binary, dfg);
                    (Vec::new(), Vec::new())
                } else {
                    let result_acir_var = self.convert_ssa_binary(binary, dfg);
                    (vec![result_ids[0]], vec![result_acir_var])
                }
            }
            Instruction::Constrain(value_id) => {
                let constrain_condition = self.convert_ssa_value(*value_id, dfg);
                self.acir_context.assert_eq_one(constrain_condition);
                (Vec::new(), Vec::new())
            }
            Instruction::Cast(value_id, typ) => {
                let result_acir_var = self.convert_ssa_cast(value_id, typ, dfg);
                let result_ids = dfg.instruction_results(instruction_id);
                (vec![result_ids[0]], vec![result_acir_var])
            }
            Instruction::Call { func, arguments } => {
                let result_ids = dfg.instruction_results(instruction_id);
                match &dfg[*func] {
                    Value::Function(id) => {
                        let func = &ssa.functions[id];
                        match func.runtime() {
                            RuntimeType::Acir => unimplemented!(
                                "expected an intrinsic/brillig call, but found {func:?}. All ACIR methods should be inlined"
                            ),
                            RuntimeType::Brillig => {
                                // Generate the brillig code of the function
                                let code = BrilligArtefact::default().link(&brillig[*id]);
                                self.acir_context.brillig(code);
                                (result_ids.to_vec(), Vec::new())
                            }
                        }
                    }
                    Value::Intrinsic(intrinsic) => {
                        let outputs = self.convert_ssa_intrinsic_call(
                            *intrinsic,
                            arguments,
                            dfg,
                            allow_log_ops,
                        );
                        let result_ids = dfg.instruction_results(instruction_id);
                        (result_ids.to_vec(), outputs)
                    }
                    _ => unreachable!("expected calling a function"),
                }
            }
            Instruction::Not(value_id) => {
                let boolean_var = self.convert_ssa_value(*value_id, dfg);
                let result_acir_var = self.acir_context.not_var(boolean_var);

                let result_ids = dfg.instruction_results(instruction_id);
                (vec![result_ids[0]], vec![result_acir_var])
            }
            Instruction::Allocate { size } => {
                let array_id = self.acir_context.allocate_array(*size as usize);
                let result_ids = dfg.instruction_results(instruction_id);
                self.ssa_value_to_array_address.insert(result_ids[0], (array_id, 0));
                (Vec::new(), Vec::new())
            }
            Instruction::Store { address, value } => {
                self.convert_ssa_store(address, value, dfg);
                (Vec::new(), Vec::new())
            }
            Instruction::Load { address } => {
                let result_acir_var = self.convert_ssa_load(address);
                let result_ids = dfg.instruction_results(instruction_id);
                (vec![result_ids[0]], vec![result_acir_var])
            }
            _ => todo!("{instruction:?}"),
        };

        // Map the results of the instructions to Acir variables
        for (result_id, result_var) in results_id.into_iter().zip(results_vars) {
            self.ssa_value_to_acir_var.insert(result_id, result_var);
        }
    }

    /// Converts an SSA terminator's return values into their ACIR representations
    fn convert_ssa_return(&mut self, terminator: &TerminatorInstruction, dfg: &DataFlowGraph) {
        let return_values = match terminator {
            TerminatorInstruction::Return { return_values } => return_values,
            _ => unreachable!("ICE: Program must have a singular return"),
        };

        // Check if the program returns the `Unit/None` type.
        // This type signifies that the program returns nothing.
        let is_return_unit_type =
            return_values.len() == 1 && dfg.type_of_value(return_values[0]) == Type::Unit;
        if is_return_unit_type {
            return;
        }

        for value_id in return_values {
            let acir_var = self.convert_ssa_value(*value_id, dfg);
            self.acir_context.return_var(acir_var);
        }
    }

    /// Gets the cached `AcirVar` that was converted from the corresponding `ValueId`. If it does
    /// not already exist in the cache, a conversion is attempted and cached for simple values
    /// that require no further context such as numeric types - values requiring more context
    /// should have already been cached elsewhere.
    ///
    /// Conversion is assumed to have already been performed for instruction results and block
    /// parameters. This is because block parameters are converted before anything else, and
    /// because instructions results are converted when the corresponding instruction is
    /// encountered. (An instruction result cannot be referenced before the instruction occurs.)
    ///
    /// It is not safe to call this function on value ids that represent addresses. Instructions
    /// involving such values are evaluated via a separate path and stored in
    /// `ssa_value_to_array_address` instead.
    fn convert_ssa_value(&mut self, value_id: ValueId, dfg: &DataFlowGraph) -> AcirVar {
        let value = &dfg[value_id];
        if let Some(acir_var) = self.ssa_value_to_acir_var.get(&value_id) {
            return *acir_var;
        }
        let acir_var = match value {
            Value::NumericConstant { constant, .. } => self.acir_context.add_constant(*constant),
            Value::Intrinsic(..) => todo!(),
            Value::Function(..) => unreachable!("ICE: All functions should have been inlined"),
            Value::Instruction { .. } | Value::Param { .. } => {
                unreachable!("ICE: Should have been in cache {value:?}")
            }
        };
        self.ssa_value_to_acir_var.insert(value_id, acir_var);
        acir_var
    }

    /// Processes a binary operation and converts the result into an `AcirVar`
    fn convert_ssa_binary(&mut self, binary: &Binary, dfg: &DataFlowGraph) -> AcirVar {
        let lhs = self.convert_ssa_value(binary.lhs, dfg);
        let rhs = self.convert_ssa_value(binary.rhs, dfg);
        match binary.operator {
            BinaryOp::Add => self.acir_context.add_var(lhs, rhs),
            BinaryOp::Sub => self.acir_context.sub_var(lhs, rhs),
            BinaryOp::Mul => self.acir_context.mul_var(lhs, rhs),
            BinaryOp::Div => self.acir_context.div_var(lhs, rhs),
            // Note: that this produces unnecessary constraints when
            // this Eq instruction is being used for a constrain statement
            BinaryOp::Eq => self.acir_context.eq_var(lhs, rhs),
            BinaryOp::Lt => self
                .acir_context
                .less_than_var(lhs, rhs)
                .expect("add Result types to all methods so errors bubble up"),
            BinaryOp::Shl => self.acir_context.shift_left_var(lhs, rhs),
            BinaryOp::Shr => self.acir_context.shift_right_var(lhs, rhs),
            BinaryOp::Xor => self
                .acir_context
                .xor_var(lhs, rhs)
                .expect("add Result types to all methods so errors bubble up"),
            BinaryOp::And => self
                .acir_context
                .and_var(lhs, rhs)
                .expect("add Result types to all methods so errors bubble up"),
            BinaryOp::Or => self
                .acir_context
                .or_var(lhs, rhs)
                .expect("add Result types to all methods so errors bubble up"),
            _ => todo!(),
        }
    }
    /// Returns an `AcirVar` that is constrained to be
    fn convert_ssa_cast(&mut self, value_id: &ValueId, typ: &Type, dfg: &DataFlowGraph) -> AcirVar {
        let variable = self.convert_ssa_value(*value_id, dfg);

        match typ {
            Type::Numeric(numeric_type) => self
                .acir_context
                .numeric_cast_var(variable, numeric_type)
                .expect("invalid range constraint was applied {numeric_type}"),
            _ => unimplemented!("The cast operation is only valid for integers."),
        }
    }

    /// Returns a vector of `AcirVar`s constrained to be result of the function call.
    ///
    /// The function being called is required to be intrinsic.
    fn convert_ssa_intrinsic_call(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
        allow_log_ops: bool,
    ) -> Vec<AcirVar> {
        let inputs = self
            .flatten_arguments(arguments, dfg)
            .expect("add Result types to all methods so errors bubble up");
        match intrinsic {
            Intrinsic::BlackBox(black_box) => self
                .acir_context
                .black_box_function(black_box, inputs)
                .expect("add Result types to all methods so errors bubble up"),
            Intrinsic::Println => {
                if allow_log_ops {
                    self.acir_context
                        .print(inputs)
                        .expect("add Result types to all methods so errors bubble up");
                }
                Vec::new()
            }
            _ => todo!("expected a black box function"),
        }
    }

    /// Maps an ssa value list, for which some values may be references to arrays, by inlining
    /// the `AcirVar`s corresponding to the contents of each array into the list of `AcirVar`s
    /// that correspond to other values.
    fn flatten_arguments(
        &mut self,
        arguments: &[ValueId],
        dfg: &DataFlowGraph,
    ) -> Result<Vec<AcirVar>, AcirGenError> {
        let mut acir_vars = Vec::new();
        for value_id in arguments {
            if Self::value_is_array_address(*value_id, dfg) {
                let (array_id, index) = self
                    .ssa_value_to_array_address
                    .get(value_id)
                    .expect("ICE: Call argument of undeclared array");
                assert_eq!(index, &0, "ICE: Call arguments only accept arrays in their entirety");
                let elements = self.acir_context.array_load_all(*array_id)?;
                acir_vars.extend(elements);
            } else {
                acir_vars.push(self.convert_ssa_value(*value_id, dfg));
            }
        }
        Ok(acir_vars)
    }

    /// Stores the `AcirVar` corresponding to `value` at the `ArrayId` and index corresponding to
    /// `address`.
    fn convert_ssa_store(&mut self, address: &ValueId, value: &ValueId, dfg: &DataFlowGraph) {
        let element_var = self.convert_ssa_value(*value, dfg);
        let (array_id, index) =
            self.ssa_value_to_array_address.get(address).expect("ICE: Load from undeclared array");
        self.acir_context.array_store(*array_id, *index, element_var).expect("invalid array load");
    }

    /// Returns the `AcirVar` that was previously stored at the given address.
    fn convert_ssa_load(&mut self, address: &ValueId) -> AcirVar {
        let (array_id, index) =
            self.ssa_value_to_array_address.get(address).expect("ICE: Load from undeclared array");
        self.acir_context.array_load(*array_id, *index).expect("invalid array load")
    }

    /// Returns true if the value has been declared as an array address
    fn value_is_array_address(value_id: ValueId, dfg: &DataFlowGraph) -> bool {
        dfg.type_of_value(value_id) == Type::Reference
    }

    /// Takes a binary instruction describing array address arithmetic and stores the result.
    fn track_array_address(&mut self, value_id: ValueId, binary: &Binary, dfg: &DataFlowGraph) {
        if binary.operator != BinaryOp::Add {
            unreachable!("ICE: Array address arithmetic only supports Add");
        }
        let lhs_address = self.ssa_value_to_array_address.get(&binary.lhs);
        let rhs_address = self.ssa_value_to_array_address.get(&binary.rhs);
        let ((array_id, offset), other_value_id) = match (lhs_address, rhs_address) {
            (Some(address), None) => (address, binary.rhs),
            (None, Some(address)) => (address, binary.lhs),
            (Some(_), Some(_)) => unreachable!("ICE: Addresses cannot be added"),
            (None, None) => unreachable!("ICE: One operand must be an address"),
        };
        let other_value = &dfg[other_value_id];
        let new_offset = match other_value {
            Value::NumericConstant { constant, .. } => {
                let further_offset =
                    constant.try_to_u64().expect("ICE: array arithmetic doesn't fit in u64")
                        as usize;
                offset + further_offset
            }
            _ => unreachable!("Invalid array address arithmetic operand"),
        };
        self.ssa_value_to_array_address.insert(value_id, (*array_id, new_offset));
    }
}
