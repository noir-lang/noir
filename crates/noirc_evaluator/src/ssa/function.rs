use std::collections::HashMap;

use crate::environment::Environment;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use noirc_frontend::hir_def::expr::{HirCallExpression, HirIdent};
use noirc_frontend::hir_def::function::Parameters;
use noirc_frontend::hir_def::stmt::HirPattern;
use noirc_frontend::node_interner::FuncId;

use super::code_gen::Value;
use super::{
    block::BlockId,
    code_gen::IRGenerator,
    context::SsaContext,
    node::{self, NodeId, ObjectType},
    ssa_form,
};

#[derive(Clone, Debug)]
pub struct SSAFunction {
    pub entry_block: BlockId,
    pub id: FuncId,
    pub index: u32,
    //signature:
    pub arguments: Vec<NodeId>,
    pub result: Vec<ObjectType>,
}

impl SSAFunction {
    //Parse the AST function body into ssa form in cfg
    pub fn parse_statements(
        igen: &mut IRGenerator,
        block: &[noirc_frontend::node_interner::StmtId],
        env: &mut Environment,
    ) -> Value {
        let mut last_value = Value::dummy();
        for stmt in block {
            last_value = igen.evaluate_statement(env, stmt).unwrap();
        }
        last_value
    }

    pub fn new(func: FuncId, block_id: BlockId, index: u32) -> SSAFunction {
        SSAFunction {
            entry_block: block_id,
            id: func,
            arguments: Vec::new(),
            result: Vec::new(),
            index,
        }
    }

    pub fn compile(&self, igen: &mut IRGenerator, last: NodeId) -> Option<NodeId> {
        let function_cfg = super::block::bfs(self.entry_block, None, &igen.context);
        super::block::compute_sub_dom(&mut igen.context, &function_cfg);
        //Optimisation
        igen.context.print_block(&igen.context[self.entry_block]);
        super::optim::full_cse(&mut igen.context, self.entry_block);

        //Unrolling
        let eval = super::flatten::unroll_tree(&mut igen.context, self.entry_block);
        super::optim::full_cse(&mut igen.context, self.entry_block);
        if eval.contains_key(&last) {
            eval[&last].into_node_id()
        } else {
            Some(last)
        }
    }

    //generates an instruction for calling the function
    pub fn call(
        func: FuncId,
        arguments: &[noirc_frontend::node_interner::ExprId],
        igen: &mut IRGenerator,
        env: &mut Environment,
    ) -> NodeId {
        let otype = igen.context.functions[&func].result[0];
        let ins_arguments = igen.expression_list_to_objects(env, arguments);
        let call_id = igen.context.new_instruction(
            NodeId::dummy(),
            NodeId::dummy(),
            node::Operation::Call(func),
            otype,
        );

        let call_ins = igen.context.get_mut_instruction(call_id);
        call_ins.ins_arguments = CallStack { arguments: ins_arguments, return_values: Vec::new() };
        call_id
    }

    pub fn get_mapped_value(
        var: Option<&NodeId>,
        ctx: &mut SsaContext,
        inline_map: &HashMap<NodeId, NodeId>,
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
                ssa_form::get_current_value(ctx, node_id)
            }
        } else {
            NodeId::dummy()
        }
    }
}

#[derive(Clone, Debug)]
pub struct CallStack {
    pub arguments: Vec<NodeId>,
    pub return_values: Vec<NodeId>,
}

impl CallStack {
    pub fn new() -> CallStack {
        CallStack { arguments: Vec::new(), return_values: Vec::new() }
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
        OPCODE::InsertRegularMerkle => (1, ObjectType::NativeField), //field?
        OPCODE::ToBits => (FieldElement::max_num_bits(), ObjectType::Boolean),
    }
}

//Lowlevel functions with no more than 2 arguments
pub fn call_low_level(
    op: OPCODE,
    call_expr: HirCallExpression,
    igen: &mut IRGenerator,
    env: &mut Environment,
) -> NodeId {
    //Inputs
    let mut args: Vec<NodeId> = Vec::new();

    for arg in &call_expr.arguments {
        if let Ok(lhs) = igen.expression_to_object(env, arg) {
            args.push(lhs.unwrap_id()); //TODO handle multiple values
        } else {
            panic!("error calling {}", op);
        }
    }
    //REM: we do not check that the nb of inputs correspond to the function signature, it is done in the frontend

    //Output:
    let result_signature = get_result_type(op);
    let result_type = if result_signature.0 > 1 {
        //We create an array that will contain the result and set the res_type to point to that array
        let result_index = igen.context.mem.create_new_array(
            result_signature.0,
            result_signature.1,
            &format!("{}_result", op),
        );
        node::ObjectType::Pointer(result_index)
    } else {
        result_signature.1
    };

    //when the function returns an array, we use ins.res_type(array)
    //else we map ins.id to the returned witness
    //Call instruction
    igen.context.new_instruction_with_multiple_operands(
        args,
        node::Operation::Intrinsic(op),
        result_type,
    )
}

