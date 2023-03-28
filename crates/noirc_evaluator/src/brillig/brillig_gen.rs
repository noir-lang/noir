
use std::collections::HashMap;

use crate::ssa::context::SsaContext;
use crate::ssa::block::{BlockId, BlockType};
use crate::ssa::mem::Memory;
use crate::ssa::node::{BinaryOp, Binary, Operation, NodeId, NodeObject, ObjectType, Instruction, self};
use acvm::{FieldElement};
use acvm::acir::brillig_bytecode;


use acvm::acir::brillig_bytecode::{Opcode as BrilligOpcode, RegisterIndex, Typ as BrilligType, RegisterMemIndex};




#[derive(Default)]
pub struct BrilligGen {
    byte_code: Vec<BrilligOpcode>,
    to_fix3: Option<usize>,      //jump to fix
    to_fix: Vec<(usize, BlockId)>,
    blocks: HashMap<BlockId, usize>,    //processed blocks and their entry point
}


impl BrilligGen {
    pub fn ir_to_brillig(ctx: &SsaContext, block: BlockId) -> Vec<BrilligOpcode> {
        let mut brillig = BrilligGen::default();
        brillig.process_blocks(ctx, block);
        brillig.fix_jumps();
        brillig.byte_code
    }

    fn fix_jumps(&mut self) {
        for (jump, block) in &self.to_fix {
            if let  BrilligOpcode::JMPIF { condition, destination } = self.byte_code[*jump] {
                assert_eq!(destination, 0);
                let current = self.blocks[block];
                self.byte_code[*jump] = BrilligOpcode::JMPIF { condition, destination: current };
            }   
        }
       }


    //pm v a faire:
    //si split; on met ses 2 fils en top de la stack
    //sinon, on met le next en bas de la stack
    fn process_blocks(
        &mut self,
        ctx: &SsaContext,
        current: BlockId,
//        data: &mut TreeBuilder,
    ) {
        let mut queue = vec![current]; //Stack of elements to visit
    
        while let Some(current) = queue.pop() {
            let children = self.process_block(ctx, current/*, data */);
    
            let mut add_to_queue = |block_id: BlockId| {
                if !block_id.is_dummy() && !queue.contains(&block_id) {
                    let block = &ctx[block_id];
                    if !block.is_join() || block.dominator == Some(current) {
                        queue.push(block_id);
                    }
                }
            };
                for i in children {
                    add_to_queue(i);
                }    
            
        }
    }
    
fn process_block(&mut self, ctx: &SsaContext, block_id: BlockId) -> Vec<BlockId> {
    let block = &ctx[block_id];
    let start = self.byte_code.len();

    //.. process block
    for i in &block.instructions {
        let ins = ctx.try_get_instruction(*i).expect("instruction in instructions list");
        self.instruction_to_bc(ctx, ins);
    }
    
    // handle Phi instructions
    if let Some(left) = block.left {
        if matches!(ctx[left].kind, BlockType::ForJoin | BlockType::IfJoin) {
            for i in &ctx[left].instructions {
                if let Some(ins) =  ctx.try_get_instruction(*i) {
                    match &ins.operation {
                        Operation::Nop => continue,
                        Operation::Phi { root, block_args } => {
                            for (id, bid) in block_args {
                                if *bid == block_id {
                                    let destination = node_2_register(ctx, ins.id);
                                    let source = node_2_register(ctx, *id);
                                    self.byte_code.push(BrilligOpcode::Mov { destination, source });
                                }
                            }
                        },
                        _ => break,
                    }
                }
            }
        }
    }
    let mut result = Vec::new();
    if let Some(right) = block.right {
        self.to_fix.push((self.byte_code.len()-1, right));
        result.push(right);
    }
    if let Some(left) = block.left {
        result.push(left);
    }
    self.blocks.insert(block_id, start);
    result
}

fn instruction_to_bc(&mut self, ctx: &SsaContext, ins: &Instruction) -> BrilligOpcode{
    match &ins.operation {
    //    Binary(Binary { lhs, rhs, operator, predicate }) => todo!(),
        Operation::Binary(bin) => binary(ctx, bin, ins.id, ins.res_type),
        Operation::Cast(_) => todo!(),
        Operation::Truncate { value, bit_size, max_bit_size } => unreachable!(),    //no overflow pass
        Operation::Not(_) => todo!(),// bitwise not
        Operation::Constrain(a, _) => todo!(),  //assert => jumpif + error => we need an error opcode
        Operation::Jne(_, _) => todo!(),
        Operation::Jeq(cond, block_id) => {
            self.to_fix.push((self.byte_code.len(), *block_id));
            BrilligOpcode::JMPIF {
                condition: node_2_register(ctx, *cond),
                destination: 0,
            }

        },
        Operation::Jmp(_) => todo!(),
        Operation::Phi { root, block_args } => todo!(),
        Operation::Call { func, arguments, returned_arrays, predicate, location } => todo!(),
        Operation::Return(_) => todo!(),
        Operation::Result { call_instruction, index } => todo!(),
        Operation::Cond { condition, val_true, val_false } => unreachable!(),
        Operation::Load { array_id, index, location } => todo!(),
        Operation::Store { array_id, index, value, predicate, location } => todo!(),
        Operation::Intrinsic(_, _) => todo!(),
        Operation::Nop => todo!(),

    }
}

}



