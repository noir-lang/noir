use std::collections::HashMap;

use crate::{ssa_refactor::{ir::{basic_block::BasicBlockId, function::Function, instruction::{Instruction, Binary, BinaryOp, Instruction::Cast, Instruction::Not, InstructionId, TerminatorInstruction, InstructionResultType}, value::{ValueId, Value}, types::{Type, NumericType}, post_order::PostOrder}, ssa_gen::SharedContext}, errors::RuntimeError};

use crate::ssa_refactor::ir::types::Type::Numeric;

use super::artefact::Artefact;

//use acvm::acir::brillig_bytecode::{self, OracleInput, OracleOutput};
use acvm::acir::brillig_bytecode::{ self,
    Opcode as BrilligOpcode, RegisterIndex, Typ as BrilligType,//OracleData, RegisterMemIndex
    Value as BrilligValue,
};

#[derive(Default)]
pub(crate) struct BrilligGen {
    obj: Artefact,
    max_register: usize,
    functions: HashMap<ValueId, usize>,
}

impl BrilligGen {
    /// Generate compilation object from ssa code
    pub(crate) fn compile(
        func: &Function,
    ) -> Result<Artefact, RuntimeError> {
        let mut brillig = BrilligGen::default();
        brillig.process_blocks(func)?;
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
        //TODO make it safe
        self.max_register += 1;
        RegisterIndex(self.max_register)
    }

    fn process_blocks(&mut self, func: &Function) -> Result<(), RuntimeError> {
        let mut rpo = PostOrder::with_function(func).as_slice();
        rpo.reverse();
        for b in rpo {
            self.process_block(func, *b);
        }
       Ok(())
    }
     // Generate brillig code from ssa instructions of the block
     fn process_block(
        &mut self,
        func: &Function,
        block_id: BasicBlockId,
    ) -> Result<(), RuntimeError> {
        let block = &func.dfg[block_id];
        self.obj.start(block_id);
        //process block instructions, except the last one
        for i in block.instructions() {
            let ins = &func.dfg[*i];
            self.to_byte_code(func, ins, *i);
        }
        
        // Jump to the next block
        let jump = block.terminator().expect("block is expected to be constructed");
        match jump {
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
                let condition = self.node_2_register(func, *condition);
                self.jump_if(condition, *then_destination);
                self.jump(*else_destination);
            }
            TerminatorInstruction::Jmp { destination, arguments } => {
                let target = func.dfg[*destination];
                for (src,dest) in arguments.iter().zip(target.parameters()) {
                    let destination = self.node_2_register(func, *dest);
                    let source = self.node_2_register(func, *src);
                    self.push_code(BrilligOpcode::Mov { destination, source });
                }
                self.jump(*destination);
            }
            //TODO return values
            TerminatorInstruction::Return { return_values } => self.push_code(BrilligOpcode::Return),
        }
  
        Ok(())
    }

    fn jump(&mut self, target: BasicBlockId) {
        self.obj.fix_jump(target);
        self.push_code(BrilligOpcode::Jump { location: 0 });
    }

    fn jump_if(&mut self, condition: RegisterIndex, target: BasicBlockId) {
        self.obj.fix_jump(target);
        self.push_code(BrilligOpcode::JumpIf { condition, location: 0 });
    }

     /// Converts ssa instruction to brillig
     fn to_byte_code(
        &mut self,
        func: &Function,
        ins: &Instruction,
        id: InstructionId,
    ) //-> Result<(), RuntimeError> 
    {
        let res_type = get_instruction_type(func,ins);
        match ins {
            Instruction::Binary(bin) => self.binary(func, bin, &id, res_type),
            Cast(v, t) => todo!(),
            Not(v) => todo!(),
            Instruction::Truncate { value, bit_size, max_bit_size } => todo!(),
            Instruction::Constrain(v) =>  {
                let condition = self.node_2_register(func, *v);
                self.push_code(BrilligOpcode::JumpIfNot { condition, location: 1 });
            },
            Instruction::Call { func: call, arguments } => {
                todo!();
                //self.unsafe_call(ctx, func, *call , arguments);//, returned_values)
            },      
            Instruction::Allocate { size: u32 } => todo!(),
            Instruction::Load { address } => todo!(),
            Instruction::Store { address, value} => todo!(),
        }
       
    }


    fn binary(&mut self, func: &Function, bin: &Binary, id: &InstructionId, result_type: BrilligType, ) {
        let lhs = self.node_2_register(func, bin.lhs);
        let rhs = self.node_2_register(func, bin.rhs);
        let result = self.instruction_2_register(func, id);
        let is_field = result_type == BrilligType::Field;
        let bit_size = match result_type {
            BrilligType::Field => 0,
            BrilligType::Unsigned { bit_size }
            | BrilligType::Signed { bit_size } => bit_size,
        };

        match &bin.operator {
            BinaryOp::Add => {
                if bit_size == 0 {
                    self.push_code(BrilligOpcode::BinaryFieldOp {
                        lhs,
                        rhs,
                        op: brillig_bytecode::BinaryFieldOp::Add,
                        destination: result,
                    });
                } else {
                    self.push_code(BrilligOpcode::BinaryIntOp {
                        lhs,
                        rhs,
                        bit_size,
                        op: brillig_bytecode::BinaryIntOp::Add,
                        destination: result,
                    });
                }
            },
            _ => todo!(),
        }
    }

    //Basic register allocation assuming infinite registers
    fn node_2_register(&mut self, func: &Function, v: ValueId) -> RegisterIndex
    {
        match &func.dfg[v] {
            Value::Instruction { instruction, position, typ: Type } => {
                assert_eq!(*position, 0);
                let reg = self.instruction_2_register(func, instruction);
                reg
            }
            Value::Param { block: BasicBlockId, position: usize, typ: Type } => todo!(),
            Value::NumericConstant { constant, .. } => {
                let destination = self.get_tmp_register();
                self.push_code(BrilligOpcode::Const{ destination, value: func.dfg[*constant].value().into()});
                destination
            } 
            Value::Function(FunctionId) => todo!(),
            Value::Intrinsic(Intrinsic) => todo!(),
        }
    }

    fn instruction_2_register(&mut self, func: &Function, ins_id: &InstructionId) -> RegisterIndex {
        let reg = ins_id.as_usize() + func.id().as_usize();
        let reg = reg*(reg+1)/2+func.id().as_usize();
        if reg > self.max_register {
            self.max_register = reg;
        }
        RegisterIndex(reg)
    }

}


fn get_instruction_type(func: &Function, ins: &Instruction) -> BrilligType {
    let res_type = ins.result_type();
    let typ = match res_type {
        InstructionResultType::Operand(v) => func.dfg.type_of_value(v),
        InstructionResultType::Known(t) => t,
        InstructionResultType::Unknown
        | InstructionResultType::None => unreachable!(),
    };
    match typ {
        Numeric(NumericType::NativeField) => BrilligType::Field,
        Numeric(NumericType::Unsigned{ bit_size }) => BrilligType::Unsigned { bit_size },
        Numeric(NumericType::Signed{ bit_size }) => BrilligType::Signed { bit_size },
        Reference => todo!(),
        Function => todo!(),
        Unit => todo!(),
    }
} 
