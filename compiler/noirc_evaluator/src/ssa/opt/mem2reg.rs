//! The goal of the mem2reg SSA optimization pass is to replace any `Load` instructions to known
//! addresses with the value stored at that address, if it is also known. This pass will also remove
//! any `Store` instructions within a block that are no longer needed because no more loads occur in
//! between the Store in question and the next Store.
//!
//! The pass works as follows:
//! - Each block in each function is iterated in forward-order.
//! - The starting value of each reference in the block is the unification of the same references
//!   at the end of each direct predecessor block to the current block.
//! - At each step, the value of each reference is either Known(ValueId) or Unknown.
//! - Two reference values unify to each other if they are exactly equal, or to Unknown otherwise.
//! - If a block has no predecessors, the starting value of each reference is Unknown.
//! - Throughout this pass, aliases of each reference are also tracked.
//!   - References typically have 1 alias - themselves.
//!   - A reference with multiple aliases means we will not be able to optimize out loads if the
//!     reference is stored to. Note that this means we can still optimize out loads if these
//!     aliased references are never stored to, or the store occurs after a load.
//!   - A reference with 0 aliases means we were unable to find which reference this reference
//!     refers to. If such a reference is stored to, we must conservatively invalidate every
//!     reference in the current block.
//!
//! From there, to figure out the value of each reference at the end of block, iterate each instruction:
//! - On `Instruction::Allocate`:
//!   - Register a new reference was made with itself as its only alias
//! - On `Instruction::Load { address }`:
//!   - If `address` is known to only have a single alias (including itself) and if the value of
//!     that alias is known, replace the value of the load with the known value.
//!   - Furthermore, if the result of the load is a reference, mark the result as an alias
//!     of the reference it dereferences to (if known).
//!     - If which reference it dereferences to is not known, this load result has no aliases.
//! - On `Instruction::Store { address, value }`:
//!   - If the address of the store is known:
//!     - If the address has exactly 1 alias:
//!       - Set the value of the address to `Known(value)`.
//!     - If the address has more than 1 alias:
//!       - Set the value of every possible alias to `Unknown`.
//!     - If the address has 0 aliases:
//!       - Conservatively mark every alias in the block to `Unknown`.
//!   - If the address of the store is not known:
//!     - Conservatively mark every alias in the block to `Unknown`.
//!   - Additionally, if there were no Loads to any alias of the address between this Store and
//!     the previous Store to the same address, the previous store can be removed.
//! - On `Instruction::Call { arguments }`:
//!   - If any argument of the call is a reference, set the value of each alias of that
//!     reference to `Unknown`
//!   - Any builtin functions that may return aliases if their input also contains a
//!     reference should be tracked. Examples: `slice_push_back`, `slice_insert`, `slice_remove`, etc.
//!
//! On a terminator instruction:
//! - If the terminator is a `Jmp`:
//!   - For each reference argument of the jmp, mark the corresponding block parameter it is passed
//!     to as an alias for the jmp argument.
//!
//! Finally, if this is the only block in the function, we can remove any Stores that were not
//! referenced by the terminator instruction.
//!
//! Repeating this algorithm for each block in the function in program order should result in
//! optimizing out most known loads. However, identifying all aliases correctly has been proven
//! undecidable in general (Landi, 1992). So this pass will not always optimize out all loads
//! that could theoretically be optimized out. This pass can be performed at any time in the
//! SSA optimization pipeline, although it will be more successful the simpler the program's CFG is.
//! This pass is currently performed several times to enable other passes - most notably being
//! performed before loop unrolling to try to allow for mutable variables used for loop indices.
mod alias_set;
mod block;

use std::collections::{BTreeMap, BTreeSet};

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        post_order::PostOrder,
        types::Type,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use self::alias_set::AliasSet;
use self::block::{Block, Expression};

impl Ssa {
    /// Attempts to remove any load instructions that recover values that are already available in
    /// scope, and attempts to remove stores that are subsequently redundant.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn mem2reg(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.mem2reg();
        }
        self
    }
}

impl Function {
    pub(crate) fn mem2reg(&mut self) {
        let mut context = PerFunctionContext::new(self);
        context.mem2reg();
        context.remove_instructions();
        context.update_data_bus();
    }
}