fn node_2_register(ctx: &SsaContext, a: NodeId) -> RegisterMemIndex  //register-value enum
{
    match &ctx[a] {
        NodeObject::Variable(_) => {
            if let Some(array) = Memory::deref(ctx, a) {
                todo!();
            } else {
                RegisterMemIndex::Register(RegisterIndex(a.0.into_raw_parts().0))   
            }
            
        }
        crate::ssa::node::NodeObject::Instr(_) => todo!(),
        NodeObject::Const(c) => RegisterMemIndex::Constant(FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be())),
        NodeObject::Function(_, _, _) => todo!(),
    }
}

fn object_type_2_typ(object_type: ObjectType) -> BrilligType {
    match object_type {
        ObjectType::NativeField => BrilligType::Field,
        ObjectType::Boolean => BrilligType::Unsigned{bit_size: 1},
        ObjectType::Unsigned(s) => BrilligType::Unsigned{bit_size: s},
        ObjectType::Signed(s) => BrilligType::Signed{bit_size: s},
        ObjectType::Pointer(_) => todo!(),
        ObjectType::Function => todo!(),
        ObjectType::NotAnObject => todo!(),
    }
}


/// on veut : y=1/x => on va arriver dans acir-gen; evaluate_inverse, avec un witness
/// on l'utilise en input du opcode brillig:
/// BRILLIG { inputs: [x], outputs[y], bc: r0 = 1 DIV r0}
/// par default les arguments sont dans les registres 1,..n
/// les return values aussi !
/// 
pub fn directive_invert() -> Vec<BrilligOpcode> {
    vec![
    BrilligOpcode::JMPIFNOT { condition: RegisterMemIndex::Register(RegisterIndex(0)), destination: 2 } ,
    BrilligOpcode::BinaryOp { result_type: BrilligType::Field, op: brillig_bytecode::BinaryOp::Div, lhs: RegisterMemIndex::Constant(FieldElement::one()),
        rhs: RegisterMemIndex::Register(RegisterIndex(0)), result: RegisterIndex(0) },
    ]
}

fn binary(ctx: &SsaContext, binary: &Binary, id: NodeId, object_type: ObjectType) -> BrilligOpcode {
    //we transform lhs,rhs into 'constant' or a register
    //result is instruction id (=register)
    //

    let lhs = node_2_register(ctx, binary.lhs);
    let rhs = node_2_register(ctx, binary.rhs);
    let result_type = object_type_2_typ(object_type);
    let result = node_2_register(ctx, id).to_register_index().unwrap();
    
    match &binary.operator {
        BinaryOp::Add => {
            BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Add,
                result,
            }
        }
        BinaryOp::SafeAdd => todo!(),
        BinaryOp::Sub { max_rhs_value } => todo!(),
        BinaryOp::SafeSub { max_rhs_value } => todo!(),
        BinaryOp::Mul => todo!(),
        BinaryOp::SafeMul => todo!(),
        BinaryOp::Udiv(_) => todo!(),
        BinaryOp::Sdiv(_) => todo!(),
        BinaryOp::Urem(_) => todo!(),
        BinaryOp::Srem(_) => todo!(),
        BinaryOp::Div(_) => {
            BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Div,
                result,
            }
        },
        BinaryOp::Eq => todo!(), //a==b => is_zero()
        BinaryOp::Ne => todo!(),   //Not is_zero()
        BinaryOp::Ult => todo!(),   // comparison
        BinaryOp::Ule => todo!(),
        BinaryOp::Slt => todo!(),
        BinaryOp::Sle => todo!(),
        BinaryOp::Lt => todo!(),
        BinaryOp::Lte => todo!(),
        BinaryOp::And => todo!(),       //bitwise 
        BinaryOp::Or => todo!(),        
        BinaryOp::Xor => todo!(),
        BinaryOp::Shl => todo!(),       //ssa remove it during overflow..
        BinaryOp::Shr(_) => todo!(),    //ssa remove it during overflow..
        BinaryOp::Assign => unreachable!(),
    }

}

