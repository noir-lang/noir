//! The loop invariant code motion pass moves code from inside a loop to before the loop
//! if that code will always have the same result on every iteration of the loop.
//!
//! To identify a loop invariant, check whether all of an instruction's values are:
//! - Outside of the loop
//! - Constant
//! - Already marked as loop invariants
//!
//! We also check that we are not hoisting instructions with side effects.
use acvm::{acir::AcirField, FieldElement};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{binary::eval_constant_binary_op, BinaryOp, Instruction, InstructionId},
        types::Type,
        value::ValueId,
    },
    Ssa,
};

use super::unrolling::{Loop, Loops};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn loop_invariant_code_motion(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.loop_invariant_code_motion();
        }

        self
    }
}

impl Function {
    pub(super) fn loop_invariant_code_motion(&mut self) {
        Loops::find_all(self).hoist_loop_invariants(self);
    }
}

impl Loops {
    fn hoist_loop_invariants(mut self, function: &mut Function) {
        let mut context = LoopInvariantContext::new(function);

        // The loops should be sorted by the number of blocks.
        // We want to access outer nested loops first, which we do by popping
        // from the top of the list.
        while let Some(loop_) = self.yet_to_unroll.pop() {
            let Ok(pre_header) = loop_.get_pre_header(context.inserter.function, &self.cfg) else {
                // If the loop does not have a preheader we skip hoisting loop invariants for this loop
                continue;
            };

            context.hoist_loop_invariants(&loop_, pre_header);
        }

        context.map_dependent_instructions();
        context.inserter.map_data_bus_in_place();
    }
}

impl Loop {
    /// Find the value that controls whether to perform a loop iteration.
    /// This is going to be the block parameter of the loop header.
    ///
    /// Consider the following example of a `for i in 0..4` loop:
    /// ```text
    /// brillig(inline) fn main f0 {
    ///   b0(v0: u32):
    ///     ...
    ///     jmp b1(u32 0)
    ///   b1(v1: u32):                  // Loop header
    ///     v5 = lt v1, u32 4           // Upper bound
    ///     jmpif v5 then: b3, else: b2
    /// ```
    /// In the example above, `v1` is the induction variable
    fn get_induction_variable(&self, function: &Function) -> ValueId {
        function.dfg.block_parameters(self.header)[0]
    }
}

struct LoopInvariantContext<'f> {
    inserter: FunctionInserter<'f>,
    defined_in_loop: HashSet<ValueId>,
    loop_invariants: HashSet<ValueId>,
    // Maps induction variable -> fixed upper loop bound
    outer_induction_variables: HashMap<ValueId, FieldElement>,
}

impl<'f> LoopInvariantContext<'f> {
    fn new(function: &'f mut Function) -> Self {
        Self {
            inserter: FunctionInserter::new(function),
            defined_in_loop: HashSet::default(),
            loop_invariants: HashSet::default(),
            outer_induction_variables: HashMap::default(),
        }
    }

    fn hoist_loop_invariants(&mut self, loop_: &Loop, pre_header: BasicBlockId) {
        self.set_values_defined_in_loop(loop_);

        for block in loop_.blocks.iter() {
            for instruction_id in self.inserter.function.dfg[*block].take_instructions() {
                let hoist_invariant = self.can_hoist_invariant(instruction_id);

                if hoist_invariant {
                    self.inserter.push_instruction(instruction_id, pre_header);

                    // If we are hoisting a MakeArray instruction,
                    // we need to issue an extra inc_rc in case they are mutated afterward.
                    if self.inserter.function.runtime().is_brillig()
                        && matches!(
                            self.inserter.function.dfg[instruction_id],
                            Instruction::MakeArray { .. }
                        )
                    {
                        let result =
                            self.inserter.function.dfg.instruction_results(instruction_id)[0];
                        let inc_rc = Instruction::IncrementRc { value: result };
                        let call_stack = self
                            .inserter
                            .function
                            .dfg
                            .get_instruction_call_stack_id(instruction_id);
                        self.inserter
                            .function
                            .dfg
                            .insert_instruction_and_results(inc_rc, *block, None, call_stack);
                    }
                } else {
                    self.inserter.push_instruction(instruction_id, *block);
                }

                self.extend_values_defined_in_loop_and_invariants(instruction_id, hoist_invariant);
            }
        }

        // Keep track of a loop induction variable and respective upper bound.
        // This will be used by later loops to determine whether they have operations
        // reliant upon the maximum induction variable.
        let upper_bound = loop_.get_const_upper_bound(self.inserter.function);
        if let Some(upper_bound) = upper_bound {
            let induction_variable = loop_.get_induction_variable(self.inserter.function);
            let induction_variable = self.inserter.resolve(induction_variable);
            self.outer_induction_variables.insert(induction_variable, upper_bound);
        }
    }

