//! The goal of the constant folding optimization pass is to propagate any constants forwards into
//! later [`Instruction`]s to maximize the impact of [compile-time simplifications][Instruction::simplify()].
//!
//! The pass works as follows:
//! - Re-insert each instruction in order to apply the instruction simplification performed
//!   by the [`DataFlowGraph`] automatically as new instructions are pushed.
//! - Check whether any input values have been constrained to be equal to a value of a simpler form
//!   by a [constrain instruction][Instruction::Constrain]. If so, replace the input value with the simpler form.
//! - Check whether the instruction [can_be_replaced][Instruction::can_be_replaced()]
//!   by duplicate instruction earlier in the same block.
//!
//! These operations are done in parallel so that they can each benefit from each other
//! without the need for multiple passes.
//!
//! Other passes perform a certain amount of constant folding automatically as they insert instructions
//! into the [`DataFlowGraph`] but this pass can become needed if [`DataFlowGraph::set_value`] or
//! [`DataFlowGraph::set_value_from_id`] are used on a value which enables instructions dependent on the value to
//! now be simplified.
//!
//! This is the only pass which removes duplicated pure [`Instruction`]s however and so is needed when
//! different blocks are merged, i.e. after the [`flatten_cfg`][super::flatten_cfg] pass.
use std::collections::HashSet;

use acvm::{acir::AcirField, FieldElement};
use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::{DataFlowGraph, InsertInstructionResult},
        function::Function,
        instruction::{Instruction, InstructionId},
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;

impl Ssa {
    /// Performs constant folding on each instruction.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn fold_constants(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.constant_fold(false);
        }
        self
    }

    /// Performs constant folding on each instruction.
    ///
    /// Also uses constraint information to inform more optimizations.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn fold_constants_using_constraints(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.constant_fold(true);
        }
        self
    }
}

impl Function {
    /// The structure of this pass is simple:
    /// Go through each block and re-insert all instructions.
    pub(crate) fn constant_fold(&mut self, use_constraint_info: bool) {
        let mut context = Context { use_constraint_info, ..Default::default() };
        context.block_queue.push(self.entry_block());

        while let Some(block) = context.block_queue.pop() {
            if context.visited_blocks.contains(&block) {
                continue;
            }

            context.visited_blocks.insert(block);
            context.fold_constants_in_block(self, block);
        }
    }
}

#[derive(Default)]
struct Context {
    use_constraint_info: bool,
    /// Maps pre-folded ValueIds to the new ValueIds obtained by re-inserting the instruction.
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: Vec<BasicBlockId>,
}

/// HashMap from (Instruction, side_effects_enabled_var) to the results of the instruction.
/// Stored as a two-level map to avoid cloning Instructions during the `.get` call.
type InstructionResultCache = HashMap<Instruction, HashMap<Option<ValueId>, Vec<ValueId>>>;

impl Context {
    fn fold_constants_in_block(&mut self, function: &mut Function, block: BasicBlockId) {
        let instructions = function.dfg[block].take_instructions();

        // Cache of instructions without any side-effects along with their outputs.
        let mut cached_instruction_results = HashMap::default();

        // Contains sets of values which are constrained to be equivalent to each other.
        //
        // The mapping's structure is `side_effects_enabled_var => (constrained_value => simplified_value)`.
        //
        // We partition the maps of constrained values according to the side-effects flag at the point
        // at which the values are constrained. This prevents constraints which are only sometimes enforced
        // being used to modify the rest of the program.
        let mut constraint_simplification_mappings: HashMap<ValueId, HashMap<ValueId, ValueId>> =
            HashMap::default();
        let mut side_effects_enabled_var =
            function.dfg.make_constant(FieldElement::one(), Type::bool());

        for instruction_id in instructions {
            self.fold_constants_into_instruction(
                &mut function.dfg,
                block,
                instruction_id,
                &mut cached_instruction_results,
                &mut constraint_simplification_mappings,
                &mut side_effects_enabled_var,
            );
        }
        self.block_queue.extend(function.dfg[block].successors());
    }