struct PerFunctionContext<'f> {
    cfg: ControlFlowGraph,
    post_order: PostOrder,

    blocks: BTreeMap<BasicBlockId, Block>,

    inserter: FunctionInserter<'f>,

    /// Load and Store instructions that should be removed at the end of the pass.
    ///
    /// We avoid removing individual instructions as we go since removing elements
    /// from the middle of Vecs many times will be slower than a single call to `retain`.
    instructions_to_remove: HashSet<InstructionId>,

    /// Track a value's last load across all blocks.
    /// If a value is not used in anymore loads we can remove the last store to that value.
    last_loads: HashMap<ValueId, (InstructionId, BasicBlockId)>,

    /// Track whether a reference was passed into another entry point
    /// This is needed to determine whether we can remove a store.
    calls_reference_input: HashSet<ValueId>,

    /// Track whether a reference has been aliased, and store the respective
    /// instruction that aliased that reference.
    /// If that store has been set for removal, we can also remove this instruction.
    aliased_references: HashMap<ValueId, HashSet<InstructionId>>,
}

impl<'f> PerFunctionContext<'f> {
    fn new(function: &'f mut Function) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_function(function);

        PerFunctionContext {
            cfg,
            post_order,
            inserter: FunctionInserter::new(function),
            blocks: BTreeMap::new(),
            instructions_to_remove: HashSet::default(),
            last_loads: HashMap::default(),
            calls_reference_input: HashSet::default(),
            aliased_references: HashMap::default(),
        }
    }

    /// Apply the mem2reg pass to the given function.
    ///
    /// This function is expected to be the same one that the internal cfg, post_order, and
    /// dom_tree were created from.
    fn mem2reg(&mut self) {
        // Iterate each block in reverse post order = forward order
        let mut block_order = PostOrder::with_function(self.inserter.function).into_vec();
        block_order.reverse();

        for block in block_order {
            let references = self.find_starting_references(block);
            self.analyze_block(block, references);
        }

        let mut all_terminator_values = HashSet::default();
        let mut per_func_block_params: HashSet<ValueId> = HashSet::default();
        for (block_id, _) in self.blocks.iter() {
            let block_params = self.inserter.function.dfg.block_parameters(*block_id);
            per_func_block_params.extend(block_params.iter());
            let terminator = self.inserter.function.dfg[*block_id].unwrap_terminator();
            terminator.for_each_value(|value| all_terminator_values.insert(value));
        }

        // If we never load from an address within a function we can remove all stores to that address.
        // This rule does not apply to reference parameters, which we must also check for before removing these stores.
        for (_, block) in self.blocks.iter() {
            for (store_address, store_instruction) in block.last_stores.iter() {
                let store_alias_used = self.is_store_alias_used(
                    store_address,
                    block,
                    &all_terminator_values,
                    &per_func_block_params,
                );

                let is_dereference = block
                    .expressions
                    .get(store_address)
                    .map_or(false, |expression| matches!(expression, Expression::Dereference(_)));

                if self.last_loads.get(store_address).is_none()
                    && !store_alias_used
                    && !is_dereference
                {
                    self.instructions_to_remove.insert(*store_instruction);
                }
            }
        }
    }

    // Extra checks on where a reference can be used aside a load instruction.
    // Even if all loads to a reference have been removed we need to make sure that
    // an allocation did not come from an entry point or was passed to an entry point.
    fn is_store_alias_used(
        &self,
        store_address: &ValueId,
        block: &Block,
        all_terminator_values: &HashSet<ValueId>,
        per_func_block_params: &HashSet<ValueId>,
    ) -> bool {
        let reference_parameters = self.reference_parameters();

        if let Some(expression) = block.expressions.get(store_address) {
            if let Some(aliases) = block.aliases.get(expression) {
                let allocation_aliases_parameter =
                    aliases.any(|alias| reference_parameters.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    return true;
                }

                let allocation_aliases_parameter =
                    aliases.any(|alias| per_func_block_params.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    return true;
                }

                let allocation_aliases_parameter =
                    aliases.any(|alias| self.calls_reference_input.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    return true;
                }

                let allocation_aliases_parameter =
                    aliases.any(|alias| all_terminator_values.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    return true;
                }

                let allocation_aliases_parameter = aliases.any(|alias| {
                    if let Some(alias_instructions) = self.aliased_references.get(&alias) {
                        self.instructions_to_remove.is_disjoint(alias_instructions)
                    } else {
                        false
                    }
                });
                if allocation_aliases_parameter == Some(true) {
                    return true;
                }
            }
        }

        false
    }

