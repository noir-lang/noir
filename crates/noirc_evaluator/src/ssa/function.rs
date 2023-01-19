use std::collections::{HashMap, VecDeque};

use crate::errors::RuntimeError;
use crate::ssa::node::Opcode;
use iter_extended::try_vecmap;
use noirc_frontend::monomorphisation::ast::{Call, Definition, FuncId, LocalId, Type};

use super::builtin;
use super::conditional::{AssumptionId, DecisionTree, TreeBuilder};
use super::mem::ArrayId;
use super::node::{Node, Operation};
use super::{
    block,
    block::BlockId,
    code_gen::IRGenerator,
    context::SsaContext,
    node::{self, NodeId, ObjectType},
    ssa_form,
};

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

    pub fn compile(&self, igen: &mut IRGenerator) -> Result<DecisionTree, RuntimeError> {
        let function_cfg = block::bfs(self.entry_block, None, &igen.context);
        block::compute_sub_dom(&mut igen.context, &function_cfg);
        //Optimisation
        //catch the error because the function may not be called
        super::optim::full_cse(&mut igen.context, self.entry_block, false)?;
        //Unrolling
        super::flatten::unroll_tree(&mut igen.context, self.entry_block)?;

        //reduce conditionals
        let mut decision = DecisionTree::new(&igen.context);
        let mut builder = TreeBuilder::new(self.entry_block);
        for (arg, _) in &self.arguments {
            if let ObjectType::Pointer(a) = igen.context.get_object_type(*arg) {
                builder.stack.created_arrays.insert(a, self.entry_block);
            }
        }

        let mut to_remove: VecDeque<BlockId> = VecDeque::new();

        let result = decision.make_decision_tree(&mut igen.context, builder);
        if result.is_err() {
            // we take the last block to ensure we have the return instruction
            let exit = block::exit(&igen.context, self.entry_block);
            //short-circuit for function: false constraint and return 0
            let instructions = &igen.context[exit].instructions.clone();
            let stack = block::short_circuit_instructions(
                &mut igen.context,
                self.entry_block,
                instructions,
            );
            if self.entry_block != exit {
                for i in &stack {
                    igen.context.get_mut_instruction(*i).parent_block = self.entry_block;
                }
            }

            let function_block = &mut igen.context[self.entry_block];
            function_block.instructions.clear();
            function_block.instructions = stack;
            function_block.left = None;
            to_remove.extend(function_cfg.iter()); //let's remove all the other blocks
        } else {
            decision.reduce(&mut igen.context, decision.root)?;
        }

        //merge blocks
        to_remove = block::merge_path(&mut igen.context, self.entry_block, BlockId::dummy(), None)?;

        igen.context[self.entry_block].dominated.retain(|b| !to_remove.contains(b));
        for i in to_remove {
            if i != self.entry_block {
                igen.context.remove_block(i);
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
        if let Some(node::NodeObj::Const(c)) = node_obj_opt {
            ctx.get_or_create_const(c.get_value_field(), c.value_type)
        } else if let Some(id) = inline_map.get(node_id) {
            *id
        } else {
            ssa_form::get_current_value_in_block(ctx, *node_id, block_id)
        }
    }
}

impl IRGenerator {
    /// Creates an ssa function and returns its type upon success
    pub fn create_function(
        &mut self,
        func_id: FuncId,
        index: FuncIndex,
    ) -> Result<ObjectType, RuntimeError> {
        let current_block = self.context.current_block;
        let current_function = self.function_context;
        let func_block = block::BasicBlock::create_cfg(&mut self.context);

        let function = &mut self.program[func_id];
        let mut func =
            SSAFunction::new(func_id, &function.name, func_block, index, &mut self.context);

        //arguments:
        for (param_id, mutable, name, typ) in std::mem::take(&mut function.parameters) {
            let node_ids = self.create_function_parameter(param_id, &typ, &name);
            func.arguments.extend(node_ids.into_iter().map(|id| (id, mutable)));
        }

        // ensure return types are defined in case of recursion call cycle
        let function = &mut self.program[func_id];
        let return_types = function.return_type.flatten();
        for typ in return_types {
            func.result_types.push(match typ {
                Type::Unit => ObjectType::NotAnObject,
                Type::Array(_, _) => ObjectType::Pointer(crate::ssa::mem::ArrayId::dummy()),
                _ => self.context.convert_type(&typ),
            });
        }

        self.function_context = Some(index);
        self.context.functions.insert(func_id, func.clone());

        let function_body = self.program.take_function_body(func_id);
        let last_value = self.codegen_expression(&function_body)?;
        let return_values = last_value.to_node_ids();

        func.result_types.clear();
        let return_values = try_vecmap(return_values, |mut return_id| {
            let node_opt = self.context.try_get_node(return_id);
            let typ = node_opt.map_or(ObjectType::NotAnObject, |node| node.get_type());

            if let Some(ins) = self.context.try_get_instruction(return_id) {
                if ins.operation.opcode() == Opcode::Results {
                    // n.b. this required for result instructions, but won't hurt if done for all i
                    let new_var = node::Variable {
                        id: NodeId::dummy(),
                        obj_type: typ,
                        name: format!("return_{}", return_id.0.into_raw_parts().0),
                        root: None,
                        def: None,
                        witness: None,
                        parent_block: self.context.current_block,
                    };
                    let b_id = self.context.add_variable(new_var, None);
                    let b_id1 = self.context.handle_assign(b_id, None, return_id)?;
                    return_id = ssa_form::get_current_value(&mut self.context, b_id1);
                }
            }
            func.result_types.push(typ);
            Ok::<NodeId, RuntimeError>(return_id)
        })?;

        self.context.new_instruction(
            node::Operation::Return(return_values),
            node::ObjectType::NotAnObject,
        )?;
        let decision = func.compile(self)?; //unroll the function
        func.decision = decision;
        self.context.functions.insert(func_id, func);
        self.context.current_block = current_block;
        self.function_context = current_function;

        Ok(ObjectType::Function)
    }

    fn create_function_parameter(&mut self, id: LocalId, typ: &Type, name: &str) -> Vec<NodeId> {
        //check if the variable is already created:
        let def = Definition::Local(id);
        let val = match self.find_variable(&def) {
            Some(var) => self.get_current_value(&var.clone()),
            None => self.create_new_value(typ, name, Some(def)),
        };
        val.to_node_ids()
    }

    //generates an instruction for calling the function
    pub fn call(&mut self, call: &Call) -> Result<Vec<NodeId>, RuntimeError> {
        let func = self.codegen_expression(&call.func)?.unwrap_id();
        let arguments = self.codegen_expression_list(&call.arguments);

        if let Some(opcode) = self.context.get_builtin_opcode(func) {
            return self.call_low_level(opcode, arguments);
        }

        let predicate = AssumptionId::dummy();
        let location = call.location;

        let mut call_op =
            Operation::Call { func, arguments, returned_arrays: vec![], predicate, location };

        let call_instruction =
            self.context.new_instruction(call_op.clone(), ObjectType::NotAnObject)?;

        if let Some(id) = self.context.try_get_funcid(func) {
            let callee = self.context.get_ssafunc(id).unwrap().idx;
            if let Some(caller) = self.function_context {
                update_call_graph(&mut self.context.call_graph, caller, callee);
            }
        }

        // Temporary: this block is needed to fix a bug in 7_function
        // where `foo` is incorrectly inferred to take an array of size 1 and
        // return an array of size 0.
        // we should check issue #628 again when this block is removed
        if let Some(func_id) = self.context.try_get_funcid(func) {
            let rtt = self.context.functions[&func_id].result_types.clone();
            let mut result = Vec::new();
            for i in rtt.iter().enumerate() {
                result.push(self.context.new_instruction(
                    node::Operation::Result { call_instruction, index: i.0 as u32 },
                    *i.1,
                )?);
            }
            let ssa_func = self.context.get_ssafunc(id).unwrap();
            let func_arguments = ssa_func.arguments.clone();
            for (caller_arg, func_arg) in arguments.iter().zip(func_arguments) {
                let mut is_array_result = false;
                if let Some(node::Instruction {
                    operation: node::Operation::Result { .. }, ..
                }) = self.context.try_get_instruction(*caller_arg)
                {
                    is_array_result =
                        super::mem::Memory::deref(&self.context, func_arg.0).is_some();
                }
                if is_array_result {
                    self.context.handle_assign(func_arg.0, None, *caller_arg)?;
                }
            }

            // If we have the function directly the ArrayIds in the Result types are correct
            // and we don't need to set returned_arrays yet as they can be set later.
            return Ok(result);
        }

        let returned_arrays = match &mut call_op {
            Operation::Call { returned_arrays, .. } => returned_arrays,
            _ => unreachable!(),
        };

        let result_ids = self.create_call_results(call, call_instruction, returned_arrays);

        // Fixup the returned_arrays, they will be incorrectly tracked for higher order functions
        // otherwise.
        self.context.get_mut_instruction(call_instruction).operation = call_op;
        result_ids
    }

    fn create_call_results(
        &mut self,
        call: &Call,
        call_instruction: NodeId,
        returned_arrays: &mut Vec<(ArrayId, u32)>,
    ) -> Result<Vec<NodeId>, RuntimeError> {
        let return_types = call.return_type.flatten().into_iter().enumerate();

        try_vecmap(return_types, |(i, typ)| {
            let result = Operation::Result { call_instruction, index: i as u32 };
            let typ = match typ {
                Type::Array(len, elem_type) => {
                    let elem_type = self.context.convert_type(&elem_type);
                    let array_id = self.context.new_array("", elem_type, len as u32, None).1;
                    returned_arrays.push((array_id, i as u32));
                    ObjectType::Pointer(array_id)
                }
                other => self.context.convert_type(&other),
            };

            self.context.new_instruction(result, typ)
        })
    }

    //Lowlevel functions with no more than 2 arguments
    pub fn call_low_level(
        &mut self,
        op: builtin::Opcode,
        args: Vec<NodeId>,
    ) -> Result<Vec<NodeId>, RuntimeError> {
        let (len, elem_type) = op.get_result_type();

        let result_type = if len > 1 {
            //We create an array that will contain the result and set the res_type to point to that array
            let result_index = self.new_array(&format!("{op}_result"), elem_type, len, None).1;
            node::ObjectType::Pointer(result_index)
        } else {
            elem_type
        };

        //when the function returns an array, we use ins.res_type(array)
        //else we map ins.id to the returned witness
        let id = self.context.new_instruction(node::Operation::Intrinsic(op, args), result_type)?;
        Ok(vec![id])
    }
}

pub fn resize_graph(call_graph: &mut Vec<Vec<u8>>, size: usize) {
    while call_graph.len() < size {
        call_graph.push(vec![0; size]);
    }

    for i in call_graph.iter_mut() {
        while i.len() < size {
            i.push(0);
        }
    }
}

fn update_call_graph(call_graph: &mut Vec<Vec<u8>>, caller: FuncIndex, callee: FuncIndex) {
    let a = caller.0;
    let b = callee.0;
    let max = a.max(b) + 1;
    resize_graph(call_graph, max);

    call_graph[a][b] = 1;
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
            super::optim::full_cse(ctx, ctx.functions[&i.1].entry_block, false)?;
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