    /// Gather the variables declared within the loop
    fn set_values_defined_in_loop(&mut self, loop_: &Loop) {
        // Clear any values that may be defined in previous loops, as the context is per function.
        self.defined_in_loop.clear();
        // These are safe to keep per function, but we want to be clear that these values
        // are used per loop.
        self.loop_invariants.clear();

        for block in loop_.blocks.iter() {
            let params = self.inserter.function.dfg.block_parameters(*block);
            self.defined_in_loop.extend(params);
            for instruction_id in self.inserter.function.dfg[*block].instructions() {
                let results = self.inserter.function.dfg.instruction_results(*instruction_id);
                self.defined_in_loop.extend(results);
            }
        }
    }

    /// Update any values defined in the loop and loop invariants after a
    /// analyzing and re-inserting a loop's instruction.
    fn extend_values_defined_in_loop_and_invariants(
        &mut self,
        instruction_id: InstructionId,
        hoist_invariant: bool,
    ) {
        let results = self.inserter.function.dfg.instruction_results(instruction_id).to_vec();
        // We will have new IDs after pushing instructions.
        // We should mark the resolved result IDs as also being defined within the loop.
        let results =
            results.into_iter().map(|value| self.inserter.resolve(value)).collect::<Vec<_>>();
        self.defined_in_loop.extend(results.iter());

        // We also want the update result IDs when we are marking loop invariants as we may not
        // be going through the blocks of the loop in execution order
        if hoist_invariant {
            // Track already found loop invariants
            self.loop_invariants.extend(results.iter());
        }
    }

    fn can_hoist_invariant(&mut self, instruction_id: InstructionId) -> bool {
        let mut is_loop_invariant = true;
        // The list of blocks for a nested loop contain any inner loops as well.
        // We may have already re-inserted new instructions if two loops share blocks
        // so we need to map all the values in the instruction which we want to check.
        let (instruction, _) = self.inserter.map_instruction(instruction_id);
        instruction.for_each_value(|value| {
            // If an instruction value is defined in the loop and not already a loop invariant
            // the instruction results are not loop invariants.
            //
            // We are implicitly checking whether the values are constant as well.
            // The set of values defined in the loop only contains instruction results and block parameters
            // which cannot be constants.
            is_loop_invariant &=
                !self.defined_in_loop.contains(&value) || self.loop_invariants.contains(&value);
        });

        let can_be_deduplicated = instruction.can_be_deduplicated(self.inserter.function, false)
            || matches!(instruction, Instruction::MakeArray { .. })
            || matches!(instruction, Instruction::Binary(_))
            || self.can_be_deduplicated_from_upper_bound(&instruction);

        is_loop_invariant && can_be_deduplicated
    }

