use std::collections::HashMap;

use crate::environment::Environment;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use noirc_frontend::hir_def::expr::HirCallExpression;
use noirc_frontend::hir_def::function::Parameters;
use noirc_frontend::hir_def::stmt::HirPattern;
use noirc_frontend::node_interner::FuncId;

use super::{
    code_gen::IRGenerator,
    context::SsaContext,
    node::{self, NodeId, ObjectType},
    ssa_form,
};

pub struct SSAFunction<'a> {
    pub igen: IRGenerator<'a>,
    pub id: FuncId,
    //signature..
    pub arguments: Vec<NodeId>,
}

impl<'a> SSAFunction<'a> {
    //Parse the AST function body into ssa form in cfg
    pub fn parse_statements(
        &mut self,
        block: &[noirc_frontend::node_interner::StmtId],
        env: &mut Environment,
    ) {
        for stmt in block {
            self.igen.evaluate_statement(env, stmt).unwrap();
        }
    }

    pub fn new(func: FuncId, ctx: &'a noirc_frontend::hir::Context) -> SSAFunction<'a> {
        SSAFunction {
            igen: IRGenerator::new(ctx),
            id: func,
            arguments: Vec::new(),
        }
    }

    pub fn compile(&mut self) -> Option<NodeId> {
        //Optimisation
        super::block::compute_dom(&mut self.igen.context);
        super::optim::cse(&mut self.igen.context);
        //Unrolling
        super::flatten::unroll_tree(&mut self.igen.context);
        super::optim::cse(&mut self.igen.context)
    }

    //generates an instruction for calling the function
    pub fn call(
        func: FuncId,
        arguments: &[noirc_frontend::node_interner::ExprId],
        igen: &mut IRGenerator,
        env: &mut Environment,
    ) -> NodeId {
        let arguments = igen.expression_list_to_objects(env, arguments);
        igen.context.new_instruction(
            node::Operation::Call(func, arguments),
            node::ObjectType::NotAnObject, //TODO how to get the function return type?
        )
    }

    pub fn get_mapped_value(
        func_id: noirc_frontend::node_interner::FuncId,
        var: Option<&NodeId>,
        ctx: &mut SsaContext,
        inline_map: &HashMap<NodeId, NodeId>,
    ) -> NodeId {
        if let Some(&node_id) = var {
            if node_id == NodeId::dummy() {
                return node_id;
            }
            let mut my_const = None;
            let node_obj_opt = ctx.functions_cfg[&func_id]
                .igen
                .context
                .try_get_node(node_id);
            if let Some(node::NodeObj::Const(c)) = node_obj_opt {
                my_const = Some((c.get_value_field(), c.value_type));
            }
            if let Some(c) = my_const {
                ctx.get_or_create_const(c.0, c.1)
            } else {
                //                dbg!(&node_id);
                *inline_map.get(&node_id).unwrap()
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
    igen.context
        .new_instruction(node::Operation::Intrinsic(op, args), result_type)
}

pub fn create_function<'a>(
    func_id: FuncId,
    context: &'a noirc_frontend::hir::Context,
    env: &mut Environment,
    parameters: &Parameters,
) -> SSAFunction<'a> {
    let mut func = SSAFunction::new(func_id, context);
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

        let node_id = ssa_form::create_function_parameter(&mut func.igen, &ident_id.unwrap().id);
        func.arguments.push(node_id);
    }
    //dbg!(&func.arguments);
    func.parse_statements(block.statements(), env);
    let last = func.compile(); //unroll the function
    add_return_instruction(&mut func.igen.context, last);
    func
}

pub fn add_return_instruction(cfg: &mut SsaContext, last: Option<NodeId>) {
    let last_id = last.unwrap_or_else(NodeId::dummy);
    let result = if last_id == NodeId::dummy() {
        vec![]
    } else {
        vec![last_id]
    };
    //Create return instruction based on the last statement of the function body
    cfg.new_instruction(
        node::Operation::Return(result),
        node::ObjectType::NotAnObject,
    );
    //n.b. should we keep the object type in the vector?
}
