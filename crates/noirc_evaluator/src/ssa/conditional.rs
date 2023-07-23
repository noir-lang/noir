use crate::{
    errors::{RuntimeError, RuntimeErrorKind},
    ssa::{
        block::{BlockId, BlockType},
        context::SsaContext,
        flatten::UnrollContext,
        inline::StackFrame,
        node::{Binary, BinaryOp, Instruction, Mark, NodeId, ObjectType, Opcode, Operation},
        {block, flatten, node, optimizations},
    },
};
use noirc_errors::Location;
use num_bigint::BigUint;
use num_traits::One;

// Functions that modify arrays work on a fresh array, which is copied to the original one,
// so that the writing to the array is made explicit and handled like all the other ones with store instructions
// we keep the original array name and add the _dup suffix for debugging purpose
const DUPLICATED: &str = "_dup";

#[derive(Debug, Clone)]
pub(crate) struct Assumption {
    pub(crate) parent: AssumptionId,
    pub(crate) val_true: Vec<AssumptionId>,
    pub(crate) val_false: Vec<AssumptionId>,
    pub(crate) condition: NodeId,
    pub(crate) entry_block: BlockId,
    pub(crate) exit_block: BlockId,
    value: Option<NodeId>,
}

impl Assumption {
    pub(crate) fn new(parent: AssumptionId) -> Assumption {
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
pub(crate) struct AssumptionId(pub(crate) arena::Index);

impl AssumptionId {
    pub(crate) fn dummy() -> AssumptionId {
        AssumptionId(SsaContext::dummy_id())
    }
}

//temporary data used to build the decision tree
pub(crate) struct TreeBuilder {
    pub(crate) join_to_process: Vec<BlockId>,
    pub(crate) stack: StackFrame,
}

impl TreeBuilder {
    pub(crate) fn new(entry: BlockId) -> TreeBuilder {
        TreeBuilder { join_to_process: Vec::new(), stack: StackFrame::new(entry) }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DecisionTree {
    arena: arena::Arena<Assumption>,
    pub(crate) root: AssumptionId,
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
    pub(crate) fn new(ctx: &SsaContext) -> DecisionTree {
        let mut tree = DecisionTree { arena: arena::Arena::new(), root: AssumptionId::dummy() };
        let root_id = tree.new_decision_leaf(AssumptionId::dummy());
        tree.root = root_id;
        tree[root_id].value = Some(ctx.one());
        tree[root_id].condition = ctx.one();
        tree
    }

    pub(crate) fn new_decision_leaf(&mut self, parent: AssumptionId) -> AssumptionId {
        let node = Assumption::new(parent);
        AssumptionId(self.arena.insert(node))
    }

    pub(crate) fn is_true_branch(&self, assumption: AssumptionId) -> bool {
        assert_ne!(assumption, self.root);
        let parent_id = self[assumption].parent;
        assert!(
            self[parent_id].val_true.contains(&assumption)
                != self[parent_id].val_false.contains(&assumption)
        );
        self[parent_id].val_true.contains(&assumption)
    }

    fn new_instruction(
        ctx: &mut SsaContext,
        block_id: BlockId,
        operator: BinaryOp,
        lhs: NodeId,
        rhs: NodeId,
        typ: ObjectType,
    ) -> Instruction {
        let operation = Operation::binary(operator, lhs, rhs);
        let mut i = Instruction::new(operation, typ, Some(block_id));
        super::optimizations::simplify(ctx, &mut i).unwrap();
        i
    }

    fn new_instruction_after_phi(
        ctx: &mut SsaContext,
        block_id: BlockId,
        operator: BinaryOp,
        lhs: NodeId,
        rhs: NodeId,
        typ: ObjectType,
    ) -> NodeId {
        let i = DecisionTree::new_instruction(ctx, block_id, operator, lhs, rhs, typ);
        if let node::Mark::ReplaceWith(replacement) = i.mark {
            return replacement;
        }
        ctx.insert_instruction_after_phi(i, block_id)
    }

    fn new_instruction_after(
        ctx: &mut SsaContext,
        block_id: BlockId,
        operator: BinaryOp,
        lhs: NodeId,
        rhs: NodeId,
        typ: ObjectType,
        after: NodeId,
    ) -> NodeId {
        let i = DecisionTree::new_instruction(ctx, block_id, operator, lhs, rhs, typ);
        if let node::Mark::ReplaceWith(replacement) = i.mark {
            return replacement;
        }
        let id = ctx.add_instruction(i);
        ctx.push_instruction_after(id, block_id, after)
    }

    pub(crate) fn compute_assumption(&mut self, ctx: &mut SsaContext, block_id: BlockId) -> NodeId {
        let block = &ctx[block_id];
        let assumption_id = block.assumption;
        let assumption = &self[block.assumption];
        if let Some(value) = assumption.value {
            return value;
        }
        let parent_value = self[assumption.parent].value.unwrap();
        let condition = self[assumption.parent].condition;
        let ins = if self.is_true_branch(block.assumption) {
            DecisionTree::new_instruction_after_phi(
                ctx,
                block_id,
                BinaryOp::Mul,
                parent_value,
                condition,
                ObjectType::boolean(),
            )
        } else {
            let not_condition = DecisionTree::new_instruction_after_phi(
                ctx,
                block_id,
                BinaryOp::Sub { max_rhs_value: BigUint::one() },
                ctx.one(),
                condition,
                ObjectType::boolean(),
            );
            DecisionTree::new_instruction_after(
                ctx,
                block_id,
                BinaryOp::Mul,
                parent_value,
                not_condition,
                ObjectType::boolean(),
                not_condition,
            )
        };
        self[assumption_id].value = Some(ins);
        ins
    }

    pub(crate) fn make_decision_tree(
        &mut self,
        ctx: &mut SsaContext,
        mut builder: TreeBuilder,
    ) -> Result<(), RuntimeError> {
        let entry_block = builder.stack.block;
        ctx[entry_block].assumption = self.root;
        self.decision_tree(ctx, entry_block, &mut builder)
    }

    //Returns a boolean to indicate if we should process the children (true) of not (false) of the block
    fn process_block(
        &mut self,
        ctx: &mut SsaContext,
        current: BlockId,
        data: &mut TreeBuilder,
    ) -> Result<Vec<BlockId>, RuntimeError> {
        data.stack.block = current;
        let mut block_assumption = ctx[current].assumption;
        let assumption = &self[block_assumption];
        let mut result = Vec::new();
        let current_block = &ctx[current];
        let mut if_assumption = None;
        let mut parent = AssumptionId::dummy();
        let mut sibling = true;
        let left = current_block.left;
        let right = current_block.right;
        // is it an exit block?
        if data.join_to_process.contains(&current) {
            assert!(current == *data.join_to_process.last().unwrap());
            block_assumption = assumption.parent;
            data.join_to_process.pop();
        }
        // is it an IF block?
        if let Some(ins) = ctx.get_if_condition(current_block) {
            //add a new assumption for the IF
            if assumption.parent == AssumptionId::dummy() {
                //Root assumption
                parent = block_assumption;
                sibling = true;
            } else {
                parent = assumption.parent;
                sibling = self[assumption.parent].val_true.contains(&block_assumption);
            };
            let mut if_decision = Assumption::new(parent);
            if let Operation::Jeq(condition, _) = ins.operation {
                if_decision.condition = condition;
            } else {
                unreachable!();
            }

            //find exit node:
            let exit = block::find_join(ctx, current_block.id);
            assert!(ctx[exit].kind == BlockType::IfJoin);
            if_decision.entry_block = current;
            if_decision.exit_block = exit;
            if_assumption = Some(if_decision);
            data.join_to_process.push(exit);
            result = vec![exit, right.unwrap(), left.unwrap()];
        }
        //Assumptions for the children
        if let Some(if_decision) = if_assumption {
            block_assumption = AssumptionId(self.arena.insert(if_decision));
            if sibling {
                self[parent].val_true.push(block_assumption);
            } else {
                self[parent].val_false.push(block_assumption);
            }
            //create the assumptions for else/then branches
            let left_assumption = self.new_decision_leaf(block_assumption);
            let right_assumption = self.new_decision_leaf(block_assumption);
            self[block_assumption].val_true.push(left_assumption);
            self[block_assumption].val_false.push(right_assumption);
            ctx[left.unwrap()].assumption = left_assumption;
            ctx[right.unwrap()].assumption = right_assumption;
        } else {
            match (left, right) {
                (Some(l), None) => {
                    ctx[l].assumption = block_assumption;
                    result = vec![l];
                }
                (Some(l), Some(r)) => {
                    ctx[l].assumption = block_assumption;
                    ctx[r].assumption = block_assumption;
                    result = vec![l, r];
                }
                (None, Some(_)) => {
                    unreachable!("Infallible, only a split block can have right successor")
                }
                (None, None) => (),
            }
        }

        ctx[current].assumption = block_assumption;
        self.compute_assumption(ctx, current);
        self.apply_condition_to_block(ctx, current, &mut data.stack)?;
        Ok(result)
    }

    fn decision_tree(
        &mut self,
        ctx: &mut SsaContext,
        current: BlockId,
        data: &mut TreeBuilder,
    ) -> Result<(), RuntimeError> {
        let mut queue = vec![current]; //Stack of elements to visit

        while let Some(current) = queue.pop() {
            let children = self.process_block(ctx, current, data)?;

            let mut test_and_push = |block_id: BlockId| {
                if !block_id.is_dummy() && !queue.contains(&block_id) {
                    let block = &ctx[block_id];
                    if !block.is_join() || block.dominator == Some(current) {
                        queue.push(block_id);
                    }
                }
            };

            for i in children {
                test_and_push(i);
            }
        }
        Ok(())
    }

    pub(crate) fn reduce(
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
        if !assumption.entry_block.is_dummy() {
            self.reduce_sub_graph(ctx, assumption.entry_block, assumption.exit_block)?;
        }
        Ok(())
    }

    //reduce if sub graph
    pub(crate) fn reduce_sub_graph(
        &self,
        ctx: &mut SsaContext,
        if_block_id: BlockId,
        exit_block_id: BlockId,
    ) -> Result<(), RuntimeError> {
        //basic reduction as a first step (i.e no optimization)
        let if_block = &ctx[if_block_id];
        let mut to_remove = Vec::new();
        let left = if_block.left.unwrap();
        let right = if_block.right.unwrap();
        let mut const_condition = None;
        if ctx.is_one(self[if_block.assumption].condition) {
            const_condition = Some(true);
        }
        if ctx.is_zero(self[if_block.assumption].condition) {
            const_condition = Some(false);
        }

        //merge then branch
        to_remove.extend(block::merge_path(
            ctx,
            left,
            exit_block_id,
            self[ctx[left].assumption].value,
        )?);

        //merge else branch
        to_remove.extend(block::merge_path(
            ctx,
            right,
            exit_block_id,
            self[ctx[right].assumption].value,
        )?);

        to_remove.push(right);
        let mut merged_ins;
        if let Some(const_condition) = const_condition {
            if const_condition {
                merged_ins = ctx[left].instructions.clone();
            } else {
                merged_ins = ctx[right].instructions.clone();
            }
        } else {
            let left_ins = ctx[left].instructions.clone();
            let right_ins = ctx[right].instructions.clone();
            merged_ins = self.synchronize(ctx, &left_ins, &right_ins, left);
        }
        let mut modified = false;
        // write the merged instructions to the block
        super::optimizations::cse_block(ctx, left, &mut merged_ins, &mut modified)?;
        if modified {
            // A second round is necessary when the synchronization optimizes function calls between the two branches.
            // In that case, the first cse updates the result instructions to the same call and then
            // the second cse can (and must) then simplify identical result instructions.
            // We clear the list because we want to perform the cse on the block instructions, if we don't it will use the list instead.
            // We should refactor cse_block so that its behavior is consistent and does not rely on the list being empty.
            merged_ins.clear();
            super::optimizations::cse_block(ctx, left, &mut merged_ins, &mut modified)?;
        }
        //housekeeping...
        let if_block = &mut ctx[if_block_id];
        if_block.dominated = vec![left];
        if_block.right = None;
        if_block.kind = BlockType::Normal;
        if_block.instructions.pop();

        block::rewire_block_left(ctx, left, exit_block_id);
        for i in to_remove {
            ctx.remove_block(i);
        }
        Ok(())
    }

    /// Apply the condition of the block to each instruction
    /// in the block.
    pub(crate) fn apply_condition_to_block(
        &self,
        ctx: &mut SsaContext,
        block: BlockId,
        stack: &mut StackFrame,
    ) -> Result<(), RuntimeError> {
        let assumption_id = ctx[block].assumption;
        let instructions = ctx[block].instructions.clone();
        self.apply_condition_to_instructions(ctx, &instructions, stack, assumption_id)?;
        ctx[block].instructions.clear();
        ctx[block].instructions.append(&mut stack.stack);
        assert!(stack.stack.is_empty());
        Ok(())
    }

    /// Applies a condition to each instruction
    /// and places into the stack frame.
    pub(crate) fn apply_condition_to_instructions(
        &self,
        ctx: &mut SsaContext,
        instructions: &[NodeId],
        result: &mut StackFrame,
        predicate: AssumptionId,
    ) -> Result<(), RuntimeError> {
        if predicate == AssumptionId::dummy() || self[predicate].value != Some(ctx.zero()) {
            let mut short_circuit = false;
            for i in instructions {
                if !self.apply_condition_to_instruction(
                    ctx,
                    result,
                    *i,
                    predicate,
                    short_circuit,
                )? {
                    short_circuit = true;
                }
            }
        }
        Ok(())
    }

    fn short_circuit(
        ctx: &mut SsaContext,
        stack: &mut StackFrame,
        condition: NodeId,
        error_kind: RuntimeErrorKind,
        location: Option<Location>,
    ) -> Result<(), RuntimeError> {
        if ctx.under_assumption(condition) {
            let avoid = stack.stack.contains(&condition).then_some(&condition);
            block::zero_instructions(ctx, &stack.stack, avoid);
            let nop = stack.stack[0];
            stack.stack.clear();
            stack.stack.push(nop);
            if avoid.is_some() {
                stack.stack.push(condition);
            }
            let operation =
                Operation::Cond { condition, val_true: ctx.zero(), val_false: ctx.one() };
            let cond = ctx.add_instruction(Instruction::new(
                operation,
                ObjectType::boolean(),
                Some(stack.block),
            ));
            stack.push(cond);
            let unreachable = Operation::Constrain(cond, None);
            let ins2 = ctx.add_instruction(Instruction::new(
                unreachable,
                ObjectType::NotAnObject,
                Some(stack.block),
            ));
            stack.push(ins2);
            Ok(())
        } else {
            Err(RuntimeError { location, kind: error_kind })
        }
    }

    /// Applies a condition to the instruction
    /// For most instructions, this does nothing
    /// but for instructions with side-effects
    /// this will alter the behavior.
    pub(crate) fn apply_condition_to_instruction(
        &self,
        ctx: &mut SsaContext,
        stack: &mut StackFrame,
        ins_id: NodeId,
        predicate: AssumptionId,
        short_circuit: bool,
    ) -> Result<bool, RuntimeError> {
        let ass_cond;
        let ass_value;
        if predicate == AssumptionId::dummy() {
            ass_cond = NodeId::dummy();
            ass_value = NodeId::dummy();
        } else {
            ass_cond = self[predicate].condition;
            ass_value = self[predicate].value.unwrap_or_else(NodeId::dummy);
        }
        assert!(!ctx.is_zero(ass_value), "code should have been already simplified");
        let ins1 = ctx.instruction(ins_id);
        match &ins1.operation {
            Operation::Call { returned_arrays, .. } => {
                for a in returned_arrays {
                    stack.new_array(a.0);
                }
            }
            Operation::Store { array_id, index, .. } => {
                if *index != NodeId::dummy() {
                    stack.new_array(*array_id);
                }
            }
            _ => {
                if let ObjectType::ArrayPointer(a) = ins1.res_type {
                    stack.new_array(a);
                }
            }
        }

        let ins = ins1.clone();
        if short_circuit {
            stack.set_zero(ctx, ins.res_type);
            let ins2 = ctx.instruction_mut(ins_id);
            if ins2.res_type == ObjectType::NotAnObject {
                ins2.mark = Mark::Deleted;
            } else {
                ins2.mark = Mark::ReplaceWith(stack.get_zero(ins2.res_type));
            }
        } else {
            match &ins.operation {
                Operation::Phi { block_args, .. } => {
                    if ctx[stack.block].kind == BlockType::IfJoin {
                        assert_eq!(block_args.len(), 2);
                        let ins2 = ctx.instruction_mut(ins_id);
                        ins2.operation = Operation::Cond {
                            condition: ass_cond,
                            val_true: block_args[0].0,
                            val_false: block_args[1].0,
                        };
                        optimizations::simplify_id(ctx, ins_id).unwrap();
                    }
                    stack.push(ins_id);
                }

                Operation::Load { array_id, index, location } => {
                    if let Some(idx) = ctx.get_as_constant(*index) {
                        if (idx.to_u128() as u32) >= ctx.mem[*array_id].len {
                            let error = RuntimeErrorKind::IndexOutOfBounds {
                                index: ctx.mem[*array_id].len,
                                bound: idx.to_u128(),
                            };

                            DecisionTree::short_circuit(ctx, stack, ass_value, error, *location)?;
                            return Ok(false);
                        }
                    } else {
                        // we use a 0 index when the predicate is false, to avoid out-of-bound issues
                        let ass_value = Self::unwrap_predicate(ctx, &Some(ass_value));
                        let op = Operation::Cast(ass_value);
                        let pred = ctx.add_instruction(Instruction::new(
                            op,
                            ctx.object_type(*index),
                            Some(stack.block),
                        ));
                        stack.push(pred);
                        let op = Operation::Binary(node::Binary {
                            lhs: *index,
                            rhs: pred,
                            operator: BinaryOp::Mul,
                            predicate: None,
                        });
                        let idx = ctx.add_instruction(Instruction::new(
                            op,
                            ObjectType::native_field(),
                            Some(stack.block),
                        ));
                        optimizations::simplify_id(ctx, idx).unwrap();
                        stack.push(idx);

                        let ins2 = ctx.instruction_mut(ins_id);
                        ins2.operation = Operation::Load {
                            array_id: *array_id,
                            index: idx,
                            location: *location,
                        };
                    }
                    stack.push(ins_id);
                }
                Operation::Binary(binary_op) => {
                    let mut cond = ass_value;
                    if let Some(pred) = binary_op.predicate {
                        assert_ne!(pred, NodeId::dummy());
                        if ass_value == NodeId::dummy() {
                            cond = pred;
                        } else {
                            let op = Operation::Binary(node::Binary {
                                lhs: ass_value,
                                rhs: pred,
                                operator: BinaryOp::Mul,
                                predicate: None,
                            });
                            cond = ctx.add_instruction(Instruction::new(
                                op,
                                ObjectType::boolean(),
                                Some(stack.block),
                            ));
                            optimizations::simplify_id(ctx, cond).unwrap();
                            stack.push(cond);
                        }
                    }
                    stack.push(ins_id);
                    let (side_effect, location) = match binary_op.operator {
                        BinaryOp::Udiv(loc)
                        | BinaryOp::Sdiv(loc)
                        | BinaryOp::Urem(loc)
                        | BinaryOp::Srem(loc)
                        | BinaryOp::Div(loc) => (true, Some(loc)),
                        _ => (false, None),
                    };
                    if side_effect {
                        if ctx.is_zero(binary_op.rhs) {
                            DecisionTree::short_circuit(
                                ctx,
                                stack,
                                cond,
                                RuntimeErrorKind::DivisionByZero,
                                location,
                            )?;
                            return Ok(false);
                        }
                        if ctx.under_assumption(cond) {
                            let ins2 = ctx.instruction_mut(ins_id);
                            ins2.operation = Operation::Binary(Binary {
                                lhs: binary_op.lhs,
                                rhs: binary_op.rhs,
                                operator: binary_op.operator.clone(),
                                predicate: Some(cond),
                            });
                        }
                    }
                }
                Operation::Store { array_id, index, value, predicate, location } => {
                    if !ins.operation.is_dummy_store() {
                        if let Some(idx) = ctx.get_as_constant(*index) {
                            if (idx.to_u128() as u32) >= ctx.mem[*array_id].len {
                                let error = RuntimeErrorKind::IndexOutOfBounds {
                                    index: ctx.mem[*array_id].len,
                                    bound: idx.to_u128(),
                                };
                                DecisionTree::short_circuit(
                                    ctx, stack, ass_value, error, *location,
                                )?;
                                return Ok(false);
                            }
                        }
                        if !stack.is_new_array(ctx, array_id) && ctx.under_assumption(ass_value) {
                            let pred =
                                Self::and_conditions(Some(ass_value), *predicate, stack, ctx);
                            // we use a 0 index when the predicate is false and index is not constant, to avoid out-of-bound issues
                            let idx = if ctx.get_as_constant(*index).is_some() {
                                *index
                            } else {
                                let op = Operation::Cast(Self::unwrap_predicate(ctx, &pred));

                                let cast = ctx.add_instruction(Instruction::new(
                                    op,
                                    ctx.object_type(*index),
                                    Some(stack.block),
                                ));
                                stack.push(cast);
                                let op = Operation::Binary(node::Binary {
                                    lhs: *index,
                                    rhs: cast,
                                    operator: BinaryOp::Mul,
                                    predicate: None,
                                });
                                let idx = ctx.add_instruction(Instruction::new(
                                    op,
                                    ObjectType::native_field(),
                                    Some(stack.block),
                                ));
                                optimizations::simplify_id(ctx, idx).unwrap();
                                stack.push(idx);
                                idx
                            };
                            let ins2 = ctx.instruction_mut(ins_id);
                            ins2.operation = Operation::Store {
                                array_id: *array_id,
                                index: idx,
                                value: *value,
                                predicate: pred,
                                location: *location,
                            };
                        }
                    }
                    stack.push(ins_id);
                }
                Operation::Intrinsic(_, _) => {
                    stack.push(ins_id);
                    if ctx.under_assumption(ass_value) {
                        if let ObjectType::ArrayPointer(a) = ins.res_type {
                            if stack.created_arrays[&a] != stack.block {
                                let array = &ctx.mem[a].clone();
                                let name = array.name.to_string() + DUPLICATED;
                                ctx.new_array(&name, array.element_type, array.len, None);
                                let array_dup = ctx.mem.last_id();
                                let ins2 = ctx.instruction_mut(ins_id);
                                ins2.res_type = ObjectType::ArrayPointer(array_dup);

                                let mut memcpy_stack = StackFrame::new(stack.block);
                                ctx.memcpy_inline(
                                    ins.res_type,
                                    ObjectType::ArrayPointer(array_dup),
                                    &mut memcpy_stack,
                                );
                                self.apply_condition_to_instructions(
                                    ctx,
                                    &memcpy_stack.stack,
                                    stack,
                                    predicate,
                                )?;
                            }
                        }
                    }
                }

                Operation::Call {
                    func: func_id,
                    arguments,
                    returned_arrays,
                    predicate: ins_pred,
                    location,
                } => {
                    if ctx.under_assumption(ass_value) {
                        assert!(*ins_pred == AssumptionId::dummy());
                        let mut ins2 = ctx.instruction_mut(ins_id);
                        ins2.operation = Operation::Call {
                            func: *func_id,
                            arguments: arguments.clone(),
                            returned_arrays: returned_arrays.clone(),
                            predicate,
                            location: *location,
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
                        if ctx.is_zero(*expr) {
                            stack.stack.clear();
                        }
                        let cond = ctx.add_instruction(Instruction::new(
                            operation,
                            ObjectType::boolean(),
                            Some(stack.block),
                        ));
                        stack.push(cond);
                        let ins2 = ctx.instruction_mut(ins_id);
                        ins2.operation = Operation::Constrain(cond, *loc);
                        if ctx.is_zero(*expr) {
                            stack.push(ins_id);
                            return Ok(false);
                        }
                    }
                    stack.push(ins_id);
                }
                _ => stack.push(ins_id),
            }
        }

        Ok(true)
    }

    pub(crate) fn get_assumption_value(&self, assumption: AssumptionId) -> Option<NodeId> {
        if assumption == AssumptionId::dummy() {
            None
        } else {
            self[assumption].value
        }
    }

    // returns condition1 AND condition2
    fn and_conditions(
        condition1: Option<NodeId>,
        condition2: Option<NodeId>,
        stack_frame: &mut StackFrame,
        ctx: &mut SsaContext,
    ) -> Option<NodeId> {
        match (condition1, condition2) {
            (None, None) => None,
            (Some(cond), other) | (other, Some(cond)) if cond.is_dummy() => {
                Self::and_conditions(None, other, stack_frame, ctx)
            }
            (Some(cond), None) | (None, Some(cond)) => Some(cond),
            (Some(cond1), Some(cond2)) if cond1 == cond2 => condition1,
            (Some(cond1), Some(cond2)) => {
                let op = Operation::Binary(node::Binary {
                    lhs: cond1,
                    rhs: cond2,
                    operator: BinaryOp::Mul,
                    predicate: None,
                });
                let cond = ctx.add_instruction(Instruction::new(
                    op,
                    ObjectType::boolean(),
                    Some(stack_frame.block),
                ));
                optimizations::simplify_id(ctx, cond).unwrap();
                stack_frame.push(cond);
                Some(cond)
            }
        }
    }

    // returns condition1 OR condition2
    fn or_conditions(
        condition1: Option<NodeId>,
        condition2: Option<NodeId>,
        stack_frame: &mut StackFrame,
        ctx: &mut SsaContext,
    ) -> Option<NodeId> {
        match (condition1, condition2) {
            (_condition, None) | (None, _condition) => None,
            (Some(cond1), Some(cond2)) => {
                if cond1.is_dummy() || cond2.is_dummy() {
                    None
                } else if cond1 == cond2 {
                    condition1
                } else {
                    let op = Operation::Binary(node::Binary {
                        lhs: cond1,
                        rhs: cond2,
                        operator: BinaryOp::Or,
                        predicate: None,
                    });
                    let cond = ctx.add_instruction(Instruction::new(
                        op,
                        ObjectType::boolean(),
                        Some(stack_frame.block),
                    ));
                    optimizations::simplify_id(ctx, cond).unwrap();
                    stack_frame.push(cond);
                    Some(cond)
                }
            }
        }
    }

    pub(crate) fn unwrap_predicate(ctx: &SsaContext, predicate: &Option<NodeId>) -> NodeId {
        let predicate = predicate.unwrap_or(NodeId::dummy());
        if predicate.is_dummy() {
            ctx.one()
        } else {
            predicate
        }
    }

    fn synchronize(
        &self,
        ctx: &mut SsaContext,
        left: &[NodeId],
        right: &[NodeId],
        block_id: BlockId,
    ) -> Vec<NodeId> {
        // 1. find potential matches between the two blocks
        let mut candidates = Vec::new();
        let keep_call_and_store = |node_id: NodeId| -> bool {
            let ins = ctx.instruction(node_id);
            matches!(ins.operation.opcode(), Opcode::Call(_) | Opcode::Store(_))
        };
        let l_iter = left.iter().enumerate().filter(|&i| keep_call_and_store(*i.1));
        let mut r_iter = right.iter().enumerate().filter(|&i| keep_call_and_store(*i.1));
        for left_node in l_iter {
            let left_ins = ctx.instruction(*left_node.1);
            for right_node in r_iter.by_ref() {
                let right_ins = ctx.instruction(*right_node.1);
                match (&left_ins.operation, &right_ins.operation) {
                    (
                        Operation::Call { func: left_func, returned_arrays: left_arrays, .. },
                        Operation::Call { func: right_func, returned_arrays: right_arrays, .. },
                    ) if left_func == right_func
                        && left_arrays.is_empty()
                        && right_arrays.is_empty() =>
                    {
                        candidates.push(Segment::new(left_node, right_node));
                    }

                    (
                        Operation::Store { array_id: left_array, index: left_index, .. },
                        Operation::Store { array_id: right_array, index: right_index, .. },
                    ) if left_array == right_array && left_index == right_index => {
                        candidates.push(Segment::new(left_node, right_node));
                    }
                    _ => (),
                }
            }
        }
        // 2. construct a solution
        let mut solution = Vec::new();
        // TODO: far from optimal greedy solution...
        for i in &candidates {
            if intersect(&solution, i).is_none() {
                solution.push(Segment { left: i.left, right: i.right });
            }
        }

        // 3. Merge the blocks using the solution
        let mut left_pos = 0;
        let mut right_pos = 0;
        let mut stack_frame = StackFrame::new(block_id);
        for i in solution {
            stack_frame.stack.extend_from_slice(&left[left_pos..i.left.0]);
            left_pos = i.left.0;
            stack_frame.stack.extend_from_slice(&right[right_pos..i.right.0]);
            right_pos = i.right.0;
            //merge i:
            let left_ins = ctx.instruction(left[left_pos]).clone();
            let right_ins = ctx.instruction(right[right_pos]).clone();
            let assumption = &self[ctx[block_id].assumption];

            let merged_op = match (&left_ins.operation, &right_ins.operation) {
                (
                    Operation::Call {
                        func: left_func,
                        arguments: left_arg,
                        returned_arrays: left_arrays,
                        location: left_location,
                        ..
                    },
                    Operation::Call { func: right_func, arguments: right_arg, .. },
                ) => {
                    debug_assert_eq!(left_func, right_func);
                    let mut args = Vec::new();
                    for a in left_arg.iter().enumerate() {
                        let op = Operation::Cond {
                            condition: self[assumption.parent].condition,
                            val_true: *a.1,
                            val_false: right_arg[a.0],
                        };
                        let typ = ctx.object_type(*a.1);
                        let arg_id = ctx.add_instruction(Instruction::new(op, typ, Some(block_id)));
                        stack_frame.stack.push(arg_id);
                        args.push(arg_id);
                    }
                    Operation::Call {
                        func: *left_func,
                        arguments: args,
                        returned_arrays: left_arrays.clone(),
                        predicate: self.root,
                        location: *left_location,
                    }
                }
                (
                    Operation::Store {
                        array_id: left_array,
                        index: left_index,
                        value: left_val,
                        predicate: left_pred,
                        location: left_loc,
                    },
                    Operation::Store {
                        value: right_val,
                        predicate: right_pred,
                        location: right_loc,
                        ..
                    },
                ) => {
                    let pred = Self::or_conditions(*left_pred, *right_pred, &mut stack_frame, ctx);
                    let op = Operation::Cond {
                        condition: self[assumption.parent].condition,
                        val_true: *left_val,
                        val_false: *right_val,
                    };
                    let merge =
                        Instruction::new(op, ctx.mem[*left_array].element_type, Some(block_id));
                    let merge_id = ctx.add_instruction(merge);
                    let location = RuntimeError::merge_location(*left_loc, *right_loc);
                    stack_frame.stack.push(merge_id);
                    Operation::Store {
                        array_id: *left_array,
                        index: *left_index,
                        value: merge_id,
                        predicate: pred,
                        location,
                    }
                }
                _ => unreachable!(),
            };
            if let Opcode::Call(_) = merged_op.opcode() {
                let left_ins = ctx.instruction_mut(left[left_pos]);
                left_ins.mark = node::Mark::ReplaceWith(right[right_pos]);
            }
            let ins1 = ctx.instruction_mut(right[right_pos]);
            ins1.operation = merged_op;
            stack_frame.stack.push(ins1.id);
            left_pos += 1;
            right_pos += 1;
        }
        stack_frame.stack.extend_from_slice(&left[left_pos..left.len()]);
        stack_frame.stack.extend_from_slice(&right[right_pos..right.len()]);
        stack_frame.stack
    }
}

//unroll an if sub-graph
pub(super) fn unroll_if(
    ctx: &mut SsaContext,
    unroll_ctx: &mut UnrollContext,
) -> Result<BlockId, RuntimeError> {
    //1. find the join block
    let if_block = &ctx[unroll_ctx.to_unroll];
    let left = if_block.left.unwrap();
    let right = if_block.right.unwrap();
    assert!(if_block.kind == BlockType::Normal);
    let exit = block::find_join(ctx, if_block.id);

    //2. create the IF subgraph
    let (new_entry, new_exit) =
        if unroll_ctx.unroll_into.is_dummy() || unroll_ctx.unroll_into == unroll_ctx.to_unroll {
            // simple mode:
            create_if_subgraph(ctx, unroll_ctx.to_unroll, true)
        } else {
            //the unroll_into is required and will be used as the prev block
            let prev = unroll_ctx.unroll_into;
            create_if_subgraph(ctx, prev, false)
        };
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
fn create_if_subgraph(
    ctx: &mut SsaContext,
    prev_block: BlockId,
    simple_mode: bool,
) -> (BlockId, BlockId) {
    //Entry block
    ctx.current_block = prev_block;
    let new_entry = if simple_mode {
        prev_block
    } else {
        block::new_sealed_block(ctx, block::BlockType::Normal, true)
    };
    //Then block
    ctx.current_block = new_entry;
    let new_then = block::new_sealed_block(ctx, block::BlockType::Normal, true);
    //Else block
    ctx.current_block = new_entry;
    let new_else = block::new_sealed_block(ctx, block::BlockType::Normal, false);
    //Exit block
    let new_exit = block::new_sealed_block(ctx, block::BlockType::IfJoin, false);
    ctx[new_exit].dominator = Some(new_entry);
    ctx[new_entry].right = Some(new_else);
    ctx[new_exit].predecessor.push(new_then);

    (new_entry, new_exit)
}

#[derive(Debug)]
struct Segment {
    left: (usize, NodeId),
    right: (usize, NodeId),
}

impl Segment {
    fn new(left_node: (usize, &NodeId), right_node: (usize, &NodeId)) -> Segment {
        Segment { left: (left_node.0, *left_node.1), right: (right_node.0, *right_node.1) }
    }

    fn intersect(&self, other: &Segment) -> bool {
        !((self.right.0 < other.right.0 && self.left.0 < other.left.0)
            || (self.right.0 > other.right.0 && self.left.0 > other.left.0))
    }
}

fn intersect(solution: &[Segment], candidate: &Segment) -> Option<usize> {
    for i in solution.iter().enumerate() {
        if i.1.intersect(candidate) {
            return Some(i.0);
        }
    }
    None
}
