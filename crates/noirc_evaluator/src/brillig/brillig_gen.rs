use std::collections::{HashMap, HashSet};

use crate::errors::{RuntimeError, RuntimeErrorKind};
use crate::ssa::block::{self, BlockId, BlockType};
use crate::ssa::context::SsaContext;
use crate::ssa::function::{RuntimeType, SsaFunction};
use crate::ssa::mem::Memory;
use crate::ssa::node::{
    Binary, BinaryOp, Instruction, NodeId, NodeObject, NumericType, ObjectType, Operation,
};
use acvm::acir::brillig_bytecode::{self, OracleInput};
use acvm::FieldElement;

use acvm::acir::brillig_bytecode::{
    Opcode as BrilligOpcode, OracleData, RegisterIndex, RegisterMemIndex, Typ as BrilligType,
};

const CALLBACK_REGISTER: usize = 10000;
const PREFIX_LEN: usize = 3;

#[derive(Default, Debug, Clone)]
pub(crate) struct BrilligArtefact {
    functions_to_process: HashSet<NodeId>,
    byte_code: Vec<BrilligOpcode>,
    to_fix: Vec<(usize, BlockId)>,
    blocks: HashMap<BlockId, usize>, //processed blocks and their entry point
}

impl BrilligArtefact {
    fn fix_jumps(&mut self, main_len: usize) {
        for (jump, block) in &self.to_fix {
            match self.byte_code[*jump] {
                BrilligOpcode::JMP { destination } => {
                    assert_eq!(destination, 0);
                    if block.is_dummy() {
                        if *jump < main_len {
                            self.byte_code[*jump] = BrilligOpcode::Stop;
                        } else {
                            self.byte_code[*jump] = BrilligOpcode::Call {
                                destination: RegisterMemIndex::Register(RegisterIndex(
                                    CALLBACK_REGISTER,
                                )),
                            };
                        }
                    } else {
                        let current = self.blocks[block];
                        self.byte_code[*jump] = BrilligOpcode::JMP { destination: current };
                    }
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
                BrilligOpcode::Mov { destination, source } => {
                    assert_eq!(source, RegisterMemIndex::Constant(FieldElement::zero()));
                    assert_eq!(
                        destination,
                        RegisterMemIndex::Register(RegisterIndex(CALLBACK_REGISTER))
                    );
                    self.byte_code[*jump] = BrilligOpcode::Mov {
                        destination: RegisterMemIndex::Register(RegisterIndex(CALLBACK_REGISTER)),
                        source: RegisterMemIndex::Constant(FieldElement::from((jump + 2) as i128)),
                    };
                }
                _ => unreachable!(),
            }
        }
    }

    fn link_with(&mut self, obj: &BrilligArtefact) {
        if obj.byte_code.is_empty() {
            panic!("ICE: unresolved symbol");
        }
        if self.byte_code.is_empty() {
            self.byte_code.push(BrilligOpcode::JMP { destination: PREFIX_LEN });
            self.byte_code.push(BrilligOpcode::Trap);
            self.byte_code.push(BrilligOpcode::Stop);
        }
        let offset = self.byte_code.len();
        for i in &obj.to_fix {
            self.to_fix.push((i.0 + offset, i.1));
        }
        for i in &obj.blocks {
            self.blocks.insert(*i.0, i.1 + offset);
        }
        self.byte_code.extend_from_slice(&obj.byte_code);
    }

    pub(crate) fn link(&mut self, ctx: &SsaContext, obj: &BrilligArtefact) -> Vec<BrilligOpcode> {
        self.link_with(obj);
        let main_len = self.byte_code.len();
        let mut queue: Vec<NodeId> = obj.functions_to_process.clone().into_iter().collect();
        while let Some(func) = queue.pop() {
            if let Some(ssa_func) = ctx.try_get_ssa_func(func) {
                if !self.blocks.contains_key(&ssa_func.entry_block) {
                    let obj = &ssa_func.obj;
                    self.link_with(obj);
                    let mut functions: Vec<NodeId> =
                        obj.functions_to_process.clone().into_iter().collect();
                    queue.append(&mut functions);
                }
            }
        }
        self.fix_jumps(main_len);
        self.byte_code.clone()
    }
}
#[derive(Default)]
pub(crate) struct BrilligGen {
    obj: BrilligArtefact,
    max_register: usize,
    functions: HashMap<NodeId, usize>,
    noir_call: Vec<NodeId>,
}

impl BrilligGen {
    /// Generate compilation object from ssa code
    pub(crate) fn compile(ctx: &SsaContext, block: BlockId) -> Result<BrilligArtefact, RuntimeError> {
        let mut brillig = BrilligGen::default();
        brillig.process_blocks(ctx, block)?;
        Ok(brillig.obj)
    }

