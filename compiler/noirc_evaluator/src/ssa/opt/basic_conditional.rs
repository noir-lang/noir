use std::collections::HashSet;

use acvm::AcirField;
use fxhash::FxHashMap;
use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::DataFlowGraph,
        function::{Function, FunctionId, RuntimeType},
        instruction::{BinaryOp, Instruction, TerminatorInstruction},
    },
    Ssa,
};

use super::flatten_cfg::Context;
#[derive(Debug, Clone)]
struct BasicConditional {
    block_entry: BasicBlockId,
    block_then: Option<BasicBlockId>,
    block_else: Option<BasicBlockId>,
    block_exit: BasicBlockId,
}

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn flatten_basic_conditionals(mut self) -> Ssa {
        // Retrieve the 'no_predicates' attribute of the functions in a map, to avoid problems with borrowing
        let mut no_predicates = FxHashMap::default();
        for function in self.functions.values() {
            no_predicates.insert(function.id(), function.is_no_predicates());
        }
        for function in self.functions.values_mut() {
            flatten_function(function, &mut no_predicates);
        }
        self
    }
}

// Returns the blocks of the simple conditional sub-graph whose input block is the entry.
// Returns None if the input block is not the entry block of a simple conditional.
fn is_conditional(
    block: BasicBlockId,
    cfg: &ControlFlowGraph,
    function: &Function,
) -> Option<BasicConditional> {
    // jump overhead is the cost for doing the conditional and jump around the blocks
    // I use 10 as a rough estimate, real cost is less.
    let jump_overhead = 10;
    let mut successors = cfg.successors(block);
    let mut result = None;
    // a conditional must have 2 branches
    if successors.len() == 2 {
        let left = successors.next().unwrap();
        let right = successors.next().unwrap();
        let mut left_successors = cfg.successors(left);
        let mut right_successors = cfg.successors(right);
        let left_successors_len = left_successors.len();
        let right_successors_len = right_successors.len();
        let next_left = left_successors.next();
        let next_right = right_successors.next();
        if next_left == Some(block) || next_right == Some(block) {
            // this is a loop, not a conditional
            return None;
        }
        if left_successors_len == 1 && right_successors_len == 1 && next_left == next_right {
            // The branches join on one block so it is a non-nested conditional
            let cost_left = block_cost(left, &function.dfg);
            let cost_right = block_cost(right, &function.dfg);
            // For the flattening to be valuable, we compare the cost of the flattened code with the average cost of the 2 branches,
            // including an overhead to take into account the jumps between the blocks.
            let cost = cost_right.saturating_add(cost_left);
            if cost < cost / 2 + jump_overhead {
                if let Some(TerminatorInstruction::JmpIf {
                    condition: _,
                    then_destination,
                    else_destination,
                    call_stack: _,
                }) = function.dfg[block].terminator()
                {
                    result = Some(BasicConditional {
                        block_entry: block,
                        block_then: Some(*then_destination),
                        block_else: Some(*else_destination),
                        block_exit: next_left.unwrap(),
                    });
                }
            }
        } else if left_successors_len == 1 && next_left == Some(right) {
            // Left branch joins the right branch, it is a if/then statement with no else
            // I am not sure whether this case can happen, but it is not difficult to handle it
            let cost = block_cost(left, &function.dfg);
            if cost < cost / 2 + jump_overhead {
                if let Some(TerminatorInstruction::JmpIf {
                    condition: _,
                    then_destination,
                    else_destination,
                    call_stack: _,
                }) = function.dfg[block].terminator()
                {
                    if left == *then_destination {
                        result = Some(BasicConditional {
                            block_entry: block,
                            block_then: Some(left),
                            block_else: None,
                            block_exit: right,
                        });
                    } else if left == *else_destination {
                        result = Some(BasicConditional {
                            block_entry: block,
                            block_then: None,
                            block_else: Some(left),
                            block_exit: right,
                        });
                    }
                }
            }
        } else if right_successors_len == 1 && next_right == Some(left) {
            // Right branch joins the right branch, it is a if/else statement with no then
            // I am not sure whether this case can happen, but it is not difficult to handle it
            let cost = block_cost(right, &function.dfg);
            if cost < cost / 2 + jump_overhead {
                if let Some(TerminatorInstruction::JmpIf {
                    condition: _,
                    then_destination,
                    else_destination,
                    call_stack: _,
                }) = function.dfg[block].terminator()
                {
                    if right == *else_destination {
                        result = Some(BasicConditional {
                            block_entry: block,
                            block_then: None,
                            block_else: Some(right),
                            block_exit: right,
                        });
                    } else if right == *then_destination {
                        result = Some(BasicConditional {
                            block_entry: block,
                            block_then: Some(right),
                            block_else: None,
                            block_exit: right,
                        });
                    }
                }
            }
        }
    }
    // A conditional exit would have exactly 2 predecessors
    result.filter(|result| cfg.predecessors(result.block_exit).len() == 2)
}

