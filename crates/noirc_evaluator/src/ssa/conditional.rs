use num_bigint::BigUint;
use num_traits::One;

use crate::{
    errors::RuntimeError,
    ssa::{node::ObjectType, optim},
};

use super::{
    block::{self, BlockId, BlockType},
    context::SsaContext,
    flatten::{self, UnrollContext},
    inline::StackFrame,
    node::{self, BinaryOp, Instruction, NodeId, Operation},
};

// Functions that modify arrays work on a fresh array, which is copied to the original one,
// so that the writing to the array is made explicit and handled like all the other ones with store instructions
// we keep the original array name and add the _dup suffix for debugging purpose
const DUPLICATED: &str = "_dup";

#[derive(Debug, Clone)]
pub struct Assumption {
    pub parent: AssumptionId,
    pub val_true: Vec<AssumptionId>,
    pub val_false: Vec<AssumptionId>,
    pub condition: NodeId,
    pub entry_block: BlockId,
    pub exit_block: BlockId,
    value: Option<NodeId>,
}

impl Assumption {
    pub fn new(parent: AssumptionId) -> Assumption {
        Assumption {
            parent,
            val_true: Vec::new(),
            val_false: Vec::new(),
            condition: NodeId::dummy(),
            entry_block: BlockId::dummy(),
            exit_block: BlockId::dummy(),
            value: None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AssumptionId(pub arena::Index);

impl AssumptionId {
    pub fn dummy() -> AssumptionId {
        AssumptionId(SsaContext::dummy_id())
    }
}

#[derive(Debug, Clone)]
pub struct DecisionTree {
    arena: arena::Arena<Assumption>,
    pub root: AssumptionId,
}

impl std::ops::Index<AssumptionId> for DecisionTree {
    type Output = Assumption;

    fn index(&self, index: AssumptionId) -> &Self::Output {
        &self.arena[index.0]
    }
}

impl std::ops::IndexMut<AssumptionId> for DecisionTree {
    fn index_mut(&mut self, index: AssumptionId) -> &mut Self::Output {
        &mut self.arena[index.0]
    }
}

impl DecisionTree {
    pub fn new(ctx: &SsaContext) -> DecisionTree {
        let mut tree = DecisionTree { arena: arena::Arena::new(), root: AssumptionId::dummy() };
        let root_id = tree.new_decision_leaf(AssumptionId::dummy());
        tree.root = root_id;
        tree[root_id].value = Some(ctx.one());
        tree[root_id].condition = ctx.one();
        tree
    }

    pub fn new_decision_leaf(&mut self, parent: AssumptionId) -> AssumptionId {
        let node = Assumption::new(parent);
        AssumptionId(self.arena.insert(node))
    }

    pub fn is_true_branch(&self, assumption: AssumptionId) -> bool {
        assert_ne!(assumption, self.root);
        let parent_id = self[assumption].parent;
        debug_assert!(
            self[parent_id].val_true.contains(&assumption)
                != self[parent_id].val_false.contains(&assumption)
        );
        self[parent_id].val_true.contains(&assumption)
    }

    fn new_instruction_after_phi(
        ctx: &mut SsaContext,
        block_id: BlockId,
        operator: BinaryOp,
        lhs: NodeId,
        rhs: NodeId,
        typ: ObjectType,
    ) -> NodeId {
        let operation = Operation::binary(operator, lhs, rhs);
        let mut i = Instruction::new(operation, typ, Some(block_id));
        super::optim::simplify(ctx, &mut i).unwrap();
        if let node::Mark::ReplaceWith(replacement) = i.mark {
            return replacement;
        }
        ctx.insert_instruction_after_phi(i, block_id)
    }

    pub fn compute_assumption(&mut self, ctx: &mut SsaContext, block_id: BlockId) -> NodeId {
        let block = &ctx[block_id];
        let assumption_id = block.assumption;
        let assumption = &self[block.assumption];
        if let Some(value) = assumption.value {
            return value;
        }
        let pvalue = self[assumption.parent].value.unwrap();
        let condition = self[assumption.parent].condition;
        let ins = if self.is_true_branch(block.assumption) {
            DecisionTree::new_instruction_after_phi(
                ctx,
                block_id,
                BinaryOp::Mul,
                pvalue,
                condition,
                ObjectType::Boolean,
            )
        } else {
            let not_condition = DecisionTree::new_instruction_after_phi(
                ctx,
                block_id,
                BinaryOp::Sub { max_rhs_value: BigUint::one() },
                ctx.one(),
                condition,
                ObjectType::Boolean,
            );
            DecisionTree::new_instruction_after_phi(
                ctx,
                block_id,
                BinaryOp::Mul,
                pvalue,
                not_condition,
                ObjectType::Boolean,
            )
        };
        self[assumption_id].value = Some(ins);
        ins
    }

    pub fn make_decision_tree(&mut self, ctx: &mut SsaContext, entry_block: BlockId) {
        let mut join_to_process = Vec::new();
        let mut join_processed = Vec::new();
        let mut stack = StackFrame::new(entry_block);
        self.decision_tree(ctx, self.root, &mut join_to_process, &mut join_processed, &mut stack);
    }

    pub fn decision_tree(
        &mut self,
        ctx: &mut SsaContext,
        current_assumption: AssumptionId,
        join_to_process: &mut Vec<BlockId>,
        join_processed: &mut Vec<BlockId>,
        stack: &mut StackFrame,
    ) {
        let assumption = &self[current_assumption];
        let block = &ctx[stack.block];
        let mut block_assumption = current_assumption;
        let mut if_assumption = None;
        let mut parent = AssumptionId::dummy();
        let mut sibling = true;
        if join_processed.contains(&stack.block) {
            return;
        }
        // is it an exit block?
        if join_to_process.contains(&stack.block) {
            debug_assert!(stack.block == *join_to_process.last().unwrap());
            block_assumption = assumption.parent;
            join_to_process.pop();
            join_processed.push(stack.block);
        }
        // is it an IF block?
        if let Some(ins) = ctx.try_get_instruction(*block.instructions.last().unwrap()) {
            if !block.is_join() && ins.operation.opcode() == super::node::Opcode::Jeq {
                //add a new assuption for the IF
                if assumption.parent == AssumptionId::dummy() {
                    //Root assumption
                    parent = current_assumption;
                    sibling = true;
                } else {
                    parent = assumption.parent;
                    sibling = self[assumption.parent].val_true.contains(&current_assumption);
                };
                let mut if_decision = Assumption::new(parent);
                if let Operation::Jeq(condition, _) = ins.operation {
                    if_decision.condition = condition;
                } else {
                    unreachable!();
                }

                //find exit node:
                let exit = block::find_join(ctx, block.left.unwrap(), block.right.unwrap());
                debug_assert!(ctx[exit].kind == BlockType::IfJoin);
                if_decision.entry_block = stack.block;
                if_decision.exit_block = exit;
                if_assumption = Some(if_decision);
                join_to_process.push(exit);
            }
        }
        //generate the assumption for split blocks and assign the assumption to the block
        let mut left_assumption = block_assumption;
        let mut right_assumption = block_assumption;
        if let Some(if_decision) = if_assumption {
            block_assumption = AssumptionId(self.arena.insert(if_decision));
            if sibling {
                self[parent].val_true.push(block_assumption);
            } else {
                self[parent].val_false.push(block_assumption);
            }
            left_assumption = self.new_decision_leaf(block_assumption);
            right_assumption = self.new_decision_leaf(block_assumption);
            self[block_assumption].val_true.push(left_assumption);
            self[block_assumption].val_false.push(right_assumption);
        }
        ctx[stack.block].assumption = block_assumption;
        self.compute_assumption(ctx, stack.block);
        let block_left = &ctx[stack.block].left.clone();
        let block_right = &ctx[stack.block].right.clone();
        self.conditionalize_block(ctx, stack.block, stack);
        //process children
        if let Some(left) = block_left {
            stack.block = *left; //TODO on enleve le block des arguments
            self.decision_tree(ctx, left_assumption, join_to_process, join_processed, stack);
        }
        if let Some(right) = block_right {
            stack.block = *right;
            self.decision_tree(ctx, right_assumption, join_to_process, join_processed, stack);
        }
    }

    pub fn reduce(
        &mut self,
        ctx: &mut SsaContext,
        node_id: AssumptionId,
    ) -> Result<(), RuntimeError> {
        //reduce children
        let assumption = self[node_id].clone();
        for i in assumption.val_true {
            self.reduce(ctx, i)?;
        }
        for i in assumption.val_false {
            self.reduce(ctx, i)?;
        }
        //reduce the node
        if assumption.entry_block != BlockId::dummy() {
            DecisionTree::reduce_sub_graph(ctx, assumption.entry_block, assumption.exit_block)?;
        }
        Ok(())
    }

    //reduce if sub graph
    pub fn reduce_sub_graph(
        ctx: &mut SsaContext,
        if_block_id: BlockId,
        exit_block_id: BlockId,
    ) -> Result<(), RuntimeError> {
        //basic reduction as a first step (i.e no optimisation)
        let if_block = &ctx[if_block_id];
        let mut to_remove = Vec::new();
        let left = if_block.left.unwrap();
        let right = if_block.right.unwrap();

        //merge then branch
        to_remove.extend(block::merge_path(ctx, left, exit_block_id));

        //merge else branch
        to_remove.extend(block::merge_path(ctx, right, exit_block_id));

        //for now we just append
        to_remove.push(right);
        let mut ins = ctx[left].instructions.clone();
        ins.extend(&ctx[right].instructions);
        let mut modified = false;
        super::optim::cse_block(ctx, left, &mut ins, &mut modified)?;

        //housekeeping...
        let if_block = &mut ctx[if_block_id];
        if_block.dominated = vec![left];
        if_block.right = None;
        if_block.kind = BlockType::Normal;
        if_block.instructions.pop();

        let exit_block = &mut ctx[exit_block_id];
        exit_block.predecessor = Vec::new();
        block::rewire_block_left(ctx, left, exit_block_id);
        for i in to_remove {
            ctx.remove_block(i);
        }
        Ok(())
    }

    pub fn conditionalize_block(
        &self,
        ctx: &mut SsaContext,
        block: BlockId,
        stack: &mut StackFrame,
    ) {
        let assumption_id = ctx[block].assumption;
        let assumption = &self[assumption_id];
        let ass_value = assumption.value.unwrap();

        if ass_value == ctx.zero() {
            todo!();
        }
        let instructions = ctx[block].instructions.clone();
        self.conditionalise_inline(ctx, &instructions, stack, assumption_id);
        ctx[block].instructions.clear();
        ctx[block].instructions.append(&mut stack.stack);
        assert!(stack.stack.is_empty());
    }

    pub fn conditionalise_inline(
        &self,
        ctx: &mut SsaContext,
        instructions: &[NodeId],
        result: &mut StackFrame,
        predicate: AssumptionId,
    ) {
        for i in instructions {
            self.conditionalise_into(ctx, result, *i, predicate);
        }
    }

    //assigns the arrays to the block where they are seen for the first time
    fn new_array(ctx: &SsaContext, array_id: super::mem::ArrayId, stack: &mut StackFrame) {
        if let std::collections::hash_map::Entry::Vacant(e) = stack.created_arrays.entry(array_id) {
            if !ctx.mem[array_id].values.is_empty() {
                e.insert(ctx.first_block);
            } else {
                e.insert(stack.block);
            }
        }
    }

    pub fn conditionalise_into(
        &self,
        ctx: &mut SsaContext,
        stack: &mut StackFrame,
        ins_id: NodeId,
        predicate: AssumptionId,
    ) {
        let ass_cond;
        let ass_value;
        if predicate == AssumptionId::dummy() {
            ass_cond = NodeId::dummy();
            ass_value = NodeId::dummy();
        } else {
            ass_cond = self[predicate].condition;
            ass_value = self[predicate].value.unwrap_or_else(NodeId::dummy);
        }
        assert_ne!(ass_value, ctx.zero(), "code should have been already simplified");
        let ins1 = ctx.get_instruction(ins_id);
        match &ins1.operation {
            Operation::Call { returned_arrays, .. } => {
                for a in returned_arrays {
                    DecisionTree::new_array(ctx, a.0, stack);
                }
            }
            Operation::Store { array_id, .. } => {
                DecisionTree::new_array(ctx, *array_id, stack);
            }
            _ => {
                if let ObjectType::Pointer(a) = ins1.res_type {
                    DecisionTree::new_array(ctx, a, stack);
                }
            }
        }

        let ins = ins1.clone();
        match &ins.operation {
            Operation::Phi { block_args, .. } => {
                if ctx[stack.block].kind == BlockType::IfJoin {
                    assert_eq!(block_args.len(), 2);
                    let ins2 = ctx.get_mut_instruction(ins_id);
                    ins2.operation = Operation::Cond {
                        condition: ass_cond,
                        val_true: block_args[0].0,
                        val_false: block_args[1].0,
                    };
                    optim::simplify_id(ctx, ins_id).unwrap();
                }
                stack.push(ins_id);
            }

            Operation::Store { array_id, index, value } => {
                if !ins.operation.is_dummy_store()
                    && ctx.under_assumption(ass_value)
                    && stack.created_arrays[array_id] != stack.block
                {
                    let load = Operation::Load { array_id: *array_id, index: *index };
                    let e_type = ctx.mem[*array_id].element_type;
                    let dummy =
                        ctx.add_instruction(Instruction::new(load, e_type, Some(stack.block)));
                    let operation = Operation::Cond {
                        condition: ass_value,
                        val_true: *value,
                        val_false: dummy,
                    };
                    let cond =
                        ctx.add_instruction(Instruction::new(operation, e_type, Some(stack.block)));

                    stack.push(dummy);
                    stack.push(cond);
                    //store the conditional value
                    let ins2 = ctx.get_mut_instruction(ins_id);
                    ins2.operation =
                        Operation::Store { array_id: *array_id, index: *index, value: cond };
                }
                stack.push(ins_id);
            }
            Operation::Intrinsic(_, _) => {
                stack.push(ins_id);
                if ctx.under_assumption(ass_value) {
                    if let ObjectType::Pointer(a) = ins.res_type {
                        if stack.created_arrays[&a] != stack.block {
                            let array = &ctx.mem[a].clone();
                            let name = array.name.to_string() + DUPLICATED;
                            ctx.new_array(&name, array.element_type, array.len, None);
                            let array_dup = ctx.mem.last_id();
                            let ins2 = ctx.get_mut_instruction(ins_id);
                            ins2.res_type = ObjectType::Pointer(array_dup);

                            let mut memcpy_stack = StackFrame::new(stack.block);
                            ctx.memcpy_inline(
                                ins.res_type,
                                ObjectType::Pointer(array_dup),
                                &mut memcpy_stack,
                            );
                            self.conditionalise_inline(ctx, &memcpy_stack.stack, stack, predicate);
                        }
                    }
                }
            }

            Operation::Call {
                func_id, arguments, returned_arrays, predicate: ins_pred, ..
            } => {
                if ctx.under_assumption(ass_value) {
                    assert!(*ins_pred == AssumptionId::dummy());
                    let mut ins2 = ctx.get_mut_instruction(ins_id);
                    ins2.operation = Operation::Call {
                        func_id: *func_id,
                        arguments: arguments.clone(),
                        returned_arrays: returned_arrays.clone(),
                        predicate,
                    };
                }
                stack.push(ins_id);
            }
            Operation::Constrain(expr, loc) => {
                if ctx.under_assumption(ass_value) {
                    let operation = Operation::Cond {
                        condition: ass_value,
                        val_true: *expr,
                        val_false: ctx.one(),
                    };
                    let cond = ctx.add_instruction(Instruction::new(
                        operation,
                        ObjectType::Boolean,
                        Some(stack.block),
                    ));
                    stack.push(cond);
                    let ins2 = ctx.get_mut_instruction(ins_id);
                    ins2.operation = Operation::Constrain(cond, *loc);
                }
                stack.push(ins_id);
            }
            _ => stack.push(ins_id),
        }
    }
}

//unroll an if sub-graph
pub fn unroll_if(
    ctx: &mut SsaContext,
    unroll_ctx: &mut UnrollContext,
) -> Result<BlockId, RuntimeError> {
    //1. find the join block
    let if_block = &ctx[unroll_ctx.to_unroll];
    let left = if_block.left.unwrap();
    let right = if_block.right.unwrap();
    debug_assert!(if_block.kind == BlockType::Normal);
    let exit = block::find_join(ctx, if_block.left.unwrap(), if_block.right.unwrap());

    // simple mode:
    if unroll_ctx.unroll_into == BlockId::dummy() || unroll_ctx.unroll_into == unroll_ctx.to_unroll
    {
        unroll_ctx.unroll_into = unroll_ctx.to_unroll;
        flatten::unroll_std_block(ctx, unroll_ctx)?;
        unroll_ctx.to_unroll = left;
        unroll_ctx.unroll_into = left;
        flatten::unroll_std_block(ctx, unroll_ctx)?;
        unroll_ctx.to_unroll = right;
        unroll_ctx.unroll_into = right;
        flatten::unroll_std_block(ctx, unroll_ctx)?;
        unroll_ctx.to_unroll = exit;
        unroll_ctx.unroll_into = exit;
        return Ok(exit);
    }

    //2. create the IF subgraph
    //the unroll_into is required and will be used as the prev block
    let prev = unroll_ctx.unroll_into;
    let (new_entry, new_exit) = create_if_subgraph(ctx, prev);
    unroll_ctx.unroll_into = new_entry;

    //3 Process the entry_block
    flatten::unroll_std_block(ctx, unroll_ctx)?;

    //4. Process the THEN branch
    let then_block = ctx[new_entry].left.unwrap();
    let else_block = ctx[new_entry].right.unwrap();
    unroll_ctx.to_unroll = left;
    unroll_ctx.unroll_into = then_block;
    flatten::unroll_until(ctx, unroll_ctx, exit)?;

    //Plug to the exit:
    ctx[unroll_ctx.unroll_into].left = Some(new_exit);
    ctx[new_exit].predecessor.push(unroll_ctx.unroll_into);

    //5. Process the ELSE branch
    unroll_ctx.to_unroll = right;
    unroll_ctx.unroll_into = else_block;
    flatten::unroll_until(ctx, unroll_ctx, exit)?;
    ctx[unroll_ctx.unroll_into].left = Some(new_exit);
    ctx[new_exit].predecessor.push(unroll_ctx.unroll_into);

    //6. Prepare the process for the JOIN
    unroll_ctx.to_unroll = exit;
    unroll_ctx.unroll_into = new_exit;

    Ok(exit)
}

//create the subgraph for unrolling IF statement
fn create_if_subgraph(ctx: &mut SsaContext, prev_block: BlockId) -> (BlockId, BlockId) {
    //Entry block
    ctx.current_block = prev_block;
    let new_entry = block::new_sealed_block(ctx, block::BlockType::Normal, true);
    //Then block
    ctx.current_block = new_entry;
    block::new_sealed_block(ctx, block::BlockType::Normal, true);
    //Else block
    ctx.current_block = new_entry;
    let new_else = block::new_sealed_block(ctx, block::BlockType::Normal, false);
    //Exit block
    let new_exit = block::new_sealed_block(ctx, block::BlockType::IfJoin, false);
    ctx[new_exit].dominator = Some(new_entry);
    ctx[new_entry].right = Some(new_else);

    (new_entry, new_exit)
}