    /// Collect the input parameters of the function which are of reference type.
    /// All references are mutable, so these inputs are shared with the function caller
    /// and thus stores should not be eliminated, even if the blocks in this function
    /// don't use them anywhere.
    fn reference_parameters(&self) -> BTreeSet<ValueId> {
        let parameters = self.inserter.function.parameters().iter();
        parameters
            .filter(|param| self.inserter.function.dfg.value_is_reference(**param))
            .copied()
            .collect()
    }

    /// The value of each reference at the start of the given block is the unification
    /// of the value of the same reference at the end of its predecessor blocks.
    fn find_starting_references(&mut self, block: BasicBlockId) -> Block {
        let mut predecessors = self.cfg.predecessors(block);

        if let Some(first_predecessor) = predecessors.next() {
            let mut first = self.blocks.get(&first_predecessor).cloned().unwrap_or_default();
            first.last_stores.clear();

            // Note that we have to start folding with the first block as the accumulator.
            // If we started with an empty block, an empty block union'd with any other block
            // is always also empty so we'd never be able to track any references across blocks.
            predecessors.fold(first, |block, predecessor| {
                let predecessor = self.blocks.entry(predecessor).or_default();
                block.unify(predecessor)
            })
        } else {
            Block::default()
        }
    }

    /// Analyze a block with the given starting reference values.
    ///
    /// This will remove any known loads in the block and track the value of references
    /// as they are stored to. When this function is finished, the value of each reference
    /// at the end of this block will be remembered in `self.blocks`.
    fn analyze_block(&mut self, block: BasicBlockId, mut references: Block) {
        let instructions = self.inserter.function.dfg[block].take_instructions();

        // If this is the entry block, take all the block parameters and assume they may
        // be aliased to each other
        if block == self.inserter.function.entry_block() {
            self.add_aliases_for_reference_parameters(block, &mut references);
        }

        for instruction in instructions {
            self.analyze_instruction(block, &mut references, instruction);
        }

        self.handle_terminator(block, &mut references);

        // If there's only 1 block in the function total, we can remove any remaining last stores
        // as well. We can't do this if there are multiple blocks since subsequent blocks may
        // reference these stores.
        if self.post_order.as_slice().len() == 1 {
            self.remove_stores_that_do_not_alias_parameters(&references);
        }

        self.blocks.insert(block, references);
    }

    /// Go through each parameter and register that all reference parameters of the same type are
    /// possibly aliased to each other. If there are parameters with nested references (arrays of
    /// references or references containing other references) we give up and assume all parameter
    /// references are `AliasSet::unknown()`.
    fn add_aliases_for_reference_parameters(&self, block: BasicBlockId, references: &mut Block) {
        let dfg = &self.inserter.function.dfg;
        let params = dfg.block_parameters(block);

        let mut aliases: HashMap<Type, AliasSet> = HashMap::default();

        for param in params {
            match dfg.type_of_value(*param) {
                // If the type indirectly contains a reference we have to assume all references
                // are unknown since we don't have any ValueIds to use.
                Type::Reference(element) if element.contains_reference() => return,
                Type::Reference(element) => {
                    let empty_aliases = AliasSet::known_empty();
                    let alias_set =
                        aliases.entry(element.as_ref().clone()).or_insert(empty_aliases);
                    alias_set.insert(*param);
                }
                typ if typ.contains_reference() => return,
                _ => continue,
            }
        }

        for aliases in aliases.into_values() {
            let first = aliases.first();
            let first = first.expect("All parameters alias at least themselves or we early return");

            let expression = Expression::Other(first);
            let previous = references.aliases.insert(expression.clone(), aliases.clone());
            assert!(previous.is_none());

            aliases.for_each(|alias| {
                let previous = references.expressions.insert(alias, expression.clone());
                assert!(previous.is_none());
            });
        }
    }

