
use std::collections::HashMap;

use crate::ssa::context::SsaContext;
use crate::ssa::block::{BlockId, BlockType, self};
use crate::ssa::mem::Memory;
use crate::ssa::node::{BinaryOp, Binary, Operation, NodeId, NodeObject, ObjectType, Instruction, self};
use acvm::{FieldElement};
use acvm::acir::brillig_bytecode;


use acvm::acir::brillig_bytecode::{Opcode as BrilligOpcode, RegisterIndex, Typ as BrilligType, RegisterMemIndex};




#[derive(Default)]
pub struct BrilligGen {
    byte_code: Vec<BrilligOpcode>,
    to_fix: Vec<(usize, BlockId)>,
    blocks: HashMap<BlockId, usize>,    //processed blocks and their entry point
    max_register: usize,
}


impl BrilligGen {
    pub fn ir_to_brillig(ctx: &SsaContext, block: BlockId) -> Vec<BrilligOpcode> {
        let mut brillig = BrilligGen::default();
        brillig.byte_code.push(BrilligOpcode::JMP { destination: 2 });
        brillig.byte_code.push(BrilligOpcode::Trap);
        brillig.process_blocks(ctx, block);
        brillig.fix_jumps();
        brillig.byte_code
    }

    fn fix_jumps(&mut self) {
        for (jump, block) in &self.to_fix {
            match self.byte_code[*jump] {
                BrilligOpcode::JMP { destination } => {
                    assert_eq!(destination, 0);
                    let current = self.blocks[block];
                    self.byte_code[*jump] = BrilligOpcode::JMP { destination: current };
                } ,
                BrilligOpcode::JMPIFNOT { condition, destination } => {
                    assert_eq!(destination, 0);
                    let current = self.blocks[block];
                    self.byte_code[*jump] = BrilligOpcode::JMPIFNOT { condition, destination: current };
                },
                BrilligOpcode::JMPIF { condition, destination } => {
                    assert_eq!(destination, 0);
                    let current = self.blocks[block];
                    self.byte_code[*jump] = BrilligOpcode::JMPIF { condition, destination: current };
                },
                _ => unreachable!(),
            } 
        }
       }


    fn get_tmp_register(&mut self) -> RegisterIndex {
        self.max_register  += 1;
        RegisterIndex(self.max_register)
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
    dbg!(&start);
    ctx.print_block(block);
    
    //let mut jumps = Vec::new();
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
                                    let destination = self.node_2_register(ctx, ins.id);
                                    let source = self.node_2_register(ctx, *id);
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
    //self.,
    let last = self.byte_code.last();
    if let Some(last) = last {
        if let Some(left) = block.left {
            if !matches!(last, BrilligOpcode::JMP { .. } | BrilligOpcode::JMPIF{..} | BrilligOpcode::JMPIFNOT{..}) {
                self.byte_code.push( BrilligOpcode::JMP { destination: 0 });
                self.to_fix.push((self.byte_code.len()-1, left));        
            }
        }
    }
    let mut result = Vec::new();
    if ctx.get_if_condition(block).is_some() {
          //find exit node:
          let exit = block::find_join(ctx, block.id);
          assert!(ctx[exit].kind == BlockType::IfJoin);
          result.push(exit);
    }
    if let Some(right) = block.right {
       // self.to_fix.push((self.byte_code.len()-1, right));
        result.push(right);
    }
    if let Some(left) = block.left {
        result.push(left);
    }
    self.blocks.insert(block_id, start);
    result
}

fn instruction_to_bc(&mut self, ctx: &SsaContext, ins: &Instruction) -> Option<BrilligOpcode> {
    match &ins.operation {
    //    Binary(Binary { lhs, rhs, operator, predicate }) => todo!(),
        Operation::Binary(bin) => {
            let code = self.binary(ctx, bin, ins.id, ins.res_type);
            self.byte_code.push(code);
        },
        Operation::Cast(_) => todo!(),
        Operation::Truncate { value, bit_size, max_bit_size } => unreachable!(),    //no overflow pass
        Operation::Not(_) => todo!(),// bitwise not
        Operation::Constrain(a, loc) => {
            let condition = self.node_2_register(ctx, *a);
            self.byte_code.push(BrilligOpcode::JMPIFNOT { condition, destination: 1 });
        },
        //todo!(),  //assert => jumpif + error => we need an error opcode
        Operation::Jne(cond, block_id) => {
            self.to_fix.push((self.byte_code.len(), *block_id));
            let condition = self.node_2_register(ctx, *cond);
            return Some(BrilligOpcode::JMPIFNOT {
                condition,
                destination: 0,
            });

        },
        Operation::Jeq(cond, block_id) => {
            self.to_fix.push((self.byte_code.len(), *block_id));
            let condition = self.node_2_register(ctx, *cond);
            return Some(BrilligOpcode::JMPIF {
                condition,
                destination: 0,
            });

        },
        Operation::Jmp(block) => {
            self.to_fix.push((self.byte_code.len(), *block));
            return Some(BrilligOpcode::JMP {
                destination: 0,
            });
        },
        Operation::Phi { root, block_args } => (),
        Operation::Call { func, arguments, returned_arrays, predicate, location } => {
            // is public function ? yes: call opcode
            //save the VM state
            //if not: put the arguments to the registers
            // set the function_call_back_register to next_instruction
            //jump function_call_back_register: NO done by the VM
            todo!();
            //let argument_registers = arguments.iter().map(|x| self.node_2_register(ctx, *x)).collect();
            // res i,f => ids for the result
            //we need the results inside the call
            //=> its easier to have them inside the call
            // sinon; le result va faire res f, i , donc on doit 'fixer f avec
            //what if oracle ?

        },
        Operation::Return(ret) => {
            match ret.len() {
                0 => (),
                1 => {
                    let ret_register = self.node_2_register(ctx, ret[0]); 
                    self.byte_code.push(BrilligOpcode::Mov { destination: RegisterMemIndex::Register(RegisterIndex(0)), source: ret_register });
                }
                _ => {
                    todo!("return the memory pointer of the array");
                }
            }
        },
        Operation::Result { call_instruction, index } => todo!(),
        Operation::Cond { condition, val_true, val_false } => unreachable!(),
        Operation::Load { array_id, index, location } => todo!(),
        Operation::Store { array_id, index, value, predicate, location } => todo!(),
        Operation::Intrinsic(_, _) => todo!(),
        Operation::UnsafeCall { .. } => todo!(),
        Operation::Nop => (),

    }
    None
}


fn node_2_register(&mut self, ctx: &SsaContext, a: NodeId) -> RegisterMemIndex  //register-value enum
{
    let a_register = a.0.into_raw_parts().0;
    match &ctx[a] {
        NodeObject::Variable(_) => {
            if let Some(array) = Memory::deref(ctx, a) {
                todo!();
            } else {
                if a_register > self.max_register {
                    self.max_register = a_register;
                }
                RegisterMemIndex::Register(RegisterIndex(a_register))   
            }
            
        }
        crate::ssa::node::NodeObject::Instr(_) => {
            if a_register > self.max_register {
                self.max_register = a_register;
            }
            RegisterMemIndex::Register(RegisterIndex(a_register))
        }
        NodeObject::Const(c) => RegisterMemIndex::Constant(FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be())),
        NodeObject::Function(_, _, _) => todo!(),
    }
}

