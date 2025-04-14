use super::{
    Ssa,
    ir::{
        dfg::DataFlowGraph,
        function::{Function, FunctionId, RuntimeType},
        instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
        value::ValueId,
    },
};
use crate::errors::RuntimeError;
use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;
use iter_extended::vecmap;
use value::NumericValue;

mod tests;
pub mod value;

use value::Value;

struct Interpreter<'ssa> {
    /// Contains each function called with `main` (or the first called function if
    /// the interpreter was manually invoked on a different function) at
    /// the front of the Vec.
    call_stack: Vec<CallContext>,

    ssa: &'ssa Ssa,

    /// This variable can be modified by `enable_side_effects_if` instructions and is
    /// expected to have no effect if there are no such instructions or if the code
    /// being executed is an unconstrained function.
    side_effects_enabled: bool,
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

type IResult = Result<Value, RuntimeError>;
type IResults = Result<Vec<Value>, RuntimeError>;

#[allow(unused)]
pub(crate) fn interpret(ssa: &Ssa) -> IResults {
    interpret_function(ssa, ssa.main_id)
}

pub(crate) fn interpret_function(ssa: &Ssa, function: FunctionId) -> IResults {
    let mut interpreter = Interpreter::new(ssa);
    interpreter.call_function(function, Vec::new())
}

