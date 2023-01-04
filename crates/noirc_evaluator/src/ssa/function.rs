use std::collections::{HashMap, VecDeque};

use crate::errors::RuntimeError;
use crate::ssa::node::Opcode;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use noirc_frontend::monomorphisation::ast::{self, Call, DefinitionId, FuncId, Type};

use super::conditional::{AssumptionId, DecisionTree, TreeBuilder};
use super::node::Node;
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
    //signature:
    pub name: String,
    pub arguments: Vec<(NodeId, bool)>,
    pub result_types: Vec<ObjectType>,
    pub decision: DecisionTree,
}

impl SSAFunction {
    pub fn new(
        func: FuncId,
        name: &str,
        block_id: BlockId,
        idx: FuncIndex,
        ctx: &SsaContext,
    ) -> SSAFunction {
        SSAFunction {
            entry_block: block_id,
            id: func,
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
        to_remove = block::merge_path(&mut igen.context, self.entry_block, BlockId::dummy(), None);

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
        if let Some(&node_id) = var {
            if node_id == NodeId::dummy() {
                return node_id;
            }
            let mut my_const = None;
            let node_obj_opt = ctx.try_get_node(node_id);
            if let Some(node::NodeObj::Const(c)) = node_obj_opt {
                my_const = Some((c.get_value_field(), c.value_type));
            }
            if let Some(c) = my_const {
                ctx.get_or_create_const(c.0, c.1)
            } else if let Some(id) = inline_map.get(&node_id) {
                *id
            } else {
                ssa_form::get_current_value_in_block(ctx, node_id, block_id)
            }
        } else {
            NodeId::dummy()
        }
    }
}

//Returns the number of elements and their type, of the output result corresponding to the OPCODE function.
pub fn get_result_type(op: OPCODE) -> (u32, ObjectType) {
    match op {
        OPCODE::AES => (0, ObjectType::NotAnObject), //Not implemented
        OPCODE::SHA256 => (32, ObjectType::Unsigned(8)),
        OPCODE::Blake2s => (32, ObjectType::Unsigned(8)),
        OPCODE::HashToField => (1, ObjectType::NativeField),
        OPCODE::MerkleMembership => (1, ObjectType::NativeField), //or bool?
        OPCODE::SchnorrVerify => (1, ObjectType::NativeField),    //or bool?
        OPCODE::Pedersen => (2, ObjectType::NativeField),
        OPCODE::EcdsaSecp256k1 => (1, ObjectType::NativeField), //field?
        OPCODE::FixedBaseScalarMul => (2, ObjectType::NativeField),
        OPCODE::ToBits => (FieldElement::max_num_bits(), ObjectType::Boolean),
        OPCODE::ToBytes => (FieldElement::max_num_bytes(), ObjectType::Boolean),
    }
}

impl IRGenerator {
    pub fn create_function(
        &mut self,
        func_id: FuncId,
        index: FuncIndex,
    ) -> Result<(), RuntimeError> {
        let current_block = self.context.current_block;
        let current_function = self.function_context;
        let func_block = block::BasicBlock::create_cfg(&mut self.context);

        let function = &mut self.program[func_id];
        let mut func = SSAFunction::new(func_id, &function.name, func_block, index, &self.context);

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
                _ => typ.into(),
            });
        }

        self.function_context = Some(index);
        self.context.functions.insert(func_id, func.clone());

        let function_body = self.program.take_function_body(func_id);
        let last_value = self.codegen_expression(&function_body)?;
        let last_values = last_value.to_node_ids();

        func.result_types.clear();
        let mut returned_values = Vec::new();
        for i in &last_values {
            let mut j = *i;
            if let Some(node) = self.context.try_get_node(*i) {
                func.result_types.push(node.get_type());
                if let Some(ins) = self.context.try_get_instruction(*i) {
                    if ins.operation.opcode() == Opcode::Results {
                        // n.b. this required for result instructions, but won't hurt if done for all i
                        let new_var = node::Variable {
                            id: NodeId::dummy(),
                            obj_type: node.get_type(),
                            name: format!("return_{}", i.0.into_raw_parts().0),
                            root: None,
                            def: None,
                            witness: None,
                            parent_block: self.context.current_block,
                        };
                        let b_id = self.context.add_variable(new_var, None);
                        let b_id1 = self.context.handle_assign(b_id, None, *i)?;
                        j = ssa_form::get_current_value(&mut self.context, b_id1);
                    }
                }
            } else {
                func.result_types.push(ObjectType::NotAnObject);
            }
            returned_values.push(j);
        }

        self.context.new_instruction(
            node::Operation::Return(returned_values),
            node::ObjectType::NotAnObject,
        )?;
        let decision = func.compile(self)?; //unroll the function
        func.decision = decision;
        self.context.functions.insert(func_id, func);
        self.context.current_block = current_block;
        self.function_context = current_function;
        Ok(())
    }

    fn create_function_parameter(
        &mut self,
        id: DefinitionId,
        typ: &Type,
        name: &str,
    ) -> Vec<NodeId> {
        //check if the variable is already created:
        let val = match self.find_variable(id) {
            Some(var) => self.get_current_value(&var.clone()),
            None => self.create_new_value(typ, name, Some(id)),
        };
        val.to_node_ids()
    }

    //generates an instruction for calling the function
    pub fn call(&mut self, call: &Call) -> Result<Vec<NodeId>, RuntimeError> {
        let arguments = self.codegen_expression_list(&call.arguments);
        let call_instruction = self.context.new_instruction(
            node::Operation::Call {
                func_id: call.func_id,
                arguments,
                returned_arrays: Vec::new(),
                predicate: AssumptionId::dummy(),
            },
            ObjectType::NotAnObject,
        )?;

        let rtt = self.context.functions[&call.func_id].result_types.clone();
        let mut result = Vec::new();
        for i in rtt.iter().enumerate() {
            result.push(self.context.new_instruction(
                node::Operation::Result { call_instruction, index: i.0 as u32 },
                *i.1,
            )?);
        }
        Ok(result)
    }

    //Lowlevel functions with no more than 2 arguments
    pub fn call_low_level(
        &mut self,
        op: OPCODE,
        call: &ast::CallLowLevel,
    ) -> Result<NodeId, RuntimeError> {
        //Inputs
        let mut args: Vec<NodeId> = Vec::new();

        for arg in &call.arguments {
            if let Ok(lhs) = self.codegen_expression(arg) {
                args.push(lhs.unwrap_id()); //TODO handle multiple values
            } else {
                panic!("error calling {}", op);
            }
        }
        //REM: we do not check that the nb of inputs correspond to the function signature, it is done in the frontend

        //Output:
        let (len, elem_type) = get_result_type(op);
        let result_type = if len > 1 {
            //We create an array that will contain the result and set the res_type to point to that array
            let result_index = self.new_array(&format!("{op}_result"), elem_type, len, None).1;
            node::ObjectType::Pointer(result_index)
        } else {
            elem_type
        };

        //when the function returns an array, we use ins.res_type(array)
        //else we map ins.id to the returned witness
        //Call instruction
        self.context.new_instruction(node::Operation::Intrinsic(op, args), result_type)
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

pub fn update_call_graph(call_graph: &mut Vec<Vec<u8>>, caller: FuncIndex, callee: FuncIndex) {
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