    /// Adds a brillig instruction to the brillig code base
    fn push_code(&mut self, code: BrilligOpcode) {
        self.obj.byte_code.push(code);
    }

    fn code_len(&self) -> usize {
        self.obj.byte_code.len()
    }

    fn get_tmp_register(&mut self) -> RegisterIndex {
        self.max_register += 1;
        RegisterIndex(self.max_register)
    }

    /// handle Phi instructions by adding a mov instruction
    fn handle_phi_instructions(&mut self, current: BlockId, left: BlockId, ctx: &SsaContext) {
        if matches!(ctx[left].kind, BlockType::ForJoin | BlockType::IfJoin) {
            for i in &ctx[left].instructions {
                if let Some(ins) = ctx.try_get_instruction(*i) {
                    match &ins.operation {
                        Operation::Nop => continue,
                        Operation::Phi { root: _, block_args } => {
                            for (id, bid) in block_args {
                                if *bid == current {
                                    let destination = self.node_2_register(ctx, ins.id);
                                    let source = self.node_2_register(ctx, *id);
                                    self.push_code(BrilligOpcode::Mov { destination, source });
                                }
                            }
                        }
                        _ => break,
                    }
                }
            }
        }
    }

    fn process_blocks(&mut self, ctx: &SsaContext, current: BlockId) -> Result<(),RuntimeError> {
        let mut queue = vec![current]; //Stack of elements to visit

        while let Some(current) = queue.pop() {
            let children = self.process_block(ctx, current)?;

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
        Ok(())
    }

    // Generate brillig code from ssa instructions of the block
    fn process_block(&mut self, ctx: &SsaContext, block_id: BlockId) -> Result<Vec<BlockId>, RuntimeError> {
        let block = &ctx[block_id];
        let start = self.obj.byte_code.len();

        //process block instructions, except the last one
        for i in block.instructions.iter().take(block.instructions.len() - 1) {
            let ins = ctx.try_get_instruction(*i).expect("instruction in instructions list");
            self.instruction_to_bc(ctx, ins)?;
        }

        // Jump to the next block
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
                        self.instruction_to_bc(ctx, ins).expect("Could not compile to brillig");
                        None
                    }
                }
            })
            .or_else(|| block.left.map(|left| (BrilligOpcode::JMP { destination: 0 }, left)));
        if let Some(left) = block.left {
            self.handle_phi_instructions(block_id, left, ctx);
        }
        if let Some((jmp, target)) = jump {
            self.obj.to_fix.push((self.code_len(), target));
            self.push_code(jmp);
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
            self.obj.to_fix.push((self.code_len(), BlockId::dummy()));
            self.push_code(BrilligOpcode::JMP { destination: 0 });
        }

        self.obj.blocks.insert(block_id, start);
        Ok(result)
    }

    /// Converts ssa instruction to brillig
    fn instruction_to_bc(&mut self, ctx: &SsaContext, ins: &Instruction) -> Result<(),RuntimeError>{
        match &ins.operation {
            Operation::Binary(bin) => {
                self.binary(ctx, bin, ins.id, ins.res_type);
            }
            Operation::Cast(_) => {
                return Err(RuntimeErrorKind::Unimplemented("Operation not supported in unsafe functions".to_string()).into());
            }
            Operation::Truncate { .. } => unreachable!("Brillig does not require an overflow pass"),
            Operation::Not(_) => todo!(), // bitwise not
            Operation::Constrain(a, _) => {
                let condition = self.node_2_register(ctx, *a);
                self.push_code(BrilligOpcode::JMPIFNOT { condition, destination: 1 });
            }
            Operation::Jne(_, _) | Operation::Jeq(_, _) | Operation::Jmp(_) => {
                unreachable!("a jump can only be at the very end of a block")
            }
            Operation::Phi { .. } => (),
            Operation::Call { .. } => {
                assert!(self.noir_call.is_empty());
                self.noir_call.push(ins.id);
            }
            Operation::Return(ret) => match ret.len() {
                0 => (),
                1 => {
                    if !ret[0].is_dummy() {
                        let ret_register = self.node_2_register(ctx, ret[0]);
                        self.push_code(BrilligOpcode::Mov {
                            destination: RegisterMemIndex::Register(RegisterIndex(0)),
                            source: ret_register,
                        });
                    }
                }
                _ => {
                    for (i, node) in ret.iter().enumerate() {
                        let ret_register = self.node_2_register(ctx, *node);
                        self.push_code(BrilligOpcode::Mov {
                            destination: RegisterMemIndex::Register(RegisterIndex(i)),
                            source: ret_register,
                        });
                    }
                }
            },
            Operation::Result { call_instruction, .. } => {
                assert!(!self.noir_call.is_empty());
                assert_eq!(*call_instruction, self.noir_call[0]);
                self.noir_call.push(ins.id);
                if let Some(call) = ctx.try_get_instruction(*call_instruction) {
                    if let Operation::Call { func, arguments, .. } = &call.operation {
                        if let Some(func_id) = ctx.try_get_func_id(*func) {
                            let ssa_func = ctx.ssa_func(func_id).unwrap();
                            if self.noir_call.len() == ssa_func.result_types.len() + 1 {
                                let returned_values = &self.noir_call[1..];
                                self.unsafe_call(ctx, *func, arguments, &returned_values.to_vec());
                                self.noir_call.clear();
                            }
                        }
                    }
                }
            }
            Operation::Cond { .. } => unreachable!("Brillig does not require the reduction pass"),
            Operation::Load { array_id, index, .. } => {
                let idx_reg = self.node_2_register(ctx, *index);
                let array_id_reg =
                    RegisterMemIndex::Constant(FieldElement::from(array_id.to_u32() as i128));
                let ins_reg = self.node_2_register(ctx, ins.id);
                self.push_code(BrilligOpcode::Load {
                    destination: ins_reg,
                    array_id_reg,
                    index: idx_reg,
                });
            }
            Operation::Store { array_id, index, value, .. } => {
                let idx_reg = self.node_2_register(ctx, *index);
                let array_id_reg =
                    RegisterMemIndex::Constant(FieldElement::from(array_id.to_u32() as i128));
                let source = self.node_2_register(ctx, *value);
                self.push_code(BrilligOpcode::Store { source, array_id_reg, index: idx_reg });
            }
            Operation::Intrinsic(_, _) => {
                todo!();
            }
            Operation::UnsafeCall { func, arguments, returned_values, .. } => {
                self.unsafe_call(ctx, *func, arguments, returned_values)
            }
            Operation::Nop => (),
        }
        Ok(())
    }

    fn node_2_register(&mut self, ctx: &SsaContext, a: NodeId) -> RegisterMemIndex //register-value enum
    {
        let a_register = a.0.into_raw_parts().0;
        assert_ne!(a_register, CALLBACK_REGISTER);
        match &ctx[a] {
            NodeObject::Variable(_) => {
                if a_register > self.max_register {
                    self.max_register = a_register;
                }
                let reg_node = RegisterMemIndex::Register(RegisterIndex(a_register));
                if let Some(array) = Memory::deref(ctx, a) {
                    self.push_code(BrilligOpcode::Mov {
                        destination: reg_node,
                        source: RegisterMemIndex::Constant(FieldElement::from(
                            array.to_u32() as i128
                        )),
                    });
                }
                reg_node
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
            self.push_code(BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Add,
                result,
            });
        }
        BinaryOp::SafeAdd => todo!(),
        BinaryOp::Sub { .. } => self.push_code(BrilligOpcode::BinaryOp {
            lhs,
            rhs,
            result_type,
            op: brillig_bytecode::BinaryOp::Sub,
            result,
        }),
        BinaryOp::SafeSub { .. } => todo!(),
        BinaryOp::Mul => self.push_code(BrilligOpcode::BinaryOp {
            lhs,
            rhs,
            result_type,
            op: brillig_bytecode::BinaryOp::Mul,
            result,
        }),
        BinaryOp::SafeMul => todo!(),
        BinaryOp::Urem(_) => {
            let q = self.get_tmp_register();
            self.push_code(BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Div,
                result:q,
            });
            self.push_code(BrilligOpcode::BinaryOp {
                result_type,
                lhs: RegisterMemIndex::Register(q),
                rhs,
                op: brillig_bytecode::BinaryOp::Mul,
                result: q,
            });
            self.push_code(BrilligOpcode::BinaryOp { result_type, op: brillig_bytecode::BinaryOp::Sub, lhs, rhs: RegisterMemIndex::Register(q), result });
        }
        BinaryOp::Srem(_) => todo!(),
        BinaryOp::Udiv(_) |
        BinaryOp::Sdiv(_) |
        BinaryOp::Div(_) => {
            self.push_code(BrilligOpcode::BinaryOp {
                lhs,
                rhs,
                result_type,
                op: brillig_bytecode::BinaryOp::Div,
                result,
            });
        },
        BinaryOp::Eq => {
            self.push_code(BrilligOpcode::BinaryOp { result_type: BrilligType::Unsigned { bit_size: 1 }, op: brillig_bytecode::BinaryOp::Cmp(brillig_bytecode::Comparison::Eq
        ), lhs, rhs, result});
        }, //a==b => is_zero()
        BinaryOp::Ne =>
     {
        self.push_code(BrilligOpcode::BinaryOp { result_type: BrilligType::Unsigned { bit_size: 1 }, op: brillig_bytecode::BinaryOp::Cmp(brillig_bytecode::Comparison::Eq
        ), lhs, rhs, result});
        self.push_code(
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
            self.push_code(BrilligOpcode::BinaryOp { result_type, op: brillig_bytecode::BinaryOp::Sub,
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

    fn get_oracle_abi(&mut self, ctx: &SsaContext, funct: &SsaFunction, arguments:  &Vec<NodeId>,) -> Vec<OracleInput> {
        let mut abi = Vec::new();
        for (param, arg) in funct.arguments.iter().zip(arguments) {
            let len = if let Some(a) = Memory::deref(ctx, param.0) { ctx.mem[a].len } else { 0 };
            abi.push(OracleInput {
                register_mem_index: self.node_2_register(ctx, *arg),
                length: len as usize,
            })
        }
        // for i in &funct.result_types {
        //     let len = if let ObjectType::ArrayPointer(a) = i { ctx.mem[*a].len } else { 0 };
        //     signature.outputs.push(len);
        // }
        abi
    }

    fn unsafe_call(
        &mut self,
        ctx: &SsaContext,
        func: NodeId,
        arguments: &Vec<NodeId>,
        returned_values: &Vec<NodeId>,
    ) {
        if let Some(func_id) = ctx.try_get_func_id(func) {
            let ssa_func = ctx.ssa_func(func_id).unwrap();
            match ssa_func.kind.clone() {
                RuntimeType::Oracle(name) => {
                    let mut outputs = Vec::new();
                    for i in returned_values {
                        outputs.push(self.node_2_register(ctx, *i).to_register_index().unwrap());
                    }
                    let abi = self.get_oracle_abi(ctx, ssa_func, arguments);
                    self.push_code(brillig_bytecode::Opcode::Oracle(OracleData {
                        name,
                        inputs: abi,
                        input_values: Vec::new(),
                        output: outputs[0], //TODO: temp  
                        output_values: Vec::new(),
                    }));
                 
                }
                RuntimeType::Unsafe | RuntimeType::Acvm => {
                    // we need to have a place for the functions
                    let func_adr =
                        if let Some(func_adr) = self.functions.get(&func) { *func_adr } else { 0 };
                    //mov inputs to function arguments:
                    for (input, arg) in ssa_func.arguments.iter().zip(arguments) {
                        let arg_reg = self.node_2_register(ctx, *arg);
                        let in_reg = self.node_2_register(ctx, input.0);
                        self.push_code(brillig_bytecode::Opcode::Mov {
                            destination: in_reg,
                            source: arg_reg,
                        });
                    }
                    self.obj.to_fix.push((self.code_len(), BlockId::dummy()));
                    self.push_code(brillig_bytecode::Opcode::Mov {
                        destination: RegisterMemIndex::Register(RegisterIndex(CALLBACK_REGISTER)),
                        source: RegisterMemIndex::Constant(FieldElement::zero()),
                    });

                    if func_adr == 0 {
                        self.obj.to_fix.push((self.code_len(), ssa_func.entry_block));
                        self.obj.functions_to_process.insert(func);
                    }
                    self.push_code(brillig_bytecode::Opcode::JMP { destination: func_adr });
                    // reset the call_back
                    self.push_code(brillig_bytecode::Opcode::Mov {
                        destination: RegisterMemIndex::Register(RegisterIndex(CALLBACK_REGISTER)),
                        source: RegisterMemIndex::Constant(FieldElement::from(
                            (PREFIX_LEN - 1) as i128,
                        )),
                    });
                    //result is in register 0
                    if returned_values.len() == 1 {
                        let first = self.node_2_register(ctx, *returned_values.first().unwrap());
                        self.push_code(brillig_bytecode::Opcode::Mov {
                            destination: first,
                            source: RegisterMemIndex::Register(RegisterIndex(0)),
                        });
                    }
                }
            }
        }
    }
}

fn object_type_2_typ(object_type: ObjectType) -> BrilligType {
    match object_type {
        ObjectType::Numeric(NumericType::NativeField) => BrilligType::Field,
        ObjectType::Numeric(NumericType::Unsigned(s)) => BrilligType::Unsigned { bit_size: s },
        ObjectType::Numeric(NumericType::Signed(s)) => BrilligType::Signed { bit_size: s },
        ObjectType::ArrayPointer(_) => todo!(),
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