    /// Certain instructions can take advantage of that our induction variable has a fixed maximum.
    ///
    /// For example, an array access can usually only be safely deduplicated when we have a constant
    /// index that is below the length of the array.
    /// Checking an array get where the index is the loop's induction variable on its own
    /// would determine that the instruction is not safe for hoisting.
    /// However, if we know that the induction variable's upper bound will always be in bounds of the array
    /// we can safely hoist the array access.
    fn can_be_deduplicated_from_upper_bound(&self, instruction: &Instruction) -> bool {
        match instruction {
            Instruction::ArrayGet { array, index } => {
                let array_typ = self.inserter.function.dfg.type_of_value(*array);
                let upper_bound = self.outer_induction_variables.get(index);
                if let (Type::Array(_, len), Some(upper_bound)) = (array_typ, upper_bound) {
                    upper_bound.to_u128() <= len.into()
                } else {
                    false
                }
            }
            Instruction::Binary(binary) => {
                if !matches!(binary.operator, BinaryOp::Add { .. } | BinaryOp::Mul { .. }) {
                    return false;
                }

                let operand_type =
                    self.inserter.function.dfg.type_of_value(binary.lhs).unwrap_numeric();

                let lhs_const =
                    self.inserter.function.dfg.get_numeric_constant_with_type(binary.lhs);
                let rhs_const =
                    self.inserter.function.dfg.get_numeric_constant_with_type(binary.rhs);
                let (lhs, rhs) = match (
                    lhs_const,
                    rhs_const,
                    self.outer_induction_variables.get(&binary.lhs),
                    self.outer_induction_variables.get(&binary.rhs),
                ) {
                    (Some((lhs, _)), None, None, Some(upper_bound)) => (lhs, *upper_bound),
                    (None, Some((rhs, _)), Some(upper_bound), None) => (*upper_bound, rhs),
                    _ => return false,
                };

                eval_constant_binary_op(lhs, rhs, binary.operator, operand_type).is_some()
            }
            _ => false,
        }
    }

