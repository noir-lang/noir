use std::collections::HashMap;

use crate::environment::Environment;
use acvm::acir::OPCODE;
use noirc_frontend::hir_def::expr::HirCallExpression;
use noirc_frontend::node_interner::FuncId;

use super::node::NodeId;
use super::{
    code_gen::IRGenerator,
    node::{self, ObjectType},
};

pub struct SSAFunction<'a> {
    pub cfg: IRGenerator<'a>,
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
        let mut last_statment = NodeId::dummy();
        for stmt in block {
            last_statment = self.cfg.evaluate_statement(env, stmt).unwrap();
        }
        let result = if last_statment == NodeId::dummy() {
            Vec::new()
        } else {
            vec![last_statment]
        };
        //Create return instruction based on the last statement of the function body
        let result_id = self.cfg.new_instruction(
            NodeId::dummy(),
            NodeId::dummy(),
            node::Operation::Ret,
            node::ObjectType::NotAnObject,
        );
        self.cfg.get_mut_instruction(result_id).ins_arguments = result; //n.b. should we keep the object type in the vector?
    }

    pub fn new(func: FuncId, ctx: &'a noirc_frontend::hir::Context) -> SSAFunction<'a> {
        SSAFunction {
            cfg: IRGenerator::new(ctx),
            id: func,
            arguments: Vec::new(),
        }
    }

    pub fn compile(&mut self) {
        //Optimisation
        super::block::compute_dom(&mut self.cfg);
        super::optim::cse(&mut self.cfg);
        //Unrolling
        super::flatten::unroll_tree(&mut self.cfg);
        super::optim::cse(&mut self.cfg);
    }

    //generates an instruction for calling the function
    pub fn call(
        func: FuncId,
        arguments: &[noirc_frontend::node_interner::ExprId],
        eval: &mut IRGenerator,
        env: &mut Environment,
    ) -> NodeId {
        let call_id = eval.new_instruction(
            NodeId::dummy(),
            NodeId::dummy(),
            node::Operation::Call(func),
            node::ObjectType::NotAnObject, //TODO how to get the function return type?
        );
        let ins_arguments = eval.expression_list_to_objects(env, arguments);
        let call_ins = eval.get_mut_instruction(call_id);
        call_ins.ins_arguments = ins_arguments;
        call_id
    }

    pub fn get_mapped_value(
        func_id: noirc_frontend::node_interner::FuncId,
        var: NodeId,
        irgen: &mut IRGenerator,
        inline_map: &HashMap<NodeId, NodeId>,
    ) -> NodeId {
        if var == NodeId::dummy() {
            return var;
        }
        let mut my_const = None;
        if let Some(node::NodeObj::Const(c)) = irgen.functions_cfg[&func_id].cfg.try_get_node(var) {
            my_const = Some((c.get_value_field(), c.value_type));
        }

        if let Some(c) = my_const {
            irgen.get_or_create_const(c.0, c.1)
        } else {
            *inline_map.get(&var).unwrap()
        }
    }
}

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
    }
}

//Lowlevel functions with no more than 2 arguments
pub fn call_low_level(
    op: OPCODE,
    call_expr: HirCallExpression,
    eval: &mut IRGenerator,
    env: &mut Environment,
) -> NodeId {
    //Inputs
    let mut args: Vec<NodeId> = Vec::new();

    for arg in &call_expr.arguments {
        if let Ok(lhs) = eval.expression_to_object(env, arg) {
            args.push(lhs);
        } else {
            panic!("error calling {}", op);
        }
    }
    //REM: we do not check that the nb of inputs correspond to the function signature, it is done in the frontend

    //Output:
    let result_signature = get_result_type(op);
    let result_type = if result_signature.0 > 1 {
        //We create an array that will contain the result and set the res_type to point to that array
        let result_index = eval.mem.create_new_array(
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
    eval.new_instruction_with_multiple_operands(
        &mut args,
        node::Operation::Intrinsic(op),
        result_type,
    )
}