    fn fold_constants_into_instruction(
        &self,
        dfg: &mut DataFlowGraph,
        block: BasicBlockId,
        id: InstructionId,
        instruction_result_cache: &mut InstructionResultCache,
        constraint_simplification_mappings: &mut HashMap<ValueId, HashMap<ValueId, ValueId>>,
        side_effects_enabled_var: &mut ValueId,
    ) {
        let constraint_simplification_mapping =
            constraint_simplification_mappings.entry(*side_effects_enabled_var).or_default();
        let instruction = Self::resolve_instruction(id, dfg, constraint_simplification_mapping);
        let old_results = dfg.instruction_results(id).to_vec();

        // If a copy of this instruction exists earlier in the block, then reuse the previous results.
        if let Some(cached_results) =
            Self::get_cached(dfg, instruction_result_cache, &instruction, *side_effects_enabled_var)
        {
            Self::replace_result_ids(dfg, &old_results, cached_results);
            return;
        }

        // Otherwise, try inserting the instruction again to apply any optimizations using the newly resolved inputs.
        let new_results = Self::push_instruction(id, instruction.clone(), &old_results, block, dfg);

        Self::replace_result_ids(dfg, &old_results, &new_results);

        self.cache_instruction(
            instruction.clone(),
            new_results,
            dfg,
            instruction_result_cache,
            constraint_simplification_mapping,
            *side_effects_enabled_var,
        );

        // If we just inserted an `Instruction::EnableSideEffectsIf`, we need to update `side_effects_enabled_var`
        // so that we use the correct set of constrained values in future.
        if let Instruction::EnableSideEffectsIf { condition } = instruction {
            *side_effects_enabled_var = condition;
        };
    }

    /// Fetches an [`Instruction`] by its [`InstructionId`] and fully resolves its inputs.
    fn resolve_instruction(
        instruction_id: InstructionId,
        dfg: &DataFlowGraph,
        constraint_simplification_mapping: &HashMap<ValueId, ValueId>,
    ) -> Instruction {
        let instruction = dfg[instruction_id].clone();

        // Alternate between resolving `value_id` in the `dfg` and checking to see if the resolved value
        // has been constrained to be equal to some simpler value in the current block.
        //
        // This allows us to reach a stable final `ValueId` for each instruction input as we add more
        // constraints to the cache.
        fn resolve_cache(
            dfg: &DataFlowGraph,
            cache: &HashMap<ValueId, ValueId>,
            value_id: ValueId,
        ) -> ValueId {
            let resolved_id = dfg.resolve(value_id);
            match cache.get(&resolved_id) {
                Some(cached_value) => resolve_cache(dfg, cache, *cached_value),
                None => resolved_id,
            }
        }

        // Resolve any inputs to ensure that we're comparing like-for-like instructions.
        instruction
            .map_values(|value_id| resolve_cache(dfg, constraint_simplification_mapping, value_id))
    }

    /// Pushes a new [`Instruction`] into the [`DataFlowGraph`] which applies any optimizations
    /// based on newly resolved values for its inputs.
    ///
    /// This may result in the [`Instruction`] being optimized away or replaced with a constant value.
    fn push_instruction(
        id: InstructionId,
        instruction: Instruction,
        old_results: &[ValueId],
        block: BasicBlockId,
        dfg: &mut DataFlowGraph,
    ) -> Vec<ValueId> {
        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(old_results, |result| dfg.type_of_value(*result)));

        let call_stack = dfg.get_call_stack(id);
        let new_results =
            match dfg.insert_instruction_and_results(instruction, block, ctrl_typevars, call_stack)
            {
                InsertInstructionResult::SimplifiedTo(new_result) => vec![new_result],
                InsertInstructionResult::SimplifiedToMultiple(new_results) => new_results,
                InsertInstructionResult::Results(_, new_results) => new_results.to_vec(),
                InsertInstructionResult::InstructionRemoved => vec![],
            };
        // Optimizations while inserting the instruction should not change the number of results.
        assert_eq!(old_results.len(), new_results.len());

