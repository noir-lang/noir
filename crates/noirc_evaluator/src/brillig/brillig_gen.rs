use std::collections::{HashMap, HashSet};

use crate::ssa;
use crate::ssa::block::{self, BlockId, BlockType};
use crate::ssa::context::SsaContext;
use crate::ssa::function::RuntimeType;
use crate::ssa::mem::Memory;
use crate::ssa::node::{Binary, BinaryOp, Instruction, NodeId, NodeObject, ObjectType, Operation};
use acvm::acir::brillig_bytecode;
use acvm::FieldElement;

use acvm::acir::brillig_bytecode::{
    Opcode as BrilligOpcode, RegisterIndex, RegisterMemIndex, Typ as BrilligType,
    OracleData,
};
 
 const CALLBACK_REGISTER: usize = 10000;

#[derive(Default)]
pub(crate) struct BrilligGen {
    byte_code: Vec<BrilligOpcode>,
    to_fix: Vec<(usize, BlockId)>,
    blocks: HashMap<BlockId, usize>, //processed blocks and their entry point
    max_register: usize,
    functions: HashMap<NodeId, usize>,
    functions_to_process: HashSet<NodeId>,
}

impl BrilligGen {
    pub(crate) fn ir_to_brillig(ctx: &SsaContext, block: BlockId) -> Vec<BrilligOpcode> {
        let mut brillig = BrilligGen::default();
        brillig.byte_code.push(BrilligOpcode::JMP { destination: 2 });
        brillig.byte_code.push(BrilligOpcode::Trap);
        brillig.process_functions(ctx);
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
                }
                BrilligOpcode::JMPIFNOT { condition, destination } => {
                    assert_eq!(destination, 0);
                    let current = self.blocks[block];
                    self.byte_code[*jump] =
                        BrilligOpcode::JMPIFNOT { condition, destination: current };
                }
                BrilligOpcode::JMPIF { condition, destination } => {
                    assert_eq!(destination, 0);
                    let current = self.blocks[block];
                    self.byte_code[*jump] =
                        BrilligOpcode::JMPIF { condition, destination: current };
                }
                _ => unreachable!(),
            }
        }
    }

    fn get_tmp_register(&mut self) -> RegisterIndex {
        self.max_register += 1;
        RegisterIndex(self.max_register)
    }

    // handle Phi instructions
    fn handle_phi_instructions(&mut self, current: BlockId, left: BlockId, ctx: &SsaContext) {
        if matches!(ctx[left].kind, BlockType::ForJoin | BlockType::IfJoin) {
            for i in &ctx[left].instructions {
                if let Some(ins) = ctx.try_get_instruction(*i) {
                    match &ins.operation {
                        Operation::Nop => continue,
                        Operation::Phi { root, block_args } => {
                            for (id, bid) in block_args {
                                if *bid == current {
                                    let destination = self.node_2_register(ctx, ins.id);
                                    let source = self.node_2_register(ctx, *id);
                                    self.byte_code.push(BrilligOpcode::Mov { destination, source });
                                }
                            }
                        }
                        _ => break,
                    }
                }
            }
        }
    }

    fn process_blocks(&mut self, ctx: &SsaContext, current: BlockId) {
        let mut queue = vec![current]; //Stack of elements to visit

        while let Some(current) = queue.pop() {
            let children = self.process_block(ctx, current);

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
        for i in block.instructions.iter().take(block.instructions.len() - 1) {
            let ins = ctx.try_get_instruction(*i).expect("instruction in instructions list");
            self.instruction_to_bc(ctx, ins);
        }

        let jump = block
            .instructions
            .last()
            .and_then(|i| {
                let ins = ctx.try_get_instruction(*i).expect("instruction in instructions list");
                match ins.operation {
                    Operation::Jne(cond, target) => {
                        let condition = self.node_2_register(ctx, cond);
                        Some((BrilligOpcode::JMPIFNOT { condition, destination: 0 }, target))
                    }
                    Operation::Jeq(cond, target) => {
                        let condition = self.node_2_register(ctx, cond);
                        Some((BrilligOpcode::JMPIF { condition, destination: 0 }, target))
                    }
                    Operation::Jmp(target) => Some((BrilligOpcode::JMP { destination: 0 }, target)),
                    _ => {
                        self.instruction_to_bc(ctx, ins);
                        None
                    }
                }
            })
            .or_else(|| {
                block.left.map(|left| (BrilligOpcode::JMP { destination: 0 }, left))
                // if let Some(left) = block.left {
                //     Some((BrilligOpcode::JMP { destination: 0 }, left))
                // } else {
                //     None
                // }
            });
        if let Some(left) = block.left {
            self.handle_phi_instructions(block_id, left, ctx);
        }
        if let Some((jmp, target)) = jump {
            self.to_fix.push((self.byte_code.len(), target));
            self.byte_code.push(jmp);
        }

        let mut result = Vec::new();
        if ctx.get_if_condition(block).is_some() {
            //find exit node:
            let exit = block::find_join(ctx, block.id);
            assert!(ctx[exit].kind == BlockType::IfJoin);
            result.push(exit);
        }
        if let Some(right) = block.right {
            result.push(right);
        }

        if let Some(left) = block.left {
            result.push(left);
        } else {
            self.byte_code.push(BrilligOpcode::JMP { destination: usize::MAX });
        }
        self.blocks.insert(block_id, start);
        result
    }

    fn instruction_to_bc(&mut self, ctx: &SsaContext, ins: &Instruction) {
        match &ins.operation {
            Operation::Binary(bin) => {
                self.binary(ctx, bin, ins.id, ins.res_type);
            }
            Operation::Cast(_) => todo!(),
            Operation::Truncate { value, bit_size, max_bit_size } => unreachable!(), //no overflow pass
            Operation::Not(_) => todo!(),                                            // bitwise not
            Operation::Constrain(a, loc) => {
                let condition = self.node_2_register(ctx, *a);
                self.byte_code.push(BrilligOpcode::JMPIFNOT { condition, destination: 1 });
            }
            Operation::Jne(_, _) | Operation::Jeq(_, _) | Operation::Jmp(_) => {
                unreachable!("a jump can only be at the very end of a block")
            }
            Operation::Phi { root, block_args } => (),
            Operation::Call { func, arguments, returned_arrays, predicate, location } => {
                // is public function ? yes: call opcode
                //save the VM state
                //  let b = safe_call f, a;
                // b = (1) ; (1:f_in-out, a-b) 
                // 
                
                //f-> bc take the inputs from signature
                //on doit juste assigner a aux bons nodeid
                //pour b; on va faire safe_call:
                //on doit juste mover les f.return vers ??
                //if not: put the arguments to the registers
                // set the function_call_back_register to next_instruction
                //jump function_call_back_register: NO done by the VM
                todo!();
                //let argument_registers = arguments.iter().map(|x| self.node_2_register(ctx, *x)).collect();
            }
            Operation::Return(ret) => {
                match ret.len() {
                    0 => (),
                    1 => {
                        let ret_register = self.node_2_register(ctx, ret[0]);
                        self.byte_code.push(BrilligOpcode::Mov {
                            destination: RegisterMemIndex::Register(RegisterIndex(0)),
                            source: ret_register,
                        });
                    }
                    _ => {
                        todo!("return the memory pointer of the array");
                    }
                }
                self.byte_code.push(BrilligOpcode::JMP { destination: usize::MAX });
            }
            Operation::Result { call_instruction, index } => todo!(),
            Operation::Cond { condition, val_true, val_false } => unreachable!(),
            Operation::Load { array_id, index, location } => todo!(),
            Operation::Store { array_id, index, value, predicate, location } => todo!(),
            Operation::Intrinsic(_, _) => todo!(),
            Operation::UnsafeCall { func, arguments, returned_values, predicate, location } => {
                if let Some(func_id) = ctx.try_get_func_id(*func) {
                    let ssa_func = ctx.ssa_func(func_id).unwrap();
                    match ssa_func.kind.clone() {
                        RuntimeType::Oracle(name) => {
                            let mut outputs = Vec::new();
                            for i in returned_values {
                                outputs.push(self.node_2_register(ctx, *i).to_register_index().unwrap());
                            }
                            let mut inputs = Vec::new();
                            for i in arguments {
                                inputs.push(self.node_2_register(ctx, *i));
                            }
                            self.byte_code.push(brillig_bytecode::Opcode::Oracle(OracleData{
                               name, inputs, input_values:  Vec::new(), outputs, output_values: Vec::new() }));
                        }
                             
                        
                        RuntimeType::Unsafe => {
                            // we need to have a place for the functions
                            let func_adr = if let Some(func_adr) = self.functions.get(func) {
                                *func_adr
                            } else {
                                //todo we will need to fix this jump later-on.
                                //but now it is not a block? or we could use the function.first_block?
                                //TODO
                                0
                            };
                                //mov inputs to function arguments:
                                for (input, arg) in ssa_func.arguments.iter().zip(arguments) {
                                    let arg_reg = self.node_2_register(ctx, *arg);
                                    let in_reg = self.node_2_register(ctx, input.0);
                                    self.byte_code.push(brillig_bytecode::Opcode::Mov { destination: in_reg, source: arg_reg } );
                                }
                                let call_back = FieldElement::from(self.byte_code.len() as i128 + 1);
                                self.byte_code.push(brillig_bytecode::Opcode::Mov { destination: RegisterMemIndex::Register(
                                    RegisterIndex(CALLBACK_REGISTER)
                                ), source: RegisterMemIndex::Constant(call_back) });
                                
                                if func_adr == 0 {
                                    self.to_fix.push((self.byte_code.len(), ssa_func.entry_block));
                                    self.functions_to_process.insert(*func);
                                }
                                self.byte_code.push(brillig_bytecode::Opcode::JMP { destination: func_adr });

                                //result is in register 0
                                if returned_values.len() == 1  {
                                    let first = self.node_2_register(ctx, *returned_values.first().unwrap());
                                    self.byte_code.push(brillig_bytecode::Opcode::Mov { destination: first,
                                        source: 
                                    RegisterMemIndex::Register(RegisterIndex(0)) });
                                }

                        }
                        RuntimeType::Acvm => unimplemented!(),
                    }
                }
            },
            Operation::Nop => (),
        }
    }

    fn node_2_register(&mut self, ctx: &SsaContext, a: NodeId) -> RegisterMemIndex //register-value enum
    {
        let a_register = a.0.into_raw_parts().0;
        assert_ne!(a_register, CALLBACK_REGISTER);
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
            NodeObject::Const(c) => RegisterMemIndex::Constant(FieldElement::from_be_bytes_reduce(
                &c.value.to_bytes_be(),
            )),
            NodeObject::Function(_, _, _) => todo!(),
        }
    }

    fn binary(&mut self, ctx: &SsaContext, binary: &Binary, id: NodeId, object_type: ObjectType) {
        let lhs = self.node_2_register(ctx, binary.lhs);
        let rhs = self.node_2_register(ctx, binary.rhs);
        let result_type = object_type_2_typ(object_type);
        let result = self.node_2_register(ctx, id).to_register_index().unwrap();

        match &binary.operator {
        BinaryOp::Add => {
            self.byte_code.push(BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Add,
                result,
            });
        }
        BinaryOp::SafeAdd => todo!(),
        BinaryOp::Sub { .. } => self.byte_code.push(BrilligOpcode::BinaryOp {
            lhs,
            rhs,
            result_type,
            op: brillig_bytecode::BinaryOp::Sub,
            result,
        }),
        BinaryOp::SafeSub { .. } => todo!(),
        BinaryOp::Mul => self.byte_code.push(BrilligOpcode::BinaryOp {
            lhs,
            rhs,
            result_type,
            op: brillig_bytecode::BinaryOp::Mul,
            result,
        }),
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
            self.byte_code.push(BrilligOpcode::BinaryOp { result_type, op: brillig_bytecode::BinaryOp::Sub, lhs, rhs: RegisterMemIndex::Register(q), result });
        }
        BinaryOp::Srem(_) => todo!(),
        BinaryOp::Udiv(_) |
        BinaryOp::Sdiv(_) |
        BinaryOp::Div(_) => {
            self.byte_code.push(BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Div,
                result,
            });
        },
        BinaryOp::Eq => {
            self.byte_code.push(BrilligOpcode::BinaryOp { result_type: BrilligType::Unsigned { bit_size: 1 }, op: brillig_bytecode::BinaryOp::Cmp(brillig_bytecode::Comparison::Eq
        ), lhs, rhs, result});
        }, //a==b => is_zero()
        BinaryOp::Ne =>
     {
        self.byte_code.push(BrilligOpcode::BinaryOp { result_type: BrilligType::Unsigned { bit_size: 1 }, op: brillig_bytecode::BinaryOp::Cmp(brillig_bytecode::Comparison::Eq
        ), lhs, rhs, result});
        self.byte_code.push(
            BrilligOpcode::BinaryOp { result_type: BrilligType::Unsigned { bit_size: 1 }, op: brillig_bytecode::BinaryOp::Sub, lhs: RegisterMemIndex::Constant(FieldElement::one())
            , rhs: RegisterMemIndex::Register(result), result}
        );
     }
           // comparison
        BinaryOp::Ule |//<= = >= , <
        BinaryOp::Lte |
        BinaryOp::Sle => {
            //a<=b : !b<a
            let t = self.get_tmp_register();
            //b<a .. todo
            self.byte_code.push(BrilligOpcode::BinaryOp { result_type, op: brillig_bytecode::BinaryOp::Sub,
            lhs: RegisterMemIndex::Constant(FieldElement::one()),
            rhs: RegisterMemIndex::Register(t),
            result,});
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
    
    fn process_functions(&mut self, ctx: &SsaContext) {
        for f in self.functions_to_process.iter() {
            if let Some(ssa_func) = ctx.try_get_ssa_func(*f) {
                self.blocks.insert(ssa_func.entry_block, self.byte_code.len());
                self.byte_code.append(&mut ssa_func.brillig_code.clone());
                self.byte_code.push(
                    BrilligOpcode::Call { destination: 
                    RegisterMemIndex::Register(RegisterIndex(CALLBACK_REGISTER))}
                );
            }
        }
    }

}

fn object_type_2_typ(object_type: ObjectType) -> BrilligType {
    match object_type {
        ObjectType::NativeField => BrilligType::Field,
        ObjectType::Boolean => BrilligType::Unsigned { bit_size: 1 },
        ObjectType::Unsigned(s) => BrilligType::Unsigned { bit_size: s },
        ObjectType::Signed(s) => BrilligType::Signed { bit_size: s },
        ObjectType::Pointer(_) => todo!(),
        ObjectType::Function => todo!(),
        ObjectType::NotAnObject => todo!(),
    }
}

pub(crate) fn directive_invert() -> Vec<BrilligOpcode> {
    vec![
        BrilligOpcode::JMPIFNOT {
            condition: RegisterMemIndex::Register(RegisterIndex(0)),
            destination: 2,
        },
        BrilligOpcode::BinaryOp {
            result_type: BrilligType::Field,
            op: brillig_bytecode::BinaryOp::Div,
            lhs: RegisterMemIndex::Constant(FieldElement::one()),
            rhs: RegisterMemIndex::Register(RegisterIndex(0)),
            result: RegisterIndex(0),
        },
    ]
}