    /// Add all instructions in `last_stores` to `self.instructions_to_remove` which do not
    /// possibly alias any parameters of the given function.
    fn remove_stores_that_do_not_alias_parameters(&mut self, references: &Block) {
        let reference_parameters = self.reference_parameters();

        for (allocation, instruction) in &references.last_stores {
            if let Some(expression) = references.expressions.get(allocation) {
                if let Some(aliases) = references.aliases.get(expression) {
                    let allocation_aliases_parameter =
                        aliases.any(|alias| reference_parameters.contains(&alias));

                    // If `allocation_aliases_parameter` is known to be false
                    if allocation_aliases_parameter == Some(false) {
                        self.instructions_to_remove.insert(*instruction);
                    }
                }
            }
        }
    }

    fn analyze_instruction(
        &mut self,
        block_id: BasicBlockId,
        references: &mut Block,
        mut instruction: InstructionId,
    ) {
        // If the instruction was simplified and optimized out of the program we shouldn't analyze
        // it. Analyzing it could make tracking aliases less accurate if it is e.g. an ArrayGet
        // call that used to hold references but has since been optimized out to a known result.
        if let Some(new_id) = self.inserter.push_instruction(instruction, block_id) {
            instruction = new_id;
        } else {
            return;
        }

        match &self.inserter.function.dfg[instruction] {
            Instruction::Load { address } => {
                let address = self.inserter.function.dfg.resolve(*address);

                let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                references.remember_dereference(self.inserter.function, address, result);

                // If the load is known, replace it with the known value and remove the load
                if let Some(value) = references.get_known_value(address) {
                    let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                    self.inserter.map_value(result, value);
                    self.instructions_to_remove.insert(instruction);
                } else {
                    references.mark_value_used(address, self.inserter.function);

                    self.last_loads.insert(address, (instruction, block_id));
                }
            }
            Instruction::Store { address, value } => {
                let address = self.inserter.function.dfg.resolve(*address);
                let value = self.inserter.function.dfg.resolve(*value);

                // FIXME: This causes errors in the sha256 tests
                //
                // If there was another store to this instruction without any (unremoved) loads or
                // function calls in-between, we can remove the previous store.
                // if let Some(last_store) = references.last_stores.get(&address) {
                //     self.instructions_to_remove.insert(*last_store);
                // }

                if self.inserter.function.dfg.value_is_reference(value) {
                    if let Some(expression) = references.expressions.get(&value) {
                        if let Some(aliases) = references.aliases.get(expression) {
                            aliases.for_each(|alias| {
                                self.aliased_references
                                    .entry(alias)
                                    .or_default()
                                    .insert(instruction);
                            });
                        }
                    }
                }

                references.set_known_value(address, value);
                references.last_stores.insert(address, instruction);
            }
            Instruction::Allocate => {
                // Register the new reference
                let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                references.expressions.insert(result, Expression::Other(result));
                references.aliases.insert(Expression::Other(result), AliasSet::known(result));
            }
            Instruction::ArrayGet { array, .. } => {
                let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                references.mark_value_used(*array, self.inserter.function);

                if self.inserter.function.dfg.value_is_reference(result) {
                    let array = self.inserter.function.dfg.resolve(*array);
                    let expression = Expression::ArrayElement(Box::new(Expression::Other(array)));

                    if let Some(aliases) = references.aliases.get_mut(&expression) {
                        aliases.insert(result);
                    }
                }
            }
            Instruction::ArraySet { array, value, .. } => {
                references.mark_value_used(*array, self.inserter.function);
                let element_type = self.inserter.function.dfg.type_of_value(*value);

                if Self::contains_references(&element_type) {
                    let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                    let array = self.inserter.function.dfg.resolve(*array);

                    let expression = Expression::ArrayElement(Box::new(Expression::Other(array)));

                    let mut aliases = if let Some(aliases) = references.aliases.get_mut(&expression)
                    {
                        aliases.clone()
                    } else if let Some((elements, _)) =
                        self.inserter.function.dfg.get_array_constant(array)
                    {
                        let aliases = references.collect_all_aliases(elements);
                        self.set_aliases(references, array, aliases.clone());
                        aliases
                    } else {
                        AliasSet::unknown()
                    };

                    aliases.unify(&references.get_aliases_for_value(*value));

                    references.expressions.insert(result, expression.clone());
                    references.aliases.insert(expression, aliases);
                }
            }
            Instruction::Call { arguments, .. } => {
                for arg in arguments {
                    if self.inserter.function.dfg.value_is_reference(*arg) {
                        if let Some(expression) = references.expressions.get(arg) {
                            if let Some(aliases) = references.aliases.get(expression) {
                                aliases.for_each(|alias| {
                                    self.calls_reference_input.insert(alias);
                                });
                            }
                        }
                    }
                }
                self.mark_all_unknown(arguments, references);
            }
            Instruction::MakeArray { elements, typ } => {
                // If `array` is an array constant that contains reference types, then insert each element
                // as a potential alias to the array itself.
                if Self::contains_references(typ) {
                    let array = self.inserter.function.dfg.instruction_results(instruction)[0];

                    let expr = Expression::ArrayElement(Box::new(Expression::Other(array)));
                    references.expressions.insert(array, expr.clone());
                    let aliases = references.aliases.entry(expr).or_default();

                    for element in elements {
                        aliases.insert(*element);
                    }
                }
            }
            _ => (),
        }
    }

