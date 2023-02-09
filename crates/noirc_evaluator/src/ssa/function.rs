use crate::errors::RuntimeError;
use crate::ssa::{
    block::BlockId,
    code_gen::{resize_graph, IRGenerator},
    conditional::{DecisionTree, TreeBuilder},
    context::SsaContext,
    node::{NodeId, ObjectType},
    {block, node, ssa_form},
};
use noirc_frontend::monomorphization::ast::FuncId;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct FuncIndex(pub usize);

impl FuncIndex {
    pub fn new(idx: usize) -> FuncIndex {
        FuncIndex(idx)
    }
}

#[derive(Clone, Debug)]
pub struct SSAFunction {
    pub entry_block: BlockId,
    pub id: FuncId,
    pub idx: FuncIndex,
    pub node_id: NodeId,

    //signature:
    pub name: String,
    pub arguments: Vec<(NodeId, bool)>,
    pub result_types: Vec<ObjectType>,
    pub decision: DecisionTree,
}

impl SSAFunction {
    pub fn new(
        id: FuncId,
        name: &str,
        block_id: BlockId,
        idx: FuncIndex,
        ctx: &mut SsaContext,
    ) -> SSAFunction {
        SSAFunction {
            entry_block: block_id,
            id,
            node_id: ctx.push_function_id(id, name),
            name: name.to_string(),
            arguments: Vec::new(),
            result_types: Vec::new(),
            decision: DecisionTree::new(ctx),
            idx,
        }
    }

    pub fn compile(&self, ir_gen: &mut IRGenerator) -> Result<DecisionTree, RuntimeError> {
        let function_cfg = block::bfs(self.entry_block, None, &ir_gen.context);
        block::compute_sub_dom(&mut ir_gen.context, &function_cfg);
        //Optimization
        //catch the error because the function may not be called
        super::optimizations::full_cse(&mut ir_gen.context, self.entry_block, false)?;
        //Unrolling
        super::flatten::unroll_tree(&mut ir_gen.context, self.entry_block)?;

        //reduce conditionals
        let mut decision = DecisionTree::new(&ir_gen.context);
        let mut builder = TreeBuilder::new(self.entry_block);
        for (arg, _) in &self.arguments {
            if let ObjectType::Pointer(a) = ir_gen.context.get_object_type(*arg) {
                builder.stack.created_arrays.insert(a, self.entry_block);
            }
        }

        let mut to_remove: VecDeque<BlockId> = VecDeque::new();

        let result = decision.make_decision_tree(&mut ir_gen.context, builder);
        if result.is_err() {
            // we take the last block to ensure we have the return instruction
            let exit = block::exit(&ir_gen.context, self.entry_block);
            //short-circuit for function: false constraint and return 0
            let instructions = &ir_gen.context[exit].instructions.clone();
            let stack = block::short_circuit_instructions(
                &mut ir_gen.context,
                self.entry_block,
                instructions,
            );
            if self.entry_block != exit {
                for i in &stack {
                    ir_gen.context.get_mut_instruction(*i).parent_block = self.entry_block;
                }
            }

            let function_block = &mut ir_gen.context[self.entry_block];
            function_block.instructions.clear();
            function_block.instructions = stack;
            function_block.left = None;
            to_remove.extend(function_cfg.iter()); //let's remove all the other blocks
        } else {
            decision.reduce(&mut ir_gen.context, decision.root)?;
        }

        //merge blocks
        to_remove =
            block::merge_path(&mut ir_gen.context, self.entry_block, BlockId::dummy(), None)?;

        ir_gen.context[self.entry_block].dominated.retain(|b| !to_remove.contains(b));
        for i in to_remove {
            if i != self.entry_block {
                ir_gen.context.remove_block(i);
            }
        }
        Ok(decision)
    }

    pub fn get_mapped_value(
        var: Option<&NodeId>,
        ctx: &mut SsaContext,
        inline_map: &HashMap<NodeId, NodeId>,
        block_id: BlockId,
    ) -> NodeId {
        let dummy = NodeId::dummy();
        let node_id = var.unwrap_or(&dummy);
        if node_id == &dummy {
            return dummy;
        }

        let node_obj_opt = ctx.try_get_node(*node_id);
        if let Some(node::NodeObject::Const(c)) = node_obj_opt {
            ctx.get_or_create_const(c.get_value_field(), c.value_type)
        } else if let Some(id) = inline_map.get(node_id) {
            *id
        } else {
            ssa_form::get_current_value_in_block(ctx, *node_id, block_id)
        }
    }
}

fn is_leaf(call_graph: &[Vec<u8>], i: FuncIndex) -> bool {
    for j in 0..call_graph[i.0].len() {
        if call_graph[i.0][j] == 1 {
            return false;
        }
    }
    true
}

fn get_new_leaf(ctx: &SsaContext, processed: &[FuncIndex]) -> (FuncIndex, FuncId) {
    for f in ctx.functions.values() {
        if !processed.contains(&(f.idx)) && is_leaf(&ctx.call_graph, f.idx) {
            return (f.idx, f.id);
        }
    }
    unimplemented!("Recursive function call is not supported");
}

//inline all functions of the call graph such that every inlining operates with a fully flattened function
pub fn inline_all(ctx: &mut SsaContext) -> Result<(), RuntimeError> {
    resize_graph(&mut ctx.call_graph, ctx.functions.len());
    let l = ctx.call_graph.len();
    let mut processed = Vec::new();
    while processed.len() < l {
        let i = get_new_leaf(ctx, &processed);
        if !processed.is_empty() {
            super::optimizations::full_cse(ctx, ctx.functions[&i.1].entry_block, false)?;
        }
        let mut to_inline = Vec::new();
        for f in ctx.functions.values() {
            if ctx.call_graph[f.idx.0][i.0 .0] == 1 {
                to_inline.push((f.id, f.idx));
            }
        }
        for (func_id, func_idx) in to_inline {
            super::inline::inline_cfg(ctx, func_id, Some(i.1))?;
            ctx.call_graph[func_idx.0][i.0 .0] = 0;
        }
        processed.push(i.0);
    }
    ctx.call_graph.clear();
    Ok(())
}
