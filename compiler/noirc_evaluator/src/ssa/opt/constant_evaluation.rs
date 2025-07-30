//! Evaluates calls to Brillig functions at compile time when all arguments are constant.
//!
//! This acts as a partial evaluator for Brillig entry points called from ACIR functions.
//! For any Brillig function call where all arguments are constant, we interpret the
//! function using the [SSA interpreter][crate::ssa::interpreter] and replace the call with the resulting constants.
//!
//! This improves constant folding across function boundaries, particularly for small utility
//! Brillig functions that can be inlined
//!
//! Note: Only calls to Brillig functions from ACIR functions (Brillig entry points) are evaluated here.
//! Most other constant evaluation occurs during [instruction simplification][crate::ssa::ir::dfg::simplify]
//! and [constant folding][crate::ssa::opt::constant_folding].
//! However, this constant evaluation is unable to remove calls to Brillig entry points with constant arguments
//! as entry points are explicitly prevented from being inlined.  
use fxhash::FxHashMap as HashMap;

use crate::ssa::{
    interpreter::{InterpreterOptions, value::Value},
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::Function,
        instruction::{Instruction, InstructionId},
        types::Type,
        value::{Value as IrValue, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`constant_evaluation`][self] module for more information.
    pub(crate) fn constant_evaluation(mut self) -> Ssa {
        // Gather constant evaluation results per function
        let mut constant_evaluations = self
            .functions
            .iter()
            .map(|(id, function)| (*id, function.constant_evaluation(&self)))
            .collect::<HashMap<_, _>>();

        // Apply constant folding for each function based on the interpreter results
        for function in self.functions.values_mut() {
            let Some(constant_evaluations) = constant_evaluations.remove(&function.id()) else {
                continue;
            };

            function.fold_constant_evaluation(constant_evaluations);
        }

        self
    }
}

impl Function {
    /// For a given function, identifies all Brillig function calls with constant arguments,
    /// interprets them using the [SSA interpreter][crate::ssa::interpreter], and records the result.
    ///
    /// This does not mutate the IR. The interpreter borrows the entire Ssa object so we split up
    /// the actual interpreter execution from the mutation of the IR.
    fn constant_evaluation(&self, ssa: &Ssa) -> HashMap<InstructionId, Vec<Value>> {
        let mut instr_to_const_result = HashMap::default();

        // Only ACIR functions can be evaluated in this pass.
        // Brillig non-entry points themselves are not subject to constant evaluation.
        if self.runtime().is_brillig() {
            return instr_to_const_result;
        }

        for block_id in self.reachable_blocks() {
            for instruction_id in self.dfg[block_id].instructions() {
                let instruction = &self.dfg[*instruction_id];
                let Instruction::Call { func: func_id, arguments } = instruction else {
                    continue;
                };

                let func_value = &self.dfg[*func_id];
                let IrValue::Function(func_id) = func_value else {
                    continue;
                };

                let Some(func) = ssa.functions.get(func_id) else {
                    unreachable!("ICE: Function {func_id} does not exist, yet it is being called");
                };

                // Skip calls to non-Brillig functions
                if !func.runtime().is_brillig()
                    // Ensure all arguments to the call are constant
                    || !arguments.iter().all(|argument| self.dfg.is_constant(*argument))
                {
                    continue;
                }

                let interpreter_args = arguments
                    .iter()
                    .map(|arg| Self::const_ir_value_to_interpreter_value(*arg, &self.dfg))
                    .collect();
                match ssa.interpret_function(
                    func.id(),
                    interpreter_args,
                    InterpreterOptions { no_foreign_calls: true, ..Default::default() },
                    std::io::empty(),
                ) {
                    Ok(values) => {
                        instr_to_const_result.insert(*instruction_id, values);
                    }
                    Err(_) => {
                        // Failed to interpret (e.g., unsupported op, failed constrain, etc.)
                        continue;
                    }
                }
            }
        }
        instr_to_const_result
    }

    /// Replaces Brillig call instructions with constant results, based on interpreter output.
    /// The interpreter output is expected to be computed by the caller of this method.
    fn fold_constant_evaluation(
        &mut self,
        mut instr_to_const_result: HashMap<InstructionId, Vec<Value>>,
    ) {
        self.simple_reachable_blocks_optimization(|context| {
            let Some(constant_results) = instr_to_const_result.remove(&context.instruction_id)
            else {
                return;
            };

            let results = context.dfg.instruction_results(context.instruction_id).to_vec();
            assert_eq!(
                results.len(),
                constant_results.len(),
                "ICE: The interpreter should return the same number of results as the SSA specifies"
            );

            for (old_result, constant_result) in results.iter().zip(constant_results) {
                let new_result = Self::interpreter_value_to_ir_value(
                    constant_result,
                    context.dfg,
                    context.block_id,
                );
                context.replace_value(*old_result, new_result);
            }

            context.remove_current_instruction();
        });
    }