        new_results
    }

    fn cache_instruction(
        &self,
        instruction: Instruction,
        instruction_results: Vec<ValueId>,
        dfg: &DataFlowGraph,
        instruction_result_cache: &mut InstructionResultCache,
        constraint_simplification_mapping: &mut HashMap<ValueId, ValueId>,
        side_effects_enabled_var: ValueId,
    ) {
        if self.use_constraint_info {
            // If the instruction was a constraint, then create a link between the two `ValueId`s
            // to map from the more complex to the simpler value.
            if let Instruction::Constrain(lhs, rhs, _) = instruction {
                // These `ValueId`s should be fully resolved now.
                match (&dfg[lhs], &dfg[rhs]) {
                    // Ignore trivial constraints
                    (Value::NumericConstant { .. }, Value::NumericConstant { .. }) => (),

                    // Prefer replacing with constants where possible.
                    (Value::NumericConstant { .. }, _) => {
                        constraint_simplification_mapping.insert(rhs, lhs);
                    }
                    (_, Value::NumericConstant { .. }) => {
                        constraint_simplification_mapping.insert(lhs, rhs);
                    }
                    // Otherwise prefer block parameters over instruction results.
                    // This is as block parameters are more likely to be a single witness rather than a full expression.
                    (Value::Param { .. }, Value::Instruction { .. }) => {
                        constraint_simplification_mapping.insert(rhs, lhs);
                    }
                    (Value::Instruction { .. }, Value::Param { .. }) => {
                        constraint_simplification_mapping.insert(lhs, rhs);
                    }
                    (_, _) => (),
                }
            }
        }

        // If the instruction doesn't have side-effects and if it won't interact with enable_side_effects during acir_gen,
        // we cache the results so we can reuse them if the same instruction appears again later in the block.
        if instruction.can_be_deduplicated(dfg, self.use_constraint_info) {
            let use_predicate =
                self.use_constraint_info && instruction.requires_acir_gen_predicate(dfg);
            let predicate = use_predicate.then_some(side_effects_enabled_var);

            instruction_result_cache
                .entry(instruction)
                .or_default()
                .insert(predicate, instruction_results);
        }
    }

    /// Replaces a set of [`ValueId`]s inside the [`DataFlowGraph`] with another.
    fn replace_result_ids(
        dfg: &mut DataFlowGraph,
        old_results: &[ValueId],
        new_results: &[ValueId],
    ) {
        for (old_result, new_result) in old_results.iter().zip(new_results) {
            dfg.set_value_from_id(*old_result, *new_result);
        }
    }

    fn get_cached<'a>(
        dfg: &DataFlowGraph,
        instruction_result_cache: &'a mut InstructionResultCache,
        instruction: &Instruction,
        side_effects_enabled_var: ValueId,
    ) -> Option<&'a Vec<ValueId>> {
        let results_for_instruction = instruction_result_cache.get(instruction);

        // See if there's a cached version with no predicate first
        if let Some(results) = results_for_instruction.and_then(|map| map.get(&None)) {
            return Some(results);
        }

        let predicate =
            instruction.requires_acir_gen_predicate(dfg).then_some(side_effects_enabled_var);

        results_for_instruction.and_then(|map| map.get(&predicate))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{map::Id, types::Type},
        opt::assert_normalized_ssa_equals,
        Ssa,
    };

    #[test]
    fn simple_constant_fold() {
        // After constructing this IR, we set the value of v0 to 2.
        // The expected return afterwards should be 9.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                v1 = add v0, Field 1
                v2 = mul v1, Field 3
                return v2
            }
            ";
        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v0 = main.parameters()[0];
        let two = main.dfg.make_constant(2_u128.into(), Type::field());

        main.dfg.set_value_from_id(v0, two);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                return Field 9
            }
            ";
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn redundant_truncation() {
        // After constructing this IR, we set the value of v1 to 2^8.
        // The expected return afterwards should be v2.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v2 = div v0, v1
                v3 = truncate v2 to 8 bits, max_bit_size: 16
                return v3
            }
            ";
        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v1 = main.parameters()[1];

        // Note that this constant guarantees that `v0/constant < 2^8`. We then do not need to truncate the result.
        let constant = 2_u128.pow(8);
        let constant = main.dfg.make_constant(constant.into(), Type::unsigned(16));

        main.dfg.set_value_from_id(v1, constant);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v3 = div v0, u16 256
                return v3
            }
            ";

        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn non_redundant_truncation() {
        // After constructing this IR, we set the value of v1 to 2^8 - 1.
        // This should not result in the truncation being removed.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v2 = div v0, v1
                v3 = truncate v2 to 8 bits, max_bit_size: 16
                return v3
            }
            ";
        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v1 = main.parameters()[1];

        // Note that this constant does not guarantee that `v0/constant < 2^8`. We must then truncate the result.
        let constant = 2_u128.pow(8) - 1;
        let constant = main.dfg.make_constant(constant.into(), Type::unsigned(16));

        main.dfg.set_value_from_id(v1, constant);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v3 = div v0, u16 255
                v4 = truncate v3 to 8 bits, max_bit_size: 16
                return v4
            }
            ";

        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn arrays_elements_are_updated() {
        // After constructing this IR, we run constant folding with no expected benefit, but to
        // ensure that all new values ids are correctly propagated.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                v2 = add v0, Field 1
                v3 = make_array [v2] : [Field; 1]
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn instruction_deduplication() {
        // After constructing this IR, we run constant folding which should replace the second cast
        // with a reference to the results to the first. This then allows us to optimize away
        // the constrain instruction as both inputs are known to be equal.
        //
        // The first cast instruction is retained and will be removed in the dead instruction elimination pass.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u16):
                v1 = cast v0 as u32
                v2 = cast v0 as u32
                constrain v1 == v2
                return
            }
            ";
        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u16):
                v1 = cast v0 as u32
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn constant_index_array_access_deduplication() {
        // After constructing this IR, we run constant folding which should replace the second constant-index array get
        // with a reference to the results to the first. This then allows us to optimize away
        // the constrain instruction as both inputs are known to be equal.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: [Field; 4], v1: u32, v2: bool, v3: bool):
                enable_side_effects v2
                v4 = array_get v0, index u32 0 -> Field
                v5 = array_get v0, index v1 -> Field
                enable_side_effects v3
                v6 = array_get v0, index u32 0 -> Field
                v7 = array_get v0, index v1 -> Field
                constrain v4 == v6
                return
            }
            ";
        let expected = "
            acir(inline) fn main f0 {
              b0(v0: [Field; 4], v1: u32, v2: u1, v3: u1):
                enable_side_effects v2
                v5 = array_get v0, index u32 0 -> Field
                v6 = array_get v0, index v1 -> Field
                enable_side_effects v3
                v7 = array_get v0, index v1 -> Field
                return
            }
            ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn constraint_decomposition() {
        // When constructing this IR, we should automatically decompose the constraint to be in terms of `v0`, `v1` and `v2`.
        //
        // The mul instructions are retained and will be removed in the dead instruction elimination pass.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1, v2: u1):
                v3 = mul v0, v1
                v4 = not v2
                v5 = mul v3, v4
                constrain v5 == u1 1
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1, v2: u1):
                v3 = mul v0, v1
                v4 = not v2
                v5 = mul v3, v4
                constrain v0 == u1 1
                constrain v1 == u1 1
                constrain v2 == u1 0
                return
            }
            ";
        assert_normalized_ssa_equals(ssa, expected);
    }

    // Regression for #4600
    #[test]
    fn array_get_regression() {
        // We want to make sure after constant folding both array_gets remain since they are
        // under different enable_side_effects_if contexts and thus one may be disabled while
        // the other is not. If one is removed, it is possible e.g. v4 is replaced with v2 which
        // is disabled (only gets from index 0) and thus returns the wrong result.
        let src = "
             acir(inline) fn main f0 {
               b0(v0: u1, v1: u64):
                 enable_side_effects v0
                 v4 = make_array [Field 0, Field 1] : [Field; 2]
                 v5 = array_get v4, index v1 -> Field
                 v6 = not v0
                 enable_side_effects v6
                 v7 = array_get v4, index v1 -> Field
                 return
             }
             ";
        let ssa = Ssa::from_str(src).unwrap();

        // Expected output is unchanged
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn deduplicate_instructions_with_predicates() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1, v2: [Field; 2]):
                enable_side_effects v0
                v6 = array_get v2, index u32 0 -> u32
                v7 = array_set v2, index u32 1, value u32 2
                v8 = array_get v7, index u32 0 -> u32
                constrain v6 == v8
                enable_side_effects v1
                v9 = array_get v2, index u32 0 -> u32
                v10 = array_set v2, index u32 1, value u32 2
                v11 = array_get v10, index u32 0 -> u32
                constrain v9 == v11
                enable_side_effects v0
                v12 = array_get v2, index u32 0 -> u32
                v13 = array_set v2, index u32 1, value u32 2
                v14 = array_get v13, index u32 0 -> u32
                constrain v12 == v14
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 15);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1, v2: [Field; 2]):
                enable_side_effects v0
                v4 = array_get v2, index u32 0 -> u32
                v7 = array_set v2, index u32 1, value u32 2
                v8 = array_get v7, index u32 0 -> u32
                constrain v4 == v8
                enable_side_effects v1
                v9 = array_set v2, index u32 1, value u32 2
                v10 = array_get v9, index u32 0 -> u32
                constrain v4 == v10
                enable_side_effects v0
                return
            }
            ";

        let ssa = ssa.fold_constants_using_constraints();
        assert_normalized_ssa_equals(ssa, expected);
    }

    // This test currently fails. It being fixed will address the issue https://github.com/noir-lang/noir/issues/5756
    #[test]
    #[should_panic]
    fn constant_array_deduplication() {
        // fn main f0 {
        //   b0(v0: u64):
        //     v5 = call keccakf1600([v0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0])
        //     v6 = call keccakf1600([v0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0])
        // }
        //
        // Here we're checking a situation where two identical arrays are being initialized twice and being assigned separate `ValueId`s.
        // This would result in otherwise identical instructions not being deduplicated.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::unsigned(64));
        let zero = builder.numeric_constant(0u128, Type::unsigned(64));
        let typ = Type::Array(Arc::new(vec![Type::unsigned(64)]), 25);

        let array_contents = im::vector![
            v0, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero,
            zero, zero, zero, zero, zero, zero, zero, zero, zero, zero,
        ];
        let array1 = builder.insert_make_array(array_contents.clone(), typ.clone());
        let array2 = builder.insert_make_array(array_contents, typ.clone());

        assert_eq!(array1, array2, "arrays were assigned different value ids");

        let keccakf1600 =
            builder.import_intrinsic("keccakf1600").expect("keccakf1600 intrinsic should exist");
        let _v10 = builder.insert_call(keccakf1600, vec![array1], vec![typ.clone()]);
        let _v11 = builder.insert_call(keccakf1600, vec![array2], vec![typ.clone()]);

        let mut ssa = builder.finish();
        ssa.normalize_ids();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let starting_instruction_count = instructions.len();
        assert_eq!(starting_instruction_count, 2);

        let ssa = ssa.fold_constants();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let ending_instruction_count = instructions.len();
        assert_eq!(ending_instruction_count, 1);
    }
}