fn binary(&mut self, ctx: &SsaContext, binary: &Binary, id: NodeId, object_type: ObjectType) -> BrilligOpcode {
    let lhs = self.node_2_register(ctx, binary.lhs);
    let rhs = self.node_2_register(ctx, binary.rhs);
    let result_type = object_type_2_typ(object_type);
    let result = self.node_2_register(ctx, id).to_register_index().unwrap();
    
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
        BinaryOp::Sub { .. } => BrilligOpcode::BinaryOp {
            lhs,
            rhs,
            result_type,
            op: brillig_bytecode::BinaryOp::Sub,
            result,
        },
        BinaryOp::SafeSub { max_rhs_value } => todo!(),
        BinaryOp::Mul => BrilligOpcode::BinaryOp {
            lhs,
            rhs,
            result_type,
            op: brillig_bytecode::BinaryOp::Mul,
            result,
        },
        BinaryOp::SafeMul => todo!(),
        BinaryOp::Urem(_) => {
            let q = self.get_tmp_register();
            self.byte_code.push(BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Div,
                result:q,
            });
            self.byte_code.push(BrilligOpcode::BinaryOp {
                result_type,
                lhs: RegisterMemIndex::Register(q),
                rhs,
                op: brillig_bytecode::BinaryOp::Mul,
                result: q,
            });
            BrilligOpcode::BinaryOp { result_type, op: brillig_bytecode::BinaryOp::Sub, lhs, rhs: RegisterMemIndex::Register(q), result }
        }
        BinaryOp::Srem(_) => todo!(),
        BinaryOp::Udiv(_) |
        BinaryOp::Sdiv(_) |
        BinaryOp::Div(_) => {
            BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Div,
                result,
            }
        },
        BinaryOp::Eq => {
           BrilligOpcode::BinaryOp { result_type: BrilligType::Unsigned { bit_size: 1 }, op: brillig_bytecode::BinaryOp::Cmp(brillig_bytecode::Comparison::Equal
        ), lhs, rhs, result}
        }, //a==b => is_zero()
        BinaryOp::Ne => 
        BrilligOpcode::BinaryOp { result_type: BrilligType::Unsigned { bit_size: 1 }, op: brillig_bytecode::BinaryOp::Cmp(brillig_bytecode::Comparison::NotEqual
     ), lhs, rhs, result},
           // comparison
        BinaryOp::Ule |//<= = >= , <
        BinaryOp::Lte |
        BinaryOp::Sle => {
            //a<=b : !b<a
            let t = self.get_tmp_register();
            //b<a .. todo
            BrilligOpcode::BinaryOp { result_type, op: brillig_bytecode::BinaryOp::Sub,
            lhs: RegisterMemIndex::Constant(FieldElement::one()),
            rhs: RegisterMemIndex::Register(t),
            result,}
        },
        BinaryOp::Ult |
        BinaryOp::Slt |
        BinaryOp::Lt => todo!(), // a<b <=> ! b<=a 
        BinaryOp::And => todo!(),       //bitwise 
        BinaryOp::Or => todo!(),        
        BinaryOp::Xor => todo!(),
        BinaryOp::Shl => {
            todo!(); //ssa remove it during overflow.. can't we simplify as well?
        },
        BinaryOp::Shr(_) => todo!(),    //ssa remove it during overflow..
        BinaryOp::Assign => unreachable!(),
    }

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


