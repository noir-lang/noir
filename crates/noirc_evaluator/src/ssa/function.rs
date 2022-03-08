use acvm::acir::OPCODE;
use noirc_frontend::hir_def::expr::HirCallExpression;
use noirc_frontend::node_interner::{IdentId, FuncId};
use crate::environment::Environment;

use super::{code_gen::IRGenerator, node::{self, ObjectType}};


pub struct SSAFunction<'a> {
    pub cfg: IRGenerator<'a>,
    pub id: FuncId,
    //signature..
}

//new: cfg= IRGenerator::new();

impl<'a> SSAFunction<'a> 
{
    pub fn parse_statements(&mut self, block: &[noirc_frontend::node_interner::StmtId], env: &mut Environment) {
        for stmt in block {
            self.cfg.evaluate_statement(env, stmt);
        }  
    }


    pub fn new(func: FuncId, ctx: &'a noirc_frontend::hir::Context) -> SSAFunction<'a> {
        SSAFunction {
            cfg: IRGenerator::new(ctx),
            id: func,
        }
    }

    //generates an instruction for calling the function
    pub fn call1(&self, arguments: &[noirc_frontend::node_interner::ExprId], eval: &mut IRGenerator, env: &mut Environment,) -> arena::Index {
       
        let call_id = eval.new_instruction(
            eval.dummy(),
            eval.dummy(), 
            node::Operation::Nop,//node::Operation::Call(self.id),  
            node::ObjectType::NotAnObject,  //TODO 
        );
        let arguments2 = eval.expression_list_to_objects(env, arguments);
        let call_ins = eval.get_mut_instruction(call_id);
        call_ins.ins_arguments = arguments2;
        call_id
    }

    //generates an instruction for calling the function
    pub fn call(func: FuncId, arguments: &[noirc_frontend::node_interner::ExprId], eval: &mut IRGenerator, env: &mut Environment,) -> arena::Index {
       
        let call_id = eval.new_instruction(
            eval.dummy(),
            eval.dummy(), 
            node::Operation::Call(func),  
            node::ObjectType::NotAnObject,  //TODO how to get the function return type?
        );
        let arguments2 = eval.expression_list_to_objects(env, arguments);
        let call_ins = eval.get_mut_instruction(call_id);
        call_ins.ins_arguments = arguments2;
        call_id
    }
}


pub fn get_result_type(op: OPCODE) -> (u32, ObjectType)
{
    match op {
        OPCODE::AES => (0, ObjectType::NotAnObject),        //Not implemented
        OPCODE::SHA256 =>  (32, ObjectType::Unsigned(8)),   //or Field?
        OPCODE::Blake2s =>  (32, ObjectType::Unsigned(8)),   //or Field?
        OPCODE::HashToField => (1, ObjectType::NativeField),  
        OPCODE::MerkleMembership => (1, ObjectType::NativeField), //or bool?
        OPCODE::SchnorrVerify => (1, ObjectType::NativeField), //or bool?
        OPCODE::Pedersen => (2, ObjectType::NativeField),  
        OPCODE::EcdsaSecp256k1 => (1, ObjectType::NativeField),  //field?
        OPCODE::FixedBaseScalarMul => (2, ObjectType::NativeField),  
        OPCODE::InsertRegularMerkle => (1, ObjectType::NativeField),  //field?
    }
}


pub fn call_func(func: FuncId, args: Vec<arena::Index>, eval: &mut IRGenerator, /*env: &mut Environment*/) -> arena::Index
{
 
 
    let call_id = eval.new_instruction(
        eval.dummy(),
        eval.dummy(), 
        node::Operation::Nop,//TODO //node::Operation::Call(func),  
        node::ObjectType::NotAnObject,  //TODO...
    );
    let call_ins = eval.get_mut_instruction(call_id);
    call_ins.ins_arguments = args;
    call_id
}



//Lowlevel functions with no more than 2 arguments
pub fn call_low_level(op: OPCODE, call_expr: HirCallExpression, eval: &mut IRGenerator, env: &mut Environment,) -> arena::Index {

    //Inputs
    let mut args : Vec<arena::Index> = Vec::new();
    for arg in &call_expr.arguments {
        if let Ok(lhs) = eval.expression_to_object(env, arg) {
            args.push(lhs);
        }
        else {
            panic!("error calling {}",op);
        }

    }
    //REM: we do not check that the nb of inputs correspond to the function signature, it should have been done in the frontend?
    while args.len() < 2 {
        if !args.is_empty() {
            args.push(args[0])
        }
        else {
            args.push(eval.dummy());
        }
        
    }
    if args.len() > 2 {
        todo!("too many arguments");
        //we should create an array that encapsulate all the inputs, or
        //use the phi_arguments vector of the instruction (a renommer du coup)
    }

    //Output:
    let result_signature = get_result_type(op);
    let result_type = if result_signature.0 > 1 {
        //We create an array that will contain the result and set the res_type to point to that array
        let result_index = eval.mem.create_new_array(result_signature.0,  result_signature.1, format!("{}_result", op));
        node::ObjectType::Pointer(result_index)
    } else {
        result_signature.1
    };

    //when the function returns an array, we use ins.res_type(array)
    //else we map ins.id to the returned witness

     //Call instruction
     eval.new_instruction(
        args[0],
         args[1],
         node::Operation::Intrinsic(op),
         result_type,
     )
}