impl<'ssa> Interpreter<'ssa> {
    fn new(ssa: &'ssa Ssa) -> Self {
        Self { ssa, call_stack: Vec::new(), side_effects_enabled: true }
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

    fn call_function(&mut self, function_id: FunctionId, mut arguments: Vec<Value>) -> IResults {
        self.call_stack.push(CallContext::new(function_id));

        let function = &self.ssa.functions[&function_id];
        let mut block_id = function.entry_block();
        let dfg = self.dfg();

        // Loop over blocks & instructions inline here to avoid pushing more
        // call frames (in rust). We only push call frames for function calls which
        // should prevent stack overflows for all but excessively large call stacks
        // that may overflow in the brillig vm as well.
        let return_values = loop {
            let block = &dfg[block_id];

            if arguments.len() != block.parameters().len() {
                todo!("Block argument count does not match the expected parameter count");
            }

            for (parameter, argument) in block.parameters().iter().zip(arguments) {
                self.define(*parameter, argument);
            }

            for instruction_id in block.instructions() {
                let results = dfg.instruction_results(*instruction_id);
                self.interpret_instruction(&dfg[*instruction_id], results)?;
            }

            match block.terminator() {
                None => todo!("No terminator"),
                Some(TerminatorInstruction::Jmp { destination, arguments: jump_args, .. }) => {
                    block_id = *destination;
                    arguments = self.lookup_all(jump_args);
                }
                Some(TerminatorInstruction::JmpIf {
                    condition,
                    then_destination,
                    else_destination,
                    call_stack: _,
                }) => {
                    block_id = if self.lookup(*condition).as_bool().unwrap() {
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

        match &self.dfg()[id] {
            super::ir::value::Value::Instruction { .. } => self.call_context().scope[&id].clone(),
            super::ir::value::Value::Param { .. } => self.call_context().scope[&id].clone(),
            super::ir::value::Value::NumericConstant { constant, typ } => {
                Value::from_constant(*constant, *typ)
            }
            super::ir::value::Value::Function(id) => Value::Function(*id),
            super::ir::value::Value::Intrinsic(intrinsic) => Value::Intrinsic(*intrinsic),
            super::ir::value::Value::ForeignFunction(name) => Value::ForeignFunction(name.clone()),
            super::ir::value::Value::Global(_) => todo!("ssa globals"),
        }
    }

    fn lookup_all(&self, ids: &[ValueId]) -> Vec<Value> {
        vecmap(ids, |id| self.lookup(*id))
    }

    fn side_effects_enabled(&self) -> bool {
        match self.current_function().runtime() {
            RuntimeType::Acir(_) => self.side_effects_enabled,
            RuntimeType::Brillig(_) => true,
        }
    }

    #[allow(unused)]
    fn interpret_instruction(
        &mut self,
        instruction: &Instruction,
        results: &[ValueId],
    ) -> Result<(), RuntimeError> {
        match instruction {
            Instruction::Binary(binary) => {
                let result = self.interpret_binary(binary)?;
                self.define(results[0], result);
            }
            // Cast in SSA changes the type without altering the value
            Instruction::Cast(value, numeric_type) => {
                let field = self.lookup(*value).as_numeric().unwrap().convert_to_field();
                let result = Value::Numeric(NumericValue::from_constant(field, *numeric_type));
                self.define(results[0], result);
            }
            Instruction::Not(id) => self.interpret_not(id, results[0]),
            Instruction::Truncate { value, bit_size, max_bit_size } => {
                self.interpret_truncate(*value, *bit_size, *max_bit_size, results[0])
            }
            Instruction::Constrain(lhs, rhs, constrain_error) => {
                let lhs = self.lookup(*lhs);
                let rhs = self.lookup(*rhs);
                if self.side_effects_enabled() && lhs != rhs {
                    panic!("Constrain {lhs} == {rhs} failed!");
                }
            }
            Instruction::ConstrainNotEqual(lhs, rhs, constrain_error) => {
                let lhs = self.lookup(*lhs);
                let rhs = self.lookup(*rhs);
                if self.side_effects_enabled() && lhs == rhs {
                    panic!("Constrain {lhs} != {rhs} failed!");
                }
            }
            Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                self.interpret_range_check(*value, *max_bit_size, assert_message.as_ref())
            }
            Instruction::Call { func, arguments } => self.interpret_call(*func, arguments, results),
            Instruction::Allocate => self.interpret_allocate(results[0]),
            Instruction::Load { address } => self.interpret_load(*address, results[0]),
            Instruction::Store { address, value } => self.interpret_store(*address, *value),
            Instruction::EnableSideEffectsIf { condition } => {
                self.side_effects_enabled = self.lookup(*condition).as_bool().unwrap();
            }
            Instruction::ArrayGet { array, index } => {
                self.interpret_array_get(*array, *index, results[0])
            }
            Instruction::ArraySet { array, index, value, mutable } => {
                self.interpret_array_set(*array, *index, *value, *mutable, results[0])
            }
            Instruction::IncrementRc { value } => self.interpret_inc_rc(*value),
            Instruction::DecrementRc { value } => self.interpret_dec_rc(*value),
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => self
                .interpret_if_else(
                    *then_condition,
                    *then_value,
                    *else_condition,
                    *else_condition,
                    results[0],
                ),
            Instruction::MakeArray { elements, typ } => {
                self.interpret_make_array(elements, results[0])
            }
            Instruction::Noop => (),
        }
        Ok(())
    }
}

macro_rules! apply_int_binop {
    ($lhs:expr, $rhs:expr, $f:expr) => {{
        use value::NumericValue::*;
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => panic!("Expected only integer values, found field values"),
            (U1(_), U1(_)) => panic!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U8($f(&lhs, &rhs)),
            (U16(lhs), U16(rhs)) => U16($f(&lhs, &rhs)),
            (U32(lhs), U32(rhs)) => U32($f(&lhs, &rhs)),
            (U64(lhs), U64(rhs)) => U64($f(&lhs, &rhs)),
            (U128(lhs), U128(rhs)) => U128($f(&lhs, &rhs)),
            (I8(lhs), I8(rhs)) => I8($f(&lhs, &rhs)),
            (I16(lhs), I16(rhs)) => I16($f(&lhs, &rhs)),
            (I32(lhs), I32(rhs)) => I32($f(&lhs, &rhs)),
            (I64(lhs), I64(rhs)) => I64($f(&lhs, &rhs)),
            (lhs, rhs) => panic!("Got mismatched types in binop: {lhs:?} and {rhs:?}"),
        }
    }};
}

macro_rules! apply_int_binop_opt {
    ($lhs:expr, $rhs:expr, $f:expr) => {{
        use value::NumericValue::*;
        // TODO: Error if None
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => panic!("Expected only integer values, found field values"),
            (U1(_), U1(_)) => panic!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U8($f(&lhs, &rhs).unwrap()),
            (U16(lhs), U16(rhs)) => U16($f(&lhs, &rhs).unwrap()),
            (U32(lhs), U32(rhs)) => U32($f(&lhs, &rhs).unwrap()),
            (U64(lhs), U64(rhs)) => U64($f(&lhs, &rhs).unwrap()),
            (U128(lhs), U128(rhs)) => U128($f(&lhs, &rhs).unwrap()),
            (I8(lhs), I8(rhs)) => I8($f(&lhs, &rhs).unwrap()),
            (I16(lhs), I16(rhs)) => I16($f(&lhs, &rhs).unwrap()),
            (I32(lhs), I32(rhs)) => I32($f(&lhs, &rhs).unwrap()),
            (I64(lhs), I64(rhs)) => I64($f(&lhs, &rhs).unwrap()),
            (lhs, rhs) => panic!("Got mismatched types in binop: {lhs:?} and {rhs:?}"),
        }
    }};
}

macro_rules! apply_int_comparison_op {
    ($lhs:expr, $rhs:expr, $f:expr) => {{
        use NumericValue::*;
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => panic!("Expected only integer values, found field values"),
            (U1(_), U1(_)) => panic!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U1($f(&lhs, &rhs)),
            (U16(lhs), U16(rhs)) => U1($f(&lhs, &rhs)),
            (U32(lhs), U32(rhs)) => U1($f(&lhs, &rhs)),
            (U64(lhs), U64(rhs)) => U1($f(&lhs, &rhs)),
            (U128(lhs), U128(rhs)) => U1($f(&lhs, &rhs)),
            (I8(lhs), I8(rhs)) => U1($f(&lhs, &rhs)),
            (I16(lhs), I16(rhs)) => U1($f(&lhs, &rhs)),
            (I32(lhs), I32(rhs)) => U1($f(&lhs, &rhs)),
            (I64(lhs), I64(rhs)) => U1($f(&lhs, &rhs)),
            (lhs, rhs) => panic!("Got mismatched types in binop: {lhs:?} and {rhs:?}"),
        }
    }};
}

impl<'ssa> Interpreter<'ssa> {
    fn interpret_binary(&mut self, binary: &Binary) -> IResult {
        // TODO: Replace unwrap with real error
        let lhs = self.lookup(binary.lhs).as_numeric().unwrap();
        let rhs = self.lookup(binary.rhs).as_numeric().unwrap();

        if lhs.get_type() != rhs.get_type()
            && !matches!(binary.operator, BinaryOp::Shl | BinaryOp::Shr)
        {
            todo!("Type error!")
        }

        // Disable this instruction if it is side-effectful and side effects are disabled.
        if !self.side_effects_enabled() && binary.requires_acir_gen_predicate(self.dfg()) {
            let zero = NumericValue::from_constant(FieldElement::zero(), lhs.get_type());
            return Ok(Value::Numeric(zero));
        }

        if let (Some(lhs), Some(rhs)) = (lhs.as_field(), rhs.as_field()) {
            return self.interpret_field_binary_op(lhs, binary.operator, rhs);
        }

        if let (Some(lhs), Some(rhs)) = (lhs.as_bool(), rhs.as_bool()) {
            return self.interpret_u1_binary_op(lhs, binary.operator, rhs);
        }

        let result = match binary.operator {
            BinaryOp::Add { unchecked: false } => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedAdd::checked_add)
            }
            BinaryOp::Add { unchecked: true } => {
                apply_int_binop!(lhs, rhs, num_traits::WrappingAdd::wrapping_add)
            }
            BinaryOp::Sub { unchecked: false } => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedSub::checked_sub)
            }
            BinaryOp::Sub { unchecked: true } => {
                apply_int_binop!(lhs, rhs, num_traits::WrappingSub::wrapping_sub)
            }
            BinaryOp::Mul { unchecked: false } => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedMul::checked_mul)
            }
            BinaryOp::Mul { unchecked: true } => {
                apply_int_binop!(lhs, rhs, num_traits::WrappingMul::wrapping_mul)
            }
            BinaryOp::Div => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedDiv::checked_div)
            }
            BinaryOp::Mod => {
                apply_int_binop_opt!(lhs, rhs, num_traits::CheckedRem::checked_rem)
            }
            BinaryOp::Eq => apply_int_comparison_op!(lhs, rhs, |a, b| a == b),
            BinaryOp::Lt => apply_int_comparison_op!(lhs, rhs, |a, b| a < b),
            BinaryOp::And => todo!(),
            BinaryOp::Or => todo!(),
            BinaryOp::Xor => todo!(),
            BinaryOp::Shl => todo!(),
            BinaryOp::Shr => todo!(),
        };
        Ok(Value::Numeric(result))
    }

    fn interpret_field_binary_op(
        &mut self,
        lhs: FieldElement,
        operator: BinaryOp,
        rhs: FieldElement,
    ) -> IResult {
        let result = match operator {
            BinaryOp::Add { unchecked: _ } => NumericValue::Field(lhs + rhs),
            BinaryOp::Sub { unchecked: _ } => NumericValue::Field(lhs - rhs),
            BinaryOp::Mul { unchecked: _ } => NumericValue::Field(lhs * rhs),
            BinaryOp::Div => NumericValue::Field(lhs / rhs),
            BinaryOp::Mod => panic!("Unsupported operator `%` for Field"),
            BinaryOp::Eq => NumericValue::U1(lhs == rhs),
            BinaryOp::Lt => NumericValue::U1(lhs < rhs),
            BinaryOp::And => panic!("Unsupported operator `&` for Field"),
            BinaryOp::Or => panic!("Unsupported operator `|` for Field"),
            BinaryOp::Xor => panic!("Unsupported operator `^` for Field"),
            BinaryOp::Shl => panic!("Unsupported operator `<<` for Field"),
            BinaryOp::Shr => panic!("Unsupported operator `>>` for Field"),
        };
        Ok(Value::Numeric(result))
    }

    fn interpret_u1_binary_op(&mut self, lhs: bool, operator: BinaryOp, rhs: bool) -> IResult {
        let result = match operator {
            BinaryOp::Add { unchecked: _ } => panic!("Unsupported operator `+` for u1"),
            BinaryOp::Sub { unchecked: _ } => panic!("Unsupported operator `-` for u1"),
            BinaryOp::Mul { unchecked: _ } => lhs & rhs, // (*) = (&) for u1
            BinaryOp::Div => todo!(),
            BinaryOp::Mod => todo!(),
            BinaryOp::Eq => lhs == rhs,
            BinaryOp::Lt => lhs < rhs,
            BinaryOp::And => lhs & rhs,
            BinaryOp::Or => lhs | rhs,
            BinaryOp::Xor => lhs ^ rhs,
            BinaryOp::Shl => panic!("Unsupported operator `<<` for u1"),
            BinaryOp::Shr => panic!("Unsupported operator `>>` for u1"),
        };
        Ok(Value::Numeric(NumericValue::U1(result)))
    }
}