pub fn param_to_ident(patern: &HirPattern) -> &HirIdent {
    match &patern {
        HirPattern::Identifier(id) => id,
        HirPattern::Mutable(pattern, _) => param_to_ident(pattern.as_ref()),
        HirPattern::Tuple(_, _) => todo!(),
        HirPattern::Struct(_, _, _) => todo!(),
    }
}

pub fn create_function(
    igen: &mut IRGenerator,
    func_id: FuncId,
    context: &noirc_frontend::hir::Context,
    env: &mut Environment,
    parameters: &Parameters,
    index: u32,
) {
    let current_block = igen.context.current_block;
    let current_function = igen.function_context;
    let func_block = super::block::BasicBlock::create_cfg(&mut igen.context);

    let mut func = SSAFunction::new(func_id, func_block, index);

    let function = context.def_interner.function(&func_id);
    let block = function.block(&context.def_interner);
    //argumemts:
    for pat in parameters.iter() {
        let ident_id = param_to_ident(&pat.0);
        let node_id = ssa_form::create_function_parameter(igen, &ident_id.id);
        func.arguments.push(node_id);
    }
    igen.function_context = Some(index);
    igen.context.functions.insert(func_id, func.clone());
    let last_value = SSAFunction::parse_statements(igen, block.statements(), env);
    let last_id = last_value.single_value(); //we do not support structures for now
    let last_mapped = func.compile(igen, last_id); //unroll the function
    let rtt = add_return_instruction(&mut igen.context, last_mapped);
    func.result.push(rtt);
    igen.context.functions.insert(func_id, func);
    igen.context.current_block = current_block;
    igen.function_context = current_function;
}

pub fn add_return_instruction(ctx: &mut SsaContext, last: Option<NodeId>) -> ObjectType {
    let mut results = Vec::new();
    if let Some(last_id) = last {
        results.push(last_id);
    }
    //Create return instruction based on the last statement of the function body
    let mut ret = node::Instruction::new(
        node::Operation::Ret,
        NodeId::dummy(),
        NodeId::dummy(),
        node::ObjectType::NotAnObject,
        Some(ctx.current_block),
    );
    let mut rtt = ObjectType::NotAnObject;
    if !results.is_empty() && results[0] != NodeId::dummy() {
        rtt = ctx.get_object_type(results[0]);
    }
    ret.ins_arguments = CallStack { arguments: results, return_values: Vec::new() };
    ctx.push_instruction(ret);
    rtt
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

pub fn update_call_graph(call_graph: &mut Vec<Vec<u8>>, caller: u32, callee: u32) {
    let a = caller as usize;
    let b = callee as usize;
    let max = a.max(b) + 1;
    resize_graph(call_graph, max);

    call_graph[a][b] = 1;
}

fn is_leaf(call_graph: &[Vec<u8>], i: usize) -> bool {
    for j in 0..call_graph[i].len() {
        if call_graph[i][j] == 1 {
            return false;
        }
    }
    true
}

fn get_new_leaf(ctx: &SsaContext, processed: &[usize]) -> (usize, FuncId) {
    for f in ctx.functions.values() {
        if !processed.contains(&(f.index as usize)) && is_leaf(&ctx.call_graph, f.index as usize) {
            return (f.index as usize, f.id);
        }
    }
    unimplemented!("Recursive function call is not supported");
}

//inline all functions of the call graph such that every inlining operates with a flatenned function
pub fn inline_all(ctx: &mut SsaContext) {
    resize_graph(&mut ctx.call_graph, ctx.functions.len());
    let l = ctx.call_graph.len();
    let mut processed = Vec::new();
    while processed.len() < l {
        let i = get_new_leaf(ctx, &processed);
        let i_entry = ctx.functions[&i.1].entry_block;
        let mut to_inline = Vec::new();
        for f in ctx.functions.values() {
            if ctx.call_graph[f.index as usize][i.0] == 1 {
                to_inline.push((f.id, f.index as usize));
            }
        }
        for j in to_inline {
            super::inline::inline_cfg(ctx, i_entry, Some(j.0));
            ctx.call_graph[j.1][i.0] = 0;
        }
        processed.push(i.0);
    }
    ctx.call_graph.clear();
}