    /// Converts a constant [SSA value][IrValue] into an [interpreter value][Value] for execution.
    fn const_ir_value_to_interpreter_value(value_id: ValueId, dfg: &DataFlowGraph) -> Value {
        let typ = dfg.type_of_value(value_id);
        match typ {
            Type::Numeric(numeric_type) => {
                let constant =
                    dfg.get_numeric_constant(value_id).expect("Should have a numeric constant");
                Value::from_constant(constant, numeric_type).expect("Should be a valid constant")
            }
            Type::Reference(_) => unreachable!("References cannot be constant values"),
            Type::Array(element_types, _) => {
                let (array_constant, _) =
                    dfg.get_array_constant(value_id).expect("Should have an array constant");
                let mut elements = Vec::new();
                for element in array_constant {
                    elements.push(Self::const_ir_value_to_interpreter_value(element, dfg));
                }
                Value::array(elements, element_types.to_vec())
            }
            Type::Slice(element_types) => {
                let (array_constant, _) =
                    dfg.get_array_constant(value_id).expect("Should have an array constant");
                let mut elements = Vec::new();
                for element in array_constant {
                    elements.push(Self::const_ir_value_to_interpreter_value(element, dfg));
                }
                Value::slice(elements, element_types)
            }
            Type::Function => unreachable!("Functions cannot be constant values"),
        }
    }

    /// Converts a constant [interpreter value][Value] back into an SSA constant.
    fn interpreter_value_to_ir_value(
        value: Value,
        dfg: &mut DataFlowGraph,
        block_id: BasicBlockId,
    ) -> ValueId {
        let typ = value.get_type();
        match typ {
            Type::Numeric(numeric_type) => {
                let constant = value.as_numeric().expect("Should be numeric").convert_to_field();
                dfg.make_constant(constant, numeric_type)
            }
            Type::Array(element_types, length) => {
                let array = match value {
                    Value::ArrayOrSlice(array) => array,
                    _ => unreachable!("Expected an ArrayOrSlice"),
                };

                let mut elements = im::Vector::new();
                for element in array.elements.unwrap_or_clone() {
                    elements.push_back(Self::interpreter_value_to_ir_value(element, dfg, block_id));
                }

                let instruction =
                    Instruction::MakeArray { elements, typ: Type::Array(element_types, length) };

                let instruction_id = dfg.make_instruction(instruction, None);
                dfg[block_id].instructions_mut().push(instruction_id);
                *dfg.instruction_results(instruction_id).first().unwrap()
            }
            Type::Slice(element_types) => {
                let array = match value {
                    Value::ArrayOrSlice(array) => array,
                    _ => unreachable!("Expected an ArrayOrSlice"),
                };

                let mut elements = im::Vector::new();
                for element in array.elements.unwrap_or_clone() {
                    elements.push_back(Self::interpreter_value_to_ir_value(element, dfg, block_id));
                }

                let instruction =
                    Instruction::MakeArray { elements, typ: Type::Slice(element_types) };

                let instruction_id = dfg.make_instruction(instruction, None);
                dfg[block_id].instructions_mut().push(instruction_id);
                *dfg.instruction_results(instruction_id).first().unwrap()
            }
            Type::Function | Type::Reference(_) => unreachable!("Cannot be a constant value"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn inlines_brillig_call_without_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1() -> Field
                return v0
            }

            brillig(inline) fn one f1 {
              b0():
                v0 = add Field 2, Field 3
                return v0
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.constant_evaluation();
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return Field 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_two_field_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, Field 3) -> Field
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: Field):
                v2 = add v0, v1
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.constant_evaluation();
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return Field 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_two_i32_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(i32 2, i32 3) -> i32
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: i32, v1: i32):
                v2 = unchecked_add v0, v1
                v3 = truncate v2 to 32 bits, max_bit_size: 33
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.constant_evaluation();
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return i32 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_array_return() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, Field 3, Field 4) -> [Field; 3]
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: Field, v2: Field):
                v3 = make_array [v0, v1, v2] : [Field; 3]
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.constant_evaluation();
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 2, Field 3, Field 4] : [Field; 3]
            return v3
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_composite_array_return() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, i32 3, Field 4, i32 5) -> [(Field, i32); 2]
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: i32, v2: Field, v3: i32):
                v4 = make_array [v0, v1, v2, v3] : [(Field, i32); 2]
                return v4
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.constant_evaluation();
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v4 = make_array [Field 2, i32 3, Field 4, i32 5] : [(Field, i32); 2]
            return v4
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_array_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = make_array [Field 2, Field 3] : [Field; 2]
                v1 = call f1(v0) -> Field
                return v1
            }

            brillig(inline) fn one f1 {
              b0(v0: [Field; 2]):
                inc_rc v0
                v2 = array_get v0, index u32 0 -> Field
                v4 = array_get v0, index u32 1 -> Field
                v5 = add v2, v4
                dec_rc v0
                return v5
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.constant_evaluation();
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field; 2]
            return Field 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_entry_point_globals() {
        let src = "
        g0 = Field 2

        acir(inline) fn main f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }

        brillig(inline) fn one f1 {
          b0():
            v1 = add g0, Field 3
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.constant_evaluation();
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 2

        acir(inline) fn main f0 {
          b0():
            return Field 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_non_entry_point_globals() {
        let src = "
        g0 = Field 2

        acir(inline) fn main f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }

        brillig(inline) fn entry_point f1 {
          b0():
            v1 = call f2() -> Field
            return v1
        }

        brillig(inline) fn one f2 {
          b0():
            v1 = add g0, Field 3
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.constant_evaluation();
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 2

        acir(inline) fn main f0 {
          b0():
            return Field 5
        }
        ");
    }
}
