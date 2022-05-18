use std::collections::HashMap;

use crate::environment::Environment;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;
use noirc_frontend::hir_def::function::Parameters;
use noirc_frontend::hir_def::stmt::HirPattern;
use noirc_frontend::node_interner::FuncId;

use super::{
    block::BlockId,
    code_gen::{IRGenerator, Value},
    context::SsaContext,
    node::{self, NodeId, ObjectType, Operation},
    ssa_form,
};

pub struct SSAFunction {
    pub entry_block: BlockId,
    pub id: FuncId,
    //signature..
    pub arguments: Vec<NodeId>,
}

impl SSAFunction {
    //Parse the AST function body into ssa form in cfg
    pub fn parse_statements(
        igen: &mut IRGenerator,
        block: &[noirc_frontend::node_interner::StmtId],
        env: &mut Environment,
    ) {
        for stmt in block {
            igen.evaluate_statement(env, stmt).unwrap();
        }
    }

    pub fn new(func: FuncId, block_id: BlockId) -> SSAFunction {
        SSAFunction { entry_block: block_id, id: func, arguments: Vec::new() }
    }

    pub fn compile(&self, igen: &mut IRGenerator) -> Option<NodeId> {
        let function_cfg = super::block::bfs(self.entry_block, None, &igen.context);
        super::block::compute_sub_dom(&mut igen.context, &function_cfg);
        //Optimisation
        super::optim::cse(&mut igen.context, self.entry_block);
        //Unrolling
        super::flatten::unroll_tree(&mut igen.context, self.entry_block);
        super::optim::cse(&mut igen.context, self.entry_block)
    }

    //generates an instruction for calling the function
    pub fn call(
        func_id: FuncId,
        arguments: &[noirc_frontend::node_interner::ExprId],
        igen: &mut IRGenerator,
        env: &mut Environment,
    ) -> Value {
        let args = igen.expression_list_to_objects(env, arguments);

        // TODO: Need to create multiple returns for functions with struct return types
        let var = node::Variable {
            id: NodeId::dummy(),
            name: "result".into(),
            obj_type: ObjectType::NotAnObject,
            root: None,
            def: None,
            witness: None,
            parent_block: igen.context.current_block,
        };

        let result = igen.context.add_variable(var, None);
        igen.context.get_current_block_mut().update_variable(result, result);

        igen.context.new_instruction(
            Operation::Call { func_id, args, results: vec![result] },
            ObjectType::NotAnObject, //TODO how to get the function return type?
        );

        Value::Single(result)
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
                println!("Get or create const {} for id {:?}", c.0, node_id);
                ctx.get_or_create_const(c.0, c.1)
            } else {
                println!("Checking inline_map for id {:?}", node_id);
                inline_map.get(&node_id).copied().unwrap_or_else(NodeId::dummy)
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
    igen.context.new_instruction(node::Operation::Intrinsic(op, args), result_type)
}

pub fn create_function(
    igen: &mut IRGenerator,
    func_id: FuncId,
    context: &noirc_frontend::hir::Context,
    env: &mut Environment,
    parameters: &Parameters,
) -> SSAFunction {
    let current_block = igen.context.current_block;
    let func_block = super::block::BasicBlock::create_cfg(&mut igen.context);

    let mut func = SSAFunction::new(func_id, func_block);
    let function = context.def_interner.function(&func_id);
    let block = function.block(&context.def_interner);
    //argumemts:
    for pat in parameters.iter() {
        let ident_id = match &pat.0 {
            HirPattern::Identifier(id) => Some(id),
            HirPattern::Mutable(_pattern, _) => {
                unreachable!("mutable arguments are not supported yet")
            }
            HirPattern::Tuple(_, _) => todo!(),
            HirPattern::Struct(_, _, _) => todo!(),
        };

        let node_id = ssa_form::create_function_parameter(igen, &ident_id.unwrap().id);
        func.arguments.push(node_id);
    }
    //dbg!(&func.arguments);
    SSAFunction::parse_statements(igen, block.statements(), env);
    let last = func.compile(igen); //unroll the function
    add_return_instruction(&mut igen.context, last);
    igen.context.current_block = current_block;
    func
}

pub fn add_return_instruction(cfg: &mut SsaContext, last: Option<NodeId>) {
    let last_id = last.unwrap_or_else(NodeId::dummy);
    let result = if last_id == NodeId::dummy() { vec![] } else { vec![last_id] };
    //Create return instruction based on the last statement of the function body
    cfg.new_instruction(node::Operation::Return(result), node::ObjectType::NotAnObject);
    //n.b. should we keep the object type in the vector?
}