    fn contains_references(typ: &Type) -> bool {
        match typ {
            Type::Numeric(_) => false,
            Type::Function => false,
            Type::Reference(_) => true,
            Type::Array(elements, _) | Type::Slice(elements) => {
                elements.iter().any(Self::contains_references)
            }
        }
    }

    fn set_aliases(&self, references: &mut Block, address: ValueId, new_aliases: AliasSet) {
        let expression =
            references.expressions.entry(address).or_insert(Expression::Other(address));
        let aliases = references.aliases.entry(expression.clone()).or_default();
        *aliases = new_aliases;
    }

    fn mark_all_unknown(&self, values: &[ValueId], references: &mut Block) {
        for value in values {
            if self.inserter.function.dfg.value_is_reference(*value) {
                let value = self.inserter.function.dfg.resolve(*value);
                references.set_unknown(value);
                references.mark_value_used(value, self.inserter.function);
            }
        }
    }

    /// Remove any instructions in `self.instructions_to_remove` from the current function.
    /// This is expected to contain any loads which were replaced and any stores which are
    /// no longer needed.
    fn remove_instructions(&mut self) {
        // The order we iterate blocks in is not important
        for block in self.post_order.as_slice() {
            self.inserter.function.dfg[*block]
                .instructions_mut()
                .retain(|instruction| !self.instructions_to_remove.contains(instruction));
        }
    }

    fn update_data_bus(&mut self) {
        let databus = self.inserter.function.dfg.data_bus.clone();
        self.inserter.function.dfg.data_bus = databus.map_values(|t| self.inserter.resolve(t));
    }

