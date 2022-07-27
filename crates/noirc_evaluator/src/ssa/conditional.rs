use num_bigint::BigUint;
use num_traits::One;

use crate::{errors::RuntimeError, ssa::node::ObjectType};

use super::{
    block::{self, BlockId, BlockType},
    context::SsaContext,
    node::{self, BinaryOp, Instruction, NodeId, Operation},
};

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

    pub fn make_decision_tree(&mut self, ctx: &mut SsaContext) {
        let mut join_to_process = Vec::new();
        let mut join_processed = Vec::new();
        self.decision_tree(
            ctx,
            self.root,
            ctx.first_block,
            &mut join_to_process,
            &mut join_processed,
        );
    }

    pub fn decision_tree(
        &mut self,
        ctx: &mut SsaContext,
        current_assumption: AssumptionId,
        block_id: BlockId,
        join_to_process: &mut Vec<BlockId>,
        join_processed: &mut Vec<BlockId>,
    ) {
        let assumption = &self[current_assumption];
        let block = &ctx[block_id];
        let mut block_assumption = current_assumption;
        let mut if_assumption = None;
        let mut parent = AssumptionId::dummy();
        let mut sibling = true;
        if join_processed.contains(&block_id) {
            return;
        }
        // is it an exit block?
        if join_to_process.contains(&block_id) {
            debug_assert!(block_id == *join_to_process.last().unwrap());
            block_assumption = assumption.parent;
            join_to_process.pop();
            join_processed.push(block_id);
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
                assert!(ctx[exit].kind == BlockType::IfJoin); //todo debug_assert
                if_decision.entry_block = block_id;
                if_decision.exit_block = exit;
                if_assumption = Some(if_decision);
                join_to_process.push(exit);
            }
        }
        //let's mutate
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
        ctx[block_id].assumption = block_assumption;
        self.compute_assumption(ctx, block_id);
        let block_left = &ctx[block_id].left.clone();
        let block_right = &ctx[block_id].right.clone();
        self.conditionalize_block(ctx, block_id);
        //process children
        if let Some(left) = block_left {
            self.decision_tree(ctx, left_assumption, *left, join_to_process, join_processed);
        }
        if let Some(right) = block_right {
            self.decision_tree(ctx, right_assumption, *right, join_to_process, join_processed);
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

    pub fn conditionalize_block(&self, ctx: &mut SsaContext, block: BlockId) {
        let assumption_id = ctx[block].assumption;
        let assumption = &self[assumption_id];
        let ass_value = assumption.value.unwrap();

        if ass_value == ctx.zero() {
            todo!();
        }
        let block_kind = ctx[block].kind.clone();
        let instructions = ctx[block].instructions.clone();
        for i in &instructions {
            let ins1 = ctx.get_instruction(*i);
            let ins = ins1.clone();
            match &ins.operation {
                Operation::Phi { block_args, .. } => {
                    if block_kind == BlockType::IfJoin {
                        assert_eq!(block_args.len(), 2);
                        let ins2 = ctx.get_mut_instruction(*i);
                        ins2.operation = Operation::Cond {
                            condition: assumption.condition,
                            val_true: block_args[0].0,
                            val_false: block_args[1].0,
                        }
                    }
                }

                Operation::Store { array, index, value } => {
                    if !ins.operation.is_dummy_store() && ass_value != ctx.one() {
                        let array = *array;
                        let load = Operation::Load { array, index: *index };
                        let e_type = ctx.mem[*array].element_type;
                        let dummy =
                            ctx.add_instruction(Instruction::new(load, e_type, Some(block)));
                        let pos = ctx[block]
                            .instructions
                            .iter()
                            .position(|value| *value == ins.id)
                            .unwrap();
                        ctx[block].instructions.insert(pos - 1, dummy);
                        let operation = Operation::Cond {
                            condition: ass_value,
                            val_true: *value,
                            val_false: dummy,
                        };
                        let cond =
                            ctx.add_instruction(Instruction::new(operation, e_type, Some(block)));
                        ctx[block].instructions.insert(pos, cond);
                        //store the conditional value
                        let ins2 = ctx.get_mut_instruction(*i);
                        ins2.operation = Operation::Store { array, index: *index, value: cond };
                    }
                }
                Operation::Intrinsic(_, _) => {
                    if ass_value != ctx.one() {
                        todo!();
                    }
                }

                Operation::Call(_, _) => {
                    if ass_value != ctx.one() {
                        todo!();
                    }
                }
                Operation::Constrain(expr, _) => {
                    if ass_value != ctx.one() {
                        let pos = ctx[block]
                            .instructions
                            .iter()
                            .position(|value| *value == ins.id)
                            .unwrap();
                        let operation = Operation::Cond {
                            condition: ass_value,
                            val_true: *expr,
                            val_false: ctx.one(),
                        };
                        let cond = ctx.add_instruction(Instruction::new(
                            operation,
                            ObjectType::Boolean,
                            Some(block),
                        ));
                        ctx[block].instructions.insert(pos, cond);
                    }
                }
                _ => (),
            }
        }
    }
}
