use super::{
    Ssa,
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::{Function, FunctionId},
        instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
        value::ValueId,
    },
};
use crate::errors::RuntimeError;
use fxhash::FxHashMap as HashMap;
use iter_extended::vecmap;
pub use value::Value;

mod tests;
mod value;

struct Interpreter<'ssa> {
    /// Contains each function called with `main` (or the first called function if
    /// the interpreter was manually invoked on a different function) at
    /// the front of the Vec.
    call_stack: Vec<CallContext>,

    ssa: &'ssa Ssa,
}

struct CallContext {
    called_function: FunctionId,

    /// Contains each value currently defined and visible to the current function.
    scope: HashMap<ValueId, Value>,
}

impl CallContext {
    fn new(called_function: FunctionId) -> Self {
        Self { called_function, scope: Default::default() }
    }
}

type IResult = Result<Vec<Value>, RuntimeError>;

pub fn interpret(ssa: &Ssa) -> IResult {
    interpret_function(ssa, ssa.main_id)
}

pub fn interpret_function(ssa: &Ssa, function: FunctionId) -> IResult {
    let mut interpreter = Interpreter::new(ssa);
    interpreter.call_function(function, Vec::new())
}

impl<'ssa> Interpreter<'ssa> {
    fn new(ssa: &'ssa Ssa) -> Self {
        Self { ssa, call_stack: Vec::new() }
    }

    fn call_context(&self) -> &CallContext {
        self.call_stack.last().expect("Expected SSA Interpreter to be executing a function")
    }

    fn call_context_mut(&mut self) -> &mut CallContext {
        self.call_stack.last_mut().expect("Expected SSA Interpreter to be executing a function")
    }

    fn current_function(&self) -> &'ssa Function {
        let current_function_id = self.call_context().called_function;
        &self.ssa.functions[&current_function_id]
    }

    fn dfg(&self) -> &'ssa DataFlowGraph {
        &self.current_function().dfg
    }

    /// Define or redefine a value.
    /// Redefinitions are expected in the case of loops.
    fn define(&mut self, id: ValueId, value: Value) {
        self.call_context_mut().scope.insert(id, value);
    }

    fn call_function(&mut self, function_id: FunctionId, mut arguments: Vec<Value>) -> IResult {
        self.call_stack.push(CallContext::new(function_id));

        let function = &self.ssa.functions[&function_id];
        let mut block_id = function.entry_block();
        let dfg = self.dfg();

        // Loop over blocks & instructions inline here to avoid pushing more
        // call frames (in rust). We only push call frames for function calls which
        // should be fine.
        let return_values = loop {
            let block = &dfg[block_id];

            if arguments.len() != block.parameters().len() {
                todo!("Block argument count does not match the expected parameter count");
            }

            for (parameter, argument) in block.parameters().iter().zip(arguments) {
                self.define(*parameter, argument);
            }

            for instruction_id in block.instructions() {
                self.interpret_instruction(&dfg[*instruction_id]);
            }

            match block.terminator() {
                None => todo!("No terminator"),
                Some(TerminatorInstruction::Jmp { destination, arguments, call_stack: _ }) => {
                    block_id = *destination;
                    arguments = self.lookup_all(arguments)?;
                }
                Some(TerminatorInstruction::JmpIf {
                    condition,
                    then_destination,
                    else_destination,
                    call_stack: _,
                }) => {
                    block_id = if self.lookup(*condition)?.as_bool().unwrap() {
                        *then_destination
                    } else {
                        *else_destination
                    };
                    arguments = Vec::new();
                }
                Some(TerminatorInstruction::Return { return_values, call_stack: _ }) => {
                    break self.lookup_all(return_values);
                }
            }
        };

        self.call_stack.pop();
        Ok(return_values)
    }

    fn lookup(&self, id: ValueId) -> Value {
        let id = self.dfg().resolve(id);
        self.call_context().scope[id].clone()
    }

    fn lookup_all(&self, ids: &[ValueId]) -> Vec<Value> {
        vecmap(ids, |id| self.lookup(id))
    }

    fn interpret_instruction(&mut self, instruction: &Instruction) -> IResult {
        match instruction {
            Instruction::Binary(binary) => self.interpret_binary(binary),
            Instruction::Cast(id, numeric_type) => todo!(),
            Instruction::Not(id) => todo!(),
            Instruction::Truncate { value, bit_size, max_bit_size } => todo!(),
            Instruction::Constrain(id, id1, constrain_error) => todo!(),
            Instruction::ConstrainNotEqual(id, id1, constrain_error) => todo!(),
            Instruction::RangeCheck { value, max_bit_size, assert_message } => todo!(),
            Instruction::Call { func, arguments } => todo!(),
            Instruction::Allocate => todo!(),
            Instruction::Load { address } => todo!(),
            Instruction::Store { address, value } => todo!(),
            Instruction::EnableSideEffectsIf { condition } => todo!(),
            Instruction::ArrayGet { array, index } => todo!(),
            Instruction::ArraySet { array, index, value, mutable } => todo!(),
            Instruction::IncrementRc { value } => todo!(),
            Instruction::DecrementRc { value } => todo!(),
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
                todo!()
            }
            Instruction::MakeArray { elements, typ } => todo!(),
            Instruction::Noop => todo!(),
        }
    }

    fn interpret_binary(&mut self, binary: &Binary) -> IResult {
        // TODO: Replace unwrap with real error
        let lhs = self.lookup(binary.lhs).as_numeric().unwrap();
        let rhs = self.lookup(binary.rhs).as_numeric().unwrap();

        if lhs.get_type() != rhs.get_type()
            && !matches!(binary.operator, Binaryop::Shl | BinaryOp::Shr)
        {
            todo!("Type error!")
        }

        match binary.operator {
            BinaryOp::Add { unchecked: false } => {
                apply_binop!(lhs, rhs, num_traits::CheckedAdd::checked_add)
            }
            BinaryOp::Add { unchecked: true } => {
                apply_binop!(lhs, rhs, num_traits::WrappingAdd::wrapping_add)
            }
            BinaryOp::Sub { unchecked: false } => {
                apply_binop!(lhs, rhs, num_traits::CheckedSub::checked_sub)
            }
            BinaryOp::Sub { unchecked: true } => {
                apply_binop!(lhs, rhs, num_traits::WrappingSub::wrapping_sub)
            }
            BinaryOp::Mul { unchecked: false } => {
                apply_binop!(lhs, rhs, num_traits::CheckedMul::checked_mul)
            }
            BinaryOp::Mul { unchecked: true } => {
                apply_binop!(lhs, rhs, num_traits::WrappingMul::wrapping_mul)
            }
            BinaryOp::Div => todo!(),
            BinaryOp::Mod => todo!(),
            BinaryOp::Eq => todo!(),
            BinaryOp::Lt => todo!(),
            BinaryOp::And => todo!(),
            BinaryOp::Or => todo!(),
            BinaryOp::Xor => todo!(),
            BinaryOp::Shl => todo!(),
            BinaryOp::Shr => todo!(),
        }
    }
}