    fn handle_terminator(&mut self, block: BasicBlockId, references: &mut Block) {
        self.inserter.map_terminator_in_place(block);

        match self.inserter.function.dfg[block].unwrap_terminator() {
            TerminatorInstruction::JmpIf { .. } => (), // Nothing to do
            TerminatorInstruction::Jmp { destination, arguments, .. } => {
                let destination_parameters = self.inserter.function.dfg[*destination].parameters();
                assert_eq!(destination_parameters.len(), arguments.len());

                // Add an alias for each reference parameter
                for (parameter, argument) in destination_parameters.iter().zip(arguments) {
                    if self.inserter.function.dfg.value_is_reference(*parameter) {
                        let argument = self.inserter.function.dfg.resolve(*argument);

                        if let Some(expression) = references.expressions.get(&argument) {
                            if let Some(aliases) = references.aliases.get_mut(expression) {
                                // The argument reference is possibly aliased by this block parameter
                                aliases.insert(*parameter);
                            }
                        }
                    }
                }
            }
            TerminatorInstruction::Return { return_values, .. } => {
                // Removing all `last_stores` for each returned reference is more important here
                // than setting them all to ReferenceValue::Unknown since no other block should
                // have a block with a Return terminator as a predecessor anyway.
                self.mark_all_unknown(return_values, references);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use acvm::{acir::AcirField, FieldElement};
    use im::vector;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            basic_block::BasicBlockId,
            dfg::DataFlowGraph,
            instruction::{BinaryOp, Instruction, Intrinsic, TerminatorInstruction},
            map::Id,
            types::Type,
        },
    };

    #[test]
    fn test_simple() {
        // fn func() {
        //   b0():
        //     v0 = allocate
        //     v1 = make_array [Field 1, Field 2]
        //     store v1 in v0
        //     v2 = load v0
        //     v3 = array_get v2, index 1
        //     return v3
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(Type::Array(Arc::new(vec![Type::field()]), 2));
        let one = builder.field_constant(FieldElement::one());
        let two = builder.field_constant(FieldElement::one());

        let element_type = Arc::new(vec![Type::field()]);
        let array_type = Type::Array(element_type, 2);
        let v1 = builder.insert_make_array(vector![one, two], array_type.clone());

        builder.insert_store(v0, v1);
        let v2 = builder.insert_load(v0, array_type);
        let v3 = builder.insert_array_get(v2, one, Type::field());
        builder.terminate_with_return(vec![v3]);

        let ssa = builder.finish().mem2reg().fold_constants();

        let func = ssa.main();
        let block_id = func.entry_block();

        assert_eq!(count_loads(block_id, &func.dfg), 0);
        assert_eq!(count_stores(block_id, &func.dfg), 0);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values, .. } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[two]);
    }

    #[test]
    fn test_simple_with_call() {
        // fn func {
        //   b0():
        //     v0 = allocate
        //     store v0, Field 1
        //     v1 = load v0
        //     call f0(v0)
        //     return v1
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(Type::field());
        let one = builder.field_constant(FieldElement::one());
        builder.insert_store(v0, one);
        let v1 = builder.insert_load(v0, Type::field());
        let f0 = builder.import_intrinsic_id(Intrinsic::AssertConstant);
        builder.insert_call(f0, vec![v0], vec![]);
        builder.terminate_with_return(vec![v1]);

        let ssa = builder.finish().mem2reg();

        let func = ssa.main();
        let block_id = func.entry_block();

        assert_eq!(count_loads(block_id, &func.dfg), 0);
        assert_eq!(count_stores(block_id, &func.dfg), 1);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values, .. } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[one]);
    }

    #[test]
    fn test_simple_with_return() {
        // fn func {
        //   b0():
        //     v0 = allocate
        //     store v0, Field 1
        //     return v0
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(Type::field());
        let const_one = builder.field_constant(FieldElement::one());
        builder.insert_store(v0, const_one);
        builder.terminate_with_return(vec![v0]);

        let ssa = builder.finish().mem2reg();

        let func = ssa.main();
        let block_id = func.entry_block();

        // Store is needed by the return value, and can't be removed
        assert_eq!(count_stores(block_id, &func.dfg), 1);
        let instructions = func.dfg[block_id].instructions();
        assert_eq!(instructions.len(), 2);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values, .. } => *return_values.first().unwrap(),
            _ => unreachable!(),
        };

        // Since the mem2reg pass simplifies as it goes, the id of the allocate instruction result
        // is most likely no longer v0. We have to retrieve the new id here.
        let allocate_id = func.dfg.instruction_results(instructions[0])[0];
        assert_eq!(ret_val_id, allocate_id);
    }

    fn count_stores(block: BasicBlockId, dfg: &DataFlowGraph) -> usize {
        dfg[block]
            .instructions()
            .iter()
            .filter(|instruction_id| matches!(dfg[**instruction_id], Instruction::Store { .. }))
            .count()
    }

    fn count_loads(block: BasicBlockId, dfg: &DataFlowGraph) -> usize {
        dfg[block]
            .instructions()
            .iter()
            .filter(|instruction_id| matches!(dfg[**instruction_id], Instruction::Load { .. }))
            .count()
    }

    // Test that loads across multiple blocks are removed
    #[test]
    fn multiple_blocks() {
        // fn main {
        //   b0():
        //     v0 = allocate
        //     store Field 5 in v0
        //     v1 = load v0
        //     jmp b1(v1):
        //   b1(v2: Field):
        //     v3 = load v0
        //     store Field 6 in v0
        //     v4 = load v0
        //     return v2, v3, v4
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());

        let five = builder.field_constant(5u128);
        builder.insert_store(v0, five);

        let v1 = builder.insert_load(v0, Type::field());
        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![v1]);

        builder.switch_to_block(b1);
        let v2 = builder.add_block_parameter(b1, Type::field());
        let v3 = builder.insert_load(v0, Type::field());

        let six = builder.field_constant(6u128);
        builder.insert_store(v0, six);
        let v4 = builder.insert_load(v0, Type::field());

        builder.terminate_with_return(vec![v2, v3, v4]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 2);

        // Expected result:
        // acir fn main f0 {
        //   b0():
        //     v7 = allocate
        //     jmp b1(Field 5)
        //   b1(v3: Field):
        //     return v3, Field 5, Field 6
        // }
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 2);

        // The loads should be removed
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_loads(b1, &main.dfg), 0);

        // All stores are removed as there are no loads to the values being stored anywhere in the function.
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_stores(b1, &main.dfg), 0);

        // The jmp to b1 should also be a constant 5 now
        match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Jmp { arguments, .. }) => {
                assert_eq!(arguments.len(), 1);
                let argument =
                    main.dfg.get_numeric_constant(arguments[0]).expect("Expected constant value");
                assert_eq!(argument.to_u128(), 5);
            }
            _ => unreachable!(),
        };
    }

    // Test that a load in a predecessor block has been removed if the value
    // is later stored in a successor block
    #[test]
    fn load_aliases_in_predecessor_block() {
        // fn main {
        //     b0():
        //       v0 = allocate
        //       store Field 0 at v0
        //       v2 = allocate
        //       store v0 at v2
        //       v3 = load v2
        //       v4 = load v2
        //       jmp b1()
        //     b1():
        //       store Field 1 at v3
        //       store Field 2 at v4
        //       v7 = load v3
        //       v8 = eq v7, Field 2
        //       return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());

        let zero = builder.field_constant(0u128);
        builder.insert_store(v0, zero);

        let v2 = builder.insert_allocate(Type::Reference(Arc::new(Type::field())));
        builder.insert_store(v2, v0);

        let v3 = builder.insert_load(v2, Type::field());
        let v4 = builder.insert_load(v2, Type::field());
        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![]);

        builder.switch_to_block(b1);

        let one = builder.field_constant(1u128);
        builder.insert_store(v3, one);

        let two = builder.field_constant(2u128);
        builder.insert_store(v4, two);

        let v8 = builder.insert_load(v3, Type::field());
        let _ = builder.insert_binary(v8, BinaryOp::Eq, two);

        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 2);

        // Expected result:
        // acir fn main f0 {
        //   b0():
        //     v9 = allocate
        //     store Field 0 at v9
        //     v10 = allocate
        //     jmp b1()
        //   b1():
        //     return
        // }
        let ssa = ssa.mem2reg();
        println!("{}", ssa);

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 2);

        // All loads should be removed
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_loads(b1, &main.dfg), 0);

        // The first store is not removed as it is used as a nested reference in another store.
        // We would need to track whether the store where `v9` is the store value gets removed to know whether
        // to remove it.
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 1);

        // The first store in b1 is removed since there is another store to the same reference
        // in the same block, and the store is not needed before the later store.
        // The rest of the stores are also removed as no loads are done within any blocks
        // to the stored values.
        //
        // NOTE: This store is not removed due to the FIXME when handling Instruction::Store.
        assert_eq!(count_stores(b1, &main.dfg), 1);

        let b1_instructions = main.dfg[b1].instructions();

        // We expect the last eq to be optimized out, only the store from above remains
        assert_eq!(b1_instructions.len(), 1);
    }

    #[test]
    fn keep_store_to_alias_in_loop_block() {
        // This test makes sure the instruction `store Field 2 at v5` in b2 remains after mem2reg.
        // Although the only instruction on v5 is a lone store without any loads,
        // v5 is an alias of the reference v0 which is stored in v2.
        // This test makes sure that we are not inadvertently removing stores to aliases across blocks.
        //
        // acir(inline) fn main f0 {
        //     b0():
        //       v0 = allocate
        //       store Field 0 at v0
        //       v2 = allocate
        //       store v0 at v2
        //       jmp b1(Field 0)
        //     b1(v3: Field):
        //       v4 = eq v3, Field 0
        //       jmpif v4 then: b2, else: b3
        //     b2():
        //       v5 = load v2
        //       store Field 2 at v5
        //       v8 = add v3, Field 1
        //       jmp b1(v8)
        //     b3():
        //       v9 = load v0
        //       v10 = eq v9, Field 2
        //       constrain v9 == Field 2
        //       v11 = load v2
        //       v12 = load v10
        //       v13 = eq v12, Field 2
        //       constrain v11 == Field 2
        //       return
        //   }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());
        let zero = builder.field_constant(0u128);
        builder.insert_store(v0, zero);

        let v2 = builder.insert_allocate(Type::field());
        // Construct alias
        builder.insert_store(v2, v0);
        let v2_type = builder.current_function.dfg.type_of_value(v2);
        assert!(builder.current_function.dfg.value_is_reference(v2));

        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![zero]);

        // Loop header
        builder.switch_to_block(b1);
        let v3 = builder.add_block_parameter(b1, Type::field());
        let is_zero = builder.insert_binary(v3, BinaryOp::Eq, zero);

        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        builder.terminate_with_jmpif(is_zero, b2, b3);

        // Loop body
        builder.switch_to_block(b2);
        let v5 = builder.insert_load(v2, v2_type.clone());
        let two = builder.field_constant(2u128);
        builder.insert_store(v5, two);
        let one = builder.field_constant(1u128);
        let v3_plus_one = builder.insert_binary(v3, BinaryOp::Add, one);
        builder.terminate_with_jmp(b1, vec![v3_plus_one]);

        builder.switch_to_block(b3);
        let v9 = builder.insert_load(v0, Type::field());
        let _ = builder.insert_binary(v9, BinaryOp::Eq, two);

        builder.insert_constrain(v9, two, None);
        let v11 = builder.insert_load(v2, v2_type);
        let v12 = builder.insert_load(v11, Type::field());
        let _ = builder.insert_binary(v12, BinaryOp::Eq, two);

        builder.insert_constrain(v11, two, None);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();

        // We expect the same result as above.
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 4);

        // The store from the original SSA should remain
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 2);
        assert_eq!(count_stores(b2, &main.dfg), 1);

        assert_eq!(count_loads(b2, &main.dfg), 1);
        assert_eq!(count_loads(b3, &main.dfg), 3);
    }

    #[test]
    fn parameter_alias() {
        // Do not assume parameters are not aliased to each other.
        // The load below shouldn't be removed since `v0` could
        // be aliased to `v1`.
        //
        // fn main f0 {
        //   b0(v0: &mut Field, v1: &mut Field):
        //     store Field 0 at v0
        //     store Field 1 at v1
        //     v4 = load v0
        //     constrain v4 == Field 1
        //     return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let field_ref = Type::Reference(Arc::new(Type::field()));
        let v0 = builder.add_parameter(field_ref.clone());
        let v1 = builder.add_parameter(field_ref.clone());

        let zero = builder.field_constant(0u128);
        let one = builder.field_constant(0u128);
        builder.insert_store(v0, zero);
        builder.insert_store(v1, one);

        let v4 = builder.insert_load(v0, Type::field());
        builder.insert_constrain(v4, one, None);
        builder.terminate_with_return(Vec::new());

        let ssa = builder.finish();
        let main = ssa.main();
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 1);

        // No change expected
        let ssa = ssa.mem2reg();
        let main = ssa.main();
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 1);
    }
}