/// Computes a cost estimate of a basic block
/// returns u32::MAX if the block has side-effect instructions
fn block_cost(block: BasicBlockId, dfg: &DataFlowGraph) -> u32 {
    let mut cost: u32 = 0;

    for instruction in dfg[block].instructions() {
        let instruction_cost = match &dfg[*instruction] {
            Instruction::Binary(binary) => {
                match binary.operator {
                    BinaryOp::Add { unchecked }
                    | BinaryOp::Sub { unchecked }
                    | BinaryOp::Mul { unchecked } => if unchecked { 1 } else { return u32::MAX },
                    BinaryOp::Div
                    | BinaryOp::Mod => return u32::MAX,
                    BinaryOp::Eq => 2,
                    BinaryOp::Lt => 5,
                    BinaryOp::And
                    | BinaryOp::Or
                    | BinaryOp::Xor => 1,   //todo
                    BinaryOp::Shl
                    | BinaryOp::Shr => return u32::MAX,
                }
            },
            Instruction::Cast(_, _) => 1,//TODO check if this instruction can fail
            Instruction::Not(_) => 1,
            Instruction::Truncate { .. } => 7,

            Instruction::Constrain(_,_,_)
            | Instruction::ConstrainNotEqual(_,_,_)
            | Instruction::RangeCheck { .. }
            | Instruction::Call { .. }  //TODO support calls with no-predicate set to true
            |      Instruction::Load { .. }
            | Instruction::Store { .. }
            | Instruction::ArraySet { .. } => {return u32::MAX},

            Instruction::ArrayGet { array, index  } => {
                // check if index is in bound
                let index = dfg.get_numeric_constant(*index);
                let mut ok_bound = false;
                if let Some(index) = index {
                    let len = dfg.try_get_array_length(*array).unwrap();
                    if index.to_u128() < len as u128 {
                        ok_bound = true;
                    }
                }
                if ok_bound {
                    1
                } else {
                    return u32::MAX;
                }
            },

            Instruction::Allocate => 0,
            Instruction::EnableSideEffectsIf { .. } => 0,
            Instruction::IncrementRc { .. } => 0,
            Instruction::DecrementRc { .. } => 0,
            Instruction::IfElse { .. } => 1,
            Instruction::MakeArray { .. } => 0,
            Instruction::Noop => 0,
        };
        cost += instruction_cost;
    }
    cost
}

/// Identifies all simple conditionals in the function and flattens them
fn flatten_function(function: &mut Function, no_predicates: &mut FxHashMap<FunctionId, bool>) {
    // This pass is dedicated to brillig functions
    if !matches!(function.runtime(), RuntimeType::Brillig(_)) {
        return;
    }
    let cfg = ControlFlowGraph::with_function(function);
    let mut stack = vec![function.entry_block()];
    let mut processed = HashSet::new();
    let mut conditionals = Vec::new();

    // 1. Process all blocks of the cfg, starting from the root and following the successors
    while let Some(block) = stack.pop() {
        // Avoid cycles
        if processed.contains(&block) {
            continue;
        }
        processed.insert(block);

        // Identify the simple conditionals
        if let Some(conditional) = is_conditional(block, &cfg, function) {
            // no need to check the branches, process the join block directly
            stack.push(conditional.block_exit);
            conditionals.push(conditional);
        } else {
            stack.extend(cfg.successors(block));
        }
    }

    // 2. Flatten all simple conditionals
    // process in reverse order because the process of a conditional does not impact the previous ones
    conditionals.reverse();
    for simple_if in conditionals {
        Context::flatten_single(&simple_if, function, no_predicates);
    }
}

impl<'f> Context<'f> {
    fn flatten_single(
        conditional: &BasicConditional,
        function: &mut Function,
        no_predicates: &mut FxHashMap<FunctionId, bool>,
    ) {
        let mut branch_ends = FxHashMap::default();
        branch_ends.insert(conditional.block_entry, conditional.block_exit);
        let cfg = ControlFlowGraph::with_function(function);
        let mut context = Context::new(function, cfg, branch_ends, conditional.block_entry);
        context.no_predicate = true;

        context.flatten_single_conditional(conditional, no_predicates);
    }

    fn flatten_single_conditional(
        &mut self,
        conditional: &BasicConditional,
        no_predicates: &mut FxHashMap<FunctionId, bool>,
    ) {
        // Manually inline 'then', 'else' and 'exit' into the entry block
        let mut queue = vec![];
        //1. process 'then' branch
        self.inline_block(conditional.block_entry, no_predicates);
        let to_process = self.handle_terminator(conditional.block_entry, &queue);
        queue.extend(to_process);
        if let Some(then) = conditional.block_then {
            assert_eq!(queue.pop(), conditional.block_then);
            self.inline_block(then, no_predicates);
            let to_process = self.handle_terminator(then, &queue);

            for incoming_block in to_process {
                if !queue.contains(&incoming_block) {
                    queue.push(incoming_block);
                }
            }
        }

        //2. process 'else' branch, in case there is no 'then'
        let next = queue.pop();
        if next == conditional.block_else {
            let next = next.unwrap();
            self.inline_block(next, no_predicates);
            let to_process = self.handle_terminator(next, &queue);
            for incoming_block in to_process {
                if !queue.contains(&incoming_block) {
                    queue.push(incoming_block);
                }
            }
        } else {
            assert_eq!(next, Some(conditional.block_exit));
        }

        //3. process 'exit' block
        self.inline_block(conditional.block_exit, no_predicates);
        // Manually set the terminator of the entry block to the one of the exit block
        let terminator =
            self.inserter.function.dfg[conditional.block_exit].terminator().unwrap().clone();
        let new_terminator = match terminator {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => {
                let condition = self.inserter.resolve(condition);
                TerminatorInstruction::JmpIf {
                    condition,
                    then_destination,
                    else_destination,
                    call_stack,
                }
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack } => {
                let arguments = vecmap(arguments, |value| self.inserter.resolve(value));
                TerminatorInstruction::Jmp { destination, arguments, call_stack }
            }
            TerminatorInstruction::Return { return_values, call_stack } => {
                let return_values = vecmap(return_values, |value| self.inserter.resolve(value));
                TerminatorInstruction::Return { return_values, call_stack }
            }
        };
        self.inserter.function.dfg.set_block_terminator(conditional.block_entry, new_terminator);

        self.inserter.map_data_bus_in_place();
    }
}