    /// Loop invariant hoisting only operates over loop instructions.
    /// The `FunctionInserter` is used for mapping old values to new values after
    /// re-inserting loop invariant instructions.
    /// However, there may be instructions which are not within loops that are
    /// still reliant upon the instruction results altered during the pass.
    /// This method re-inserts all instructions so that all instructions have
    /// correct new value IDs based upon the `FunctionInserter` internal map.
    /// Leaving out this mapping could lead to instructions with values that do not exist.
    fn map_dependent_instructions(&mut self) {
        let blocks = self.inserter.function.reachable_blocks();
        for block in blocks {
            for instruction_id in self.inserter.function.dfg[block].take_instructions() {
                self.inserter.push_instruction(instruction_id, block);
            }
            self.inserter.map_terminator_in_place(block);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ssa::opt::assert_normalized_ssa_equals;
    use crate::ssa::Ssa;

    #[test]
    fn simple_loop_invariant_code_motion() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
              jmp b1(i32 0)
          b1(v2: i32):
              v5 = lt v2, i32 4
              jmpif v5 then: b3, else: b2
          b2():
              return
          b3():
              v6 = mul v0, v1
              constrain v6 == i32 6
              v8 = add v2, i32 1
              jmp b1(v8)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        // `v6 = mul v0, v1` in b3 should now be `v3 = mul v0, v1` in b0
        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v3 = mul v0, v1
            jmp b1(i32 0)
          b1(v2: i32):
            v6 = lt v2, i32 4
            jmpif v6 then: b3, else: b2
          b2():
            return
          b3():
            constrain v3 == i32 6
            v9 = add v2, i32 1
            jmp b1(v9)
        }
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn nested_loop_invariant_code_motion() {
        // Check that a loop invariant in the inner loop of a nested loop
        // is hoisted to the parent loop's pre-header block.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            jmp b1(i32 0)
          b1(v2: i32):
            v6 = lt v2, i32 4
            jmpif v6 then: b3, else: b2
          b2():
            return
          b3():
            jmp b4(i32 0)
          b4(v3: i32):
            v7 = lt v3, i32 4
            jmpif v7 then: b6, else: b5
          b5():
            v9 = add v2, i32 1
            jmp b1(v9)
          b6():
            v10 = mul v0, v1
            constrain v10 == i32 6
            v12 = add v3, i32 1
            jmp b4(v12)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        // `v10 = mul v0, v1` in b6 should now be `v4 = mul v0, v1` in b0
        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v4 = mul v0, v1
            jmp b1(i32 0)
          b1(v2: i32):
            v7 = lt v2, i32 4
            jmpif v7 then: b3, else: b2
          b2():
            return
          b3():
            jmp b4(i32 0)
          b4(v3: i32):
            v8 = lt v3, i32 4
            jmpif v8 then: b6, else: b5
          b5():
            v10 = add v2, i32 1
            jmp b1(v10)
          b6():
            constrain v4 == i32 6
            v12 = add v3, i32 1
            jmp b4(v12)
        }
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn hoist_invariant_with_invariant_as_argument() {
        // Check that an instruction which has arguments defined in the loop
        // but which are already marked loop invariants is still hoisted to the preheader.
        //
        // For example, in b3 we have the following instructions:
        // ```text
        // v6 = mul v0, v1
        // v7 = mul v6, v0
        // ```
        // `v6` should be marked a loop invariants as `v0` and `v1` are both declared outside of the loop.
        // As we will be hoisting `v6 = mul v0, v1` to the loop preheader we know that we can also
        // hoist `v7 = mul v6, v0`.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            jmp b1(i32 0)
          b1(v2: i32):
            v5 = lt v2, i32 4
            jmpif v5 then: b3, else: b2
          b2():
            return
          b3():
            v6 = mul v0, v1
            v7 = mul v6, v0
            v8 = eq v7, i32 12
            constrain v7 == i32 12
            v9 = add v2, i32 1
            jmp b1(v9)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v3 = mul v0, v1
            v4 = mul v3, v0
            v6 = eq v4, i32 12
            jmp b1(i32 0)
          b1(v2: i32):
            v9 = lt v2, i32 4
            jmpif v9 then: b3, else: b2
          b2():
            return
          b3():
            constrain v4 == i32 12
            v11 = add v2, i32 1
            jmp b1(v11)
        }
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn do_not_hoist_instructions_with_side_effects() {
        // In `v12 = load v5` in `b3`, `v5` is defined outside the loop.
        // However, as the instruction has side effects, we want to make sure
        // we do not hoist the instruction to the loop preheader.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v4 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 5]
            inc_rc v4
            v5 = allocate -> &mut [u32; 5]
            store v4 at v5
            jmp b1(u32 0)
          b1(v2: u32):
            v7 = lt v2, u32 4
            jmpif v7 then: b3, else: b2
          b2():
            v8 = load v5 -> [u32; 5]
            v10 = array_get v8, index u32 2 -> u32
            constrain v10 == u32 3
            return
          b3():
            v12 = load v5 -> [u32; 5]
            v13 = array_set v12, index v0, value v1
            store v13 at v5
            v15 = add v2, u32 1
            jmp b1(v15)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 4); // The final return is not counted

        let ssa = ssa.loop_invariant_code_motion();
        // The code should be unchanged
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn hoist_array_gets_using_induction_variable_with_const_bound() {
        // SSA for the following program:
        //
        // fn triple_loop(x: u32) {
        //   let arr = [2; 5];
        //   for i in 0..4 {
        //       for j in 0..4 {
        //           for _ in 0..4 {
        //               assert_eq(arr[i], x);
        //               assert_eq(arr[j], x);
        //           }
        //       }
        //   }
        // }
        //
        // `arr[i]` and `arr[j]` are safe to hoist as we know the maximum possible index
        // to be used for both array accesses.
        // We want to make sure `arr[i]` is hoisted to the outermost loop body and that
        // `arr[j]` is hoisted to the second outermost loop body.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v6 = make_array [u32 2, u32 2, u32 2, u32 2, u32 2] : [u32; 5]
            inc_rc v6
            jmp b1(u32 0)
          b1(v2: u32):
            v9 = lt v2, u32 4
            jmpif v9 then: b3, else: b2
          b2():
            return
          b3():
            jmp b4(u32 0)
          b4(v3: u32):
            v10 = lt v3, u32 4
            jmpif v10 then: b6, else: b5
          b5():
            v12 = add v2, u32 1
            jmp b1(v12)
          b6():
            jmp b7(u32 0)
          b7(v4: u32):
            v13 = lt v4, u32 4
            jmpif v13 then: b9, else: b8
          b8():
            v14 = add v3, u32 1
            jmp b4(v14)
          b9():
            v15 = array_get v6, index v2 -> u32
            v16 = eq v15, v0
            constrain v15 == v0
            v17 = array_get v6, index v3 -> u32
            v18 = eq v17, v0
            constrain v17 == v0
            v19 = add v4, u32 1
            jmp b7(v19)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v6 = make_array [u32 2, u32 2, u32 2, u32 2, u32 2] : [u32; 5]
            inc_rc v6
            jmp b1(u32 0)
          b1(v2: u32):
            v9 = lt v2, u32 4
            jmpif v9 then: b3, else: b2
          b2():
            return
          b3():
            v10 = array_get v6, index v2 -> u32
            v11 = eq v10, v0
            jmp b4(u32 0)
          b4(v3: u32):
            v12 = lt v3, u32 4
            jmpif v12 then: b6, else: b5
          b5():
            v14 = add v2, u32 1
            jmp b1(v14)
          b6():
            v15 = array_get v6, index v3 -> u32
            v16 = eq v15, v0
            jmp b7(u32 0)
          b7(v4: u32):
            v17 = lt v4, u32 4
            jmpif v17 then: b9, else: b8
          b8():
            v18 = add v3, u32 1
            jmp b4(v18)
          b9():
            constrain v10 == v0
            constrain v15 == v0
            v19 = add v4, u32 1
            jmp b7(v19)
        }
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn insert_inc_rc_when_moving_make_array() {
        // SSA for the following program:
        //
        // unconstrained fn main(x: u32, y: u32) {
        //   let mut a1 = [1, 2, 3, 4, 5];
        //   a1[x] = 64;
        //   for i in 0 .. 5 {
        //       let mut a2 = [1, 2, 3, 4, 5];
        //       a2[y + i] = 128;
        //       foo(a2);
        //   }
        //   foo(a1);
        // }
        //
        // We want to make sure move a loop invariant make_array instruction,
        // to account for whether that array has been marked as mutable.
        // To do so, we increment the reference counter on the array we are moving.
        // In the SSA below, we want to move `v42` out of the loop.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v8 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            v9 = allocate -> &mut [Field; 5]
            v11 = array_set v8, index v0, value Field 64
            v13 = add v0, u32 1
            store v11 at v9
            jmp b1(u32 0)
          b1(v2: u32):
            v16 = lt v2, u32 5
            jmpif v16 then: b3, else: b2
          b2():
            v17 = load v9 -> [Field; 5]
            call f1(v17)
            return
          b3():
            v19 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            v20 = allocate -> &mut [Field; 5]
            v21 = add v1, v2
            v23 = array_set v19, index v21, value Field 128
            call f1(v23)
            v25 = add v2, u32 1
            jmp b1(v25)
        }
        brillig(inline) fn foo f1 {
          b0(v0: [Field; 5]):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        // We expect the `make_array` at the top of `b3` to be replaced with an `inc_rc`
        // of the newly hoisted `make_array` at the end of `b0`.
        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v8 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            v9 = allocate -> &mut [Field; 5]
            v11 = array_set v8, index v0, value Field 64
            v13 = add v0, u32 1
            store v11 at v9
            v14 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            jmp b1(u32 0)
          b1(v2: u32):
            v17 = lt v2, u32 5
            jmpif v17 then: b3, else: b2
          b2():
            v18 = load v9 -> [Field; 5]
            call f1(v18)
            return
          b3():
            inc_rc v14
            v20 = allocate -> &mut [Field; 5]
            v21 = add v1, v2
            v23 = array_set v14, index v21, value Field 128
            call f1(v23)
            v25 = add v2, u32 1
            jmp b1(v25)
        }
        brillig(inline) fn foo f1 {
          b0(v0: [Field; 5]):
            return
        }
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
    }
}
