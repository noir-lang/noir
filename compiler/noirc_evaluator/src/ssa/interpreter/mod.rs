use std::{cmp::Ordering, io::Write};

use super::{
    Ssa,
    ir::{
        dfg::DataFlowGraph,
        function::{Function, FunctionId, RuntimeType},
        instruction::{
            ArrayOffset, Binary, BinaryOp, ConstrainError, Instruction, TerminatorInstruction,
        },
        types::Type,
        value::ValueId,
    },
};
use crate::ssa::ir::{instruction::binary::truncate_field, printer::display_binary};
use acvm::{AcirField, FieldElement};
use errors::{InternalError, InterpreterError, MAX_UNSIGNED_BIT_SIZE};
use fxhash::FxHashMap as HashMap;
use iter_extended::{try_vecmap, vecmap};
use noirc_frontend::Shared;
use value::{ArrayValue, NumericValue, ReferenceValue};

pub mod errors;
mod intrinsics;
pub(crate) mod tests;
pub mod value;

use value::Value;

struct Interpreter<'ssa, W> {
    /// Contains each function called with `main` (or the first called function if
    /// the interpreter was manually invoked on a different function) at
    /// the front of the Vec.
    call_stack: Vec<CallContext>,

    ssa: &'ssa Ssa,

    /// This variable can be modified by `enable_side_effects_if` instructions and is
    /// expected to have no effect if there are no such instructions or if the code
    /// being executed is an unconstrained function.
    side_effects_enabled: bool,

    options: InterpreterOptions,
    /// Print output.
    output: W,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InterpreterOptions {
    /// If true, the interpreter will trace its execution.
    pub trace: bool,
}

struct CallContext {
    /// The function that was called. This is `None` only for the top-level global
    /// scope where global instructions are evaluated.
    called_function: Option<FunctionId>,

    /// Contains each value currently defined and visible to the current function.
    scope: HashMap<ValueId, Value>,
}

impl CallContext {
    fn new(called_function: FunctionId) -> Self {
        Self { called_function: Some(called_function), scope: Default::default() }
    }

    fn global_context() -> Self {
        Self { called_function: None, scope: Default::default() }
    }
}

type IResult<T> = Result<T, InterpreterError>;
type IResults = IResult<Vec<Value>>;

#[allow(unused)]
impl Ssa {
    pub fn interpret(&self, args: Vec<Value>) -> IResults {
        self.interpret_with_options(args, InterpreterOptions::default(), std::io::empty())
    }

    pub fn interpret_with_options<W: Write>(
        &self,
        args: Vec<Value>,
        options: InterpreterOptions,
        output: W,
    ) -> IResults {
        self.interpret_function(self.main_id, args, options, output)
    }

    fn interpret_function<W: Write>(
        &self,
        function: FunctionId,
        args: Vec<Value>,
        options: InterpreterOptions,
        output: W,
    ) -> IResults {
        let mut interpreter = Interpreter::new(self, options, output);
        interpreter.interpret_globals()?;
        interpreter.call_function(function, args)
    }
}

impl<'ssa, W: Write> Interpreter<'ssa, W> {
    fn new(ssa: &'ssa Ssa, options: InterpreterOptions, output: W) -> Self {
        let call_stack = vec![CallContext::global_context()];
        Self { ssa, call_stack, side_effects_enabled: true, options, output }
    }

    fn call_context(&self) -> &CallContext {
        self.call_stack.last().expect("call_stack should always be non-empty")
    }

    fn call_context_mut(&mut self) -> &mut CallContext {
        self.call_stack.last_mut().expect("call_stack should always be non-empty")
    }

    fn global_scope(&self) -> &HashMap<ValueId, Value> {
        &self.call_stack.first().expect("call_stack should always be non-empty").scope
    }

    fn try_current_function(&self) -> Option<&'ssa Function> {
        let current_function_id = self.call_context().called_function;
        current_function_id.map(|current_function_id| &self.ssa.functions[&current_function_id])
    }

    fn current_function(&self) -> &'ssa Function {
        self.try_current_function().expect(
            "Tried calling `Interpreter::current_function` while evaluating global instructions",
        )
    }

    fn dfg(&self) -> &'ssa DataFlowGraph {
        &self.current_function().dfg
    }

    fn in_unconstrained_context(&self) -> bool {
        self.current_function().runtime().is_brillig()
    }

    /// Define or redefine a value.
    /// Redefinitions are expected in the case of loops.
    fn define(&mut self, id: ValueId, value: Value) -> IResult<()> {
        if self.options.trace {
            println!("{id} = {value}");
        }

        if let Some(func) = self.try_current_function() {
            let expected_type = func.dfg.type_of_value(id);
            let actual_type = value.get_type();

            if expected_type != actual_type {
                return Err(InterpreterError::Internal(
                    InternalError::ValueTypeDoesNotMatchReturnType {
                        value_id: id,
                        expected_type: expected_type.to_string(),
                        actual_type: actual_type.to_string(),
                    },
                ));
            }
        }

        self.call_context_mut().scope.insert(id, value);

        Ok(())
    }

    fn interpret_globals(&mut self) -> IResult<()> {
        let globals = &self.ssa.main().dfg.globals;
        for (global_id, global) in globals.values_iter() {
            let value = match global {
                super::ir::value::Value::Instruction { instruction, .. } => {
                    let instruction = &globals[*instruction];
                    self.interpret_instruction(instruction, &[global_id])?;
                    continue;
                }
                super::ir::value::Value::NumericConstant { constant, typ } => {
                    Value::from_constant(*constant, *typ)?
                }
                super::ir::value::Value::Function(id) => Value::Function(*id),
                super::ir::value::Value::Intrinsic(intrinsic) => Value::Intrinsic(*intrinsic),
                super::ir::value::Value::ForeignFunction(name) => {
                    Value::ForeignFunction(name.clone())
                }
                super::ir::value::Value::Global(_) | super::ir::value::Value::Param { .. } => {
                    unreachable!()
                }
            };
            self.define(global_id, value)?;
        }
        Ok(())
    }

    fn call_function(&mut self, function_id: FunctionId, mut arguments: Vec<Value>) -> IResults {
        self.call_stack.push(CallContext::new(function_id));

        let function = &self.ssa.functions[&function_id];
        if self.options.trace {
            println!();
            println!("enter function {} ({})", function_id, function.name());
        }

        let mut block_id = function.entry_block();
        let dfg = self.dfg();

        // Loop over blocks & instructions inline here to avoid pushing more
        // call frames (in rust). We only push call frames for function calls which
        // should prevent stack overflows for all but excessively large call stacks
        // that may overflow in the brillig vm as well.
        let return_values = loop {
            let block = &dfg[block_id];

            if arguments.len() != block.parameters().len() {
                return Err(internal(InternalError::BlockArgumentCountMismatch {
                    block: block_id,
                    arguments: arguments.len(),
                    parameters: block.parameters().len(),
                }));
            }

            for (parameter, argument) in block.parameters().iter().zip(arguments) {
                self.define(*parameter, argument)?;
            }

            for instruction_id in block.instructions() {
                let results = dfg.instruction_results(*instruction_id);
                self.interpret_instruction(&dfg[*instruction_id], results)?;
            }

            match block.terminator() {
                None => {
                    return Err(internal(InternalError::BlockMissingTerminator {
                        block: block_id,
                    }));
                }
                Some(TerminatorInstruction::Jmp { destination, arguments: jump_args, .. }) => {
                    block_id = *destination;
                    if self.options.trace {
                        println!("jump to {block_id}");
                    }
                    arguments = self.lookup_all(jump_args)?;
                }
                Some(TerminatorInstruction::JmpIf {
                    condition,
                    then_destination,
                    else_destination,
                    call_stack: _,
                }) => {
                    block_id = if self.lookup_bool(*condition, "jmpif condition")? {
                        *then_destination
                    } else {
                        *else_destination
                    };
                    if self.options.trace {
                        println!("jump to {block_id}");
                    }
                    arguments = Vec::new();
                }
                Some(TerminatorInstruction::Return { return_values, call_stack: _ }) => {
                    let return_values = self.lookup_all(return_values)?;
                    if self.options.trace && !return_values.is_empty() {
                        let return_values =
                            return_values.iter().map(ToString::to_string).collect::<Vec<_>>();
                        println!("return {}", return_values.join(", "));
                    }

                    break return_values;
                }
                Some(TerminatorInstruction::Unreachable { .. }) => {
                    return Err(InterpreterError::ReachedTheUnreachable);
                }
            }
        };

        if self.options.trace {
            println!("exit function {} ({})", function_id, function.name());
            println!();
        }

        self.call_stack.pop();

        if self.options.trace {
            if let Some(context) = self.call_stack.last() {
                if let Some(function_id) = context.called_function {
                    let function = &self.ssa.functions[&function_id];
                    println!("back in function {} ({})", function_id, function.name());
                }
            }
        }

        Ok(return_values)
    }

    fn lookup(&self, id: ValueId) -> IResult<Value> {
        if let Some(value) = self.call_context().scope.get(&id) {
            return Ok(value.clone());
        }

        if let Some(value) = self.global_scope().get(&id) {
            return Ok(value.clone());
        }

        Ok(match &self.dfg()[id] {
            super::ir::value::Value::NumericConstant { constant, typ } => {
                Value::from_constant(*constant, *typ)?
            }
            super::ir::value::Value::Function(id) => Value::Function(*id),
            super::ir::value::Value::Intrinsic(intrinsic) => Value::Intrinsic(*intrinsic),
            super::ir::value::Value::ForeignFunction(name) => Value::ForeignFunction(name.clone()),
            super::ir::value::Value::Instruction { .. }
            | super::ir::value::Value::Param { .. }
            | super::ir::value::Value::Global(_) => {
                unreachable!("`{id}` should already be in scope")
            }
        })
    }

    fn lookup_helper<T>(
        &self,
        value_id: ValueId,
        instruction: &'static str,
        expected_type: &'static str,
        convert: impl FnOnce(&Value) -> Option<T>,
    ) -> IResult<T> {
        let value = self.lookup(value_id)?;
        match convert(&value) {
            Some(value) => Ok(value),
            None => {
                let value = value.to_string();
                Err(internal(InternalError::TypeError {
                    value_id,
                    value,
                    expected_type,
                    instruction,
                }))
            }
        }
    }

    fn lookup_bool(&self, value_id: ValueId, instruction: &'static str) -> IResult<bool> {
        self.lookup_helper(value_id, instruction, "bool", Value::as_bool)
    }

    fn lookup_u32(&self, value_id: ValueId, instruction: &'static str) -> IResult<u32> {
        self.lookup_helper(value_id, instruction, "u32", Value::as_u32)
    }

    fn lookup_field(&self, value_id: ValueId, instruction: &'static str) -> IResult<FieldElement> {
        self.lookup_helper(value_id, instruction, "Field", Value::as_field)
    }

    fn lookup_numeric(
        &self,
        value_id: ValueId,
        instruction: &'static str,
    ) -> IResult<NumericValue> {
        self.lookup_helper(value_id, instruction, "numeric", Value::as_numeric)
    }

    fn lookup_array_or_slice(
        &self,
        value_id: ValueId,
        instruction: &'static str,
    ) -> IResult<ArrayValue> {
        self.lookup_helper(value_id, instruction, "array or slice", Value::as_array_or_slice)
    }

    fn lookup_bytes(&self, value_id: ValueId, instruction: &'static str) -> IResult<Vec<u8>> {
        let array =
            self.lookup_helper(value_id, instruction, "array or slice", Value::as_array_or_slice)?;
        let array = array.elements.borrow();
        array
            .iter()
            .map(|v| {
                v.as_u8().ok_or_else(|| {
                    internal(InternalError::TypeError {
                        value_id,
                        value: v.to_string(),
                        expected_type: "u8",
                        instruction,
                    })
                })
            })
            .collect::<Result<Vec<u8>, _>>()
    }

    fn lookup_vec_u32(&self, value_id: ValueId, instruction: &'static str) -> IResult<Vec<u32>> {
        let array =
            self.lookup_helper(value_id, instruction, "array or slice", Value::as_array_or_slice)?;
        let array = array.elements.borrow();
        array
            .iter()
            .map(|v| {
                v.as_u32().ok_or_else(|| {
                    internal(InternalError::TypeError {
                        value_id,
                        value: v.to_string(),
                        expected_type: "u32",
                        instruction,
                    })
                })
            })
            .collect::<Result<Vec<u32>, _>>()
    }

    fn lookup_vec_u64(&self, value_id: ValueId, instruction: &'static str) -> IResult<Vec<u64>> {
        let array =
            self.lookup_helper(value_id, instruction, "array or slice", Value::as_array_or_slice)?;
        let array = array.elements.borrow();
        array
            .iter()
            .map(|v| {
                v.as_u64().ok_or_else(|| {
                    internal(InternalError::TypeError {
                        value_id,
                        value: v.to_string(),
                        expected_type: "u64",
                        instruction,
                    })
                })
            })
            .collect::<Result<Vec<u64>, _>>()
    }

    fn lookup_vec_field(
        &self,
        value_id: ValueId,
        instruction: &'static str,
    ) -> IResult<Vec<FieldElement>> {
        let array =
            self.lookup_helper(value_id, instruction, "array or slice", Value::as_array_or_slice)?;
        let array = array.elements.borrow();
        array
            .iter()
            .map(|v| {
                v.as_field().ok_or_else(|| {
                    internal(InternalError::TypeError {
                        value_id,
                        value: v.to_string(),
                        expected_type: "Field",
                        instruction,
                    })
                })
            })
            .collect::<Result<Vec<FieldElement>, _>>()
    }

    fn lookup_string(&self, value_id: ValueId, instruction: &'static str) -> IResult<String> {
        self.lookup_helper(value_id, instruction, "string", Value::as_string)
    }

    fn lookup_reference(
        &self,
        value_id: ValueId,
        instruction: &'static str,
    ) -> IResult<ReferenceValue> {
        self.lookup_helper(value_id, instruction, "reference", Value::as_reference)
    }

    fn lookup_all(&self, ids: &[ValueId]) -> IResult<Vec<Value>> {
        try_vecmap(ids, |id| self.lookup(*id))
    }

    fn side_effects_enabled(&self, instruction: &Instruction) -> bool {
        let Some(current_function) = self.try_current_function() else {
            // If there's no current function it means we are evaluating global instructions
            return true;
        };

        match current_function.runtime() {
            RuntimeType::Acir(_) => {
                self.side_effects_enabled
                    || !instruction.requires_acir_gen_predicate(&current_function.dfg)
            }
            RuntimeType::Brillig(_) => true,
        }
    }

    #[allow(unused)]
    fn interpret_instruction(
        &mut self,
        instruction: &Instruction,
        results: &[ValueId],
    ) -> IResult<()> {
        let side_effects_enabled = self.side_effects_enabled(instruction);

        match instruction {
            Instruction::Binary(binary) => {
                let result = self.interpret_binary(binary, side_effects_enabled)?;
                self.define(results[0], result)?;
                Ok(())
            }
            // Cast in SSA changes the type without altering the value
            Instruction::Cast(value, numeric_type) => {
                let value = self.lookup_numeric(*value, "cast")?;
                let field = value.convert_to_field();
                let result = Value::from_constant(field, *numeric_type)?;
                self.define(results[0], result)?;
                Ok(())
            }
            Instruction::Not(id) => self.interpret_not(*id, results[0]),
            Instruction::Truncate { value, bit_size, max_bit_size } => {
                self.interpret_truncate(*value, *bit_size, *max_bit_size, results[0])
            }
            Instruction::Constrain(lhs_id, rhs_id, constrain_error) => {
                let lhs = self.lookup(*lhs_id)?;
                let rhs = self.lookup(*rhs_id)?;
                if side_effects_enabled && lhs != rhs {
                    let lhs = lhs.to_string();
                    let rhs = rhs.to_string();
                    let lhs_id = *lhs_id;
                    let rhs_id = *rhs_id;
                    let msg = if let Some(ConstrainError::StaticString(msg)) = constrain_error {
                        Some(msg.clone())
                    } else {
                        None
                    };
                    return Err(InterpreterError::ConstrainEqFailed {
                        lhs,
                        lhs_id,
                        rhs,
                        rhs_id,
                        msg,
                    });
                }
                Ok(())
            }
            Instruction::ConstrainNotEqual(lhs_id, rhs_id, constrain_error) => {
                let lhs = self.lookup(*lhs_id)?;
                let rhs = self.lookup(*rhs_id)?;
                if side_effects_enabled && lhs == rhs {
                    let lhs = lhs.to_string();
                    let rhs = rhs.to_string();
                    let lhs_id = *lhs_id;
                    let rhs_id = *rhs_id;
                    let msg = if let Some(ConstrainError::StaticString(msg)) = constrain_error {
                        Some(msg.clone())
                    } else {
                        None
                    };
                    return Err(InterpreterError::ConstrainNeFailed {
                        lhs,
                        lhs_id,
                        rhs,
                        rhs_id,
                        msg,
                    });
                }
                Ok(())
            }
            Instruction::RangeCheck { value, max_bit_size, assert_message } => self
                .interpret_range_check(
                    *value,
                    *max_bit_size,
                    assert_message.as_ref(),
                    side_effects_enabled,
                ),
            Instruction::Call { func, arguments } => {
                self.interpret_call(*func, arguments, results, side_effects_enabled)
            }
            Instruction::Allocate => {
                self.interpret_allocate(results[0]);
                Ok(())
            }
            Instruction::Load { address } => self.interpret_load(*address, results[0]),
            Instruction::Store { address, value } => self.interpret_store(*address, *value),
            Instruction::EnableSideEffectsIf { condition } => {
                self.side_effects_enabled = self.lookup_bool(*condition, "enable_side_effects")?;
                Ok(())
            }
            Instruction::ArrayGet { array, index, offset } => {
                self.interpret_array_get(*array, *index, *offset, results[0], side_effects_enabled)
            }
            Instruction::ArraySet { array, index, value, mutable, offset } => self
                .interpret_array_set(
                    *array,
                    *index,
                    *value,
                    *mutable,
                    *offset,
                    results[0],
                    side_effects_enabled,
                ),
            Instruction::IncrementRc { value } => self.interpret_inc_rc(*value),
            Instruction::DecrementRc { value } => self.interpret_dec_rc(*value),
            Instruction::IfElse { then_condition, then_value, else_condition, else_value } => self
                .interpret_if_else(
                    *then_condition,
                    *then_value,
                    *else_condition,
                    *else_value,
                    results[0],
                ),
            Instruction::MakeArray { elements, typ } => {
                self.interpret_make_array(elements, results[0], typ)
            }
            Instruction::Noop => Ok(()),
        }
    }

    fn interpret_not(&mut self, id: ValueId, result: ValueId) -> IResult<()> {
        let new_result = match self.lookup_numeric(id, "not instruction")? {
            NumericValue::Field(_) => {
                return Err(internal(InternalError::UnsupportedOperatorForType {
                    operator: "!",
                    typ: "Field",
                }));
            }
            NumericValue::U1(value) => NumericValue::U1(!value),
            NumericValue::U8(value) => NumericValue::U8(!value),
            NumericValue::U16(value) => NumericValue::U16(!value),
            NumericValue::U32(value) => NumericValue::U32(!value),
            NumericValue::U64(value) => NumericValue::U64(!value),
            NumericValue::U128(value) => NumericValue::U128(!value),
            NumericValue::I8(value) => NumericValue::I8(!value),
            NumericValue::I16(value) => NumericValue::I16(!value),
            NumericValue::I32(value) => NumericValue::I32(!value),
            NumericValue::I64(value) => NumericValue::I64(!value),
        };
        self.define(result, Value::Numeric(new_result))
    }

    fn interpret_truncate(
        &mut self,
        value_id: ValueId,
        bit_size: u32,
        max_bit_size: u32,
        result: ValueId,
    ) -> IResult<()> {
        let value = self.lookup_numeric(value_id, "truncate")?;
        if bit_size == 0 {
            return Err(internal(InternalError::TruncateToZeroBits { value_id, max_bit_size }));
        }

        let truncated = match value {
            NumericValue::Field(value) => NumericValue::Field(truncate_field(value, bit_size)),
            NumericValue::U1(value) => NumericValue::U1(value),
            NumericValue::U8(value) => NumericValue::U8(truncate_unsigned(value, bit_size)?),
            NumericValue::U16(value) => NumericValue::U16(truncate_unsigned(value, bit_size)?),
            NumericValue::U32(value) => NumericValue::U32(truncate_unsigned(value, bit_size)?),
            NumericValue::U64(value) => NumericValue::U64(truncate_unsigned(value, bit_size)?),
            NumericValue::U128(value) => NumericValue::U128(truncate_unsigned(value, bit_size)?),
            NumericValue::I8(value) => {
                NumericValue::I8(truncate_unsigned(value as u8, bit_size)? as i8)
            }
            NumericValue::I16(value) => {
                NumericValue::I16(truncate_unsigned(value as u16, bit_size)? as i16)
            }
            NumericValue::I32(value) => {
                NumericValue::I32(truncate_unsigned(value as u32, bit_size)? as i32)
            }
            NumericValue::I64(value) => {
                NumericValue::I64(truncate_unsigned(value as u64, bit_size)? as i64)
            }
        };

        self.define(result, Value::Numeric(truncated))
    }

    fn interpret_range_check(
        &mut self,
        value_id: ValueId,
        max_bit_size: u32,
        error_message: Option<&String>,
        side_effects_enabled: bool,
    ) -> IResult<()> {
        if !side_effects_enabled {
            return Ok(());
        }

        if max_bit_size == 0 {
            return Err(internal(InternalError::RangeCheckToZeroBits { value_id }));
        }

        let value = self.lookup_numeric(value_id, "range check")?;

        fn bit_count(x: impl Into<f64>) -> u32 {
            let x = x.into();
            if x <= 0.0001 { 0 } else { x.log2() as u32 + 1 }
        }

        let bit_count = match value {
            NumericValue::Field(value) => value.num_bits(),
            // max_bit_size > 0 so u1 should always pass these checks
            NumericValue::U1(_) => return Ok(()),
            NumericValue::U8(value) => bit_count(value),
            NumericValue::U16(value) => bit_count(value),
            NumericValue::U32(value) => bit_count(value),
            NumericValue::U64(value) => {
                // u64, u128, and i64 don't impl Into<f64>
                if value == 0 { 0 } else { value.ilog2() + 1 }
            }
            NumericValue::U128(value) => {
                if value == 0 {
                    0
                } else {
                    value.ilog2() + 1
                }
            }
            NumericValue::I8(value) => bit_count(value),
            NumericValue::I16(value) => bit_count(value),
            NumericValue::I32(value) => bit_count(value),
            NumericValue::I64(value) => {
                if value == 0 {
                    0
                } else {
                    value.ilog2() + 1
                }
            }
        };

        if bit_count > max_bit_size {
            let value = value.to_string();
            let actual_bits = bit_count;
            let max_bits = max_bit_size;

            Err(InterpreterError::RangeCheckFailed {
                value,
                value_id,
                actual_bits,
                max_bits,
                msg: error_message.cloned(),
            })
        } else {
            Ok(())
        }
    }

    fn interpret_call(
        &mut self,
        function_id: ValueId,
        argument_ids: &[ValueId],
        results: &[ValueId],
        side_effects_enabled: bool,
    ) -> IResult<()> {
        let function = self.lookup(function_id)?;
        let mut arguments = try_vecmap(argument_ids, |argument| self.lookup(*argument))?;

        let new_results = if side_effects_enabled {
            match function {
                Value::Function(id) => {
                    // If we're crossing a constrained -> unconstrained boundary we have to wipe
                    // any shared mutable fields in our arguments since brillig should conceptually
                    // receive fresh array on each invocation.
                    if !self.in_unconstrained_context()
                        && self.ssa.functions[&id].runtime().is_brillig()
                    {
                        for argument in arguments.iter_mut() {
                            Self::reset_array_state(argument)?;
                        }
                    }
                    self.call_function(id, arguments)?
                }
                Value::Intrinsic(intrinsic) => {
                    self.call_intrinsic(intrinsic, argument_ids, results)?
                }
                Value::ForeignFunction(name) if name == "print" => self.call_print(arguments)?,
                Value::ForeignFunction(name) => {
                    return Err(InterpreterError::UnknownForeignFunctionCall { name });
                }
                other => {
                    return Err(internal(InternalError::CalledNonFunction {
                        value: other.to_string(),
                        value_id: function_id,
                    }));
                }
            }
        } else {
            vecmap(results, |result| {
                let typ = self.dfg().type_of_value(*result);
                Value::uninitialized(&typ, *result)
            })
        };

        if new_results.len() != results.len() {
            let function_name = self.try_get_function_name(function_id);
            return Err(internal(InternalError::FunctionReturnedIncorrectArgCount {
                function: function_id,
                function_name,
                expected: results.len(),
                actual: new_results.len(),
            }));
        }

        for (result, new_result) in results.iter().zip(new_results) {
            self.define(*result, new_result)?;
        }
        Ok(())
    }

    /// Try to get a function's name or approximate it if it is not known
    fn try_get_function_name(&self, function: ValueId) -> String {
        match self.lookup(function) {
            Ok(Value::Function(id)) => match self.ssa.functions.get(&id) {
                Some(function) => function.name().to_string(),
                None => "unknown function".to_string(),
            },
            Ok(Value::Intrinsic(intrinsic)) => intrinsic.to_string(),
            Ok(Value::ForeignFunction(name)) => name,
            _ => "non-function".to_string(),
        }
    }

    /// Reset the value's `Shared` states in each array within. This is used to mimic each
    /// invocation of the brillig vm receiving fresh values. No matter the history of this value
    /// (e.g. even if they were previously returned from another brillig function) the reference
    /// count should always be 1 and it shouldn't alias any other arrays.
    fn reset_array_state(value: &mut Value) -> IResult<()> {
        match value {
            Value::Numeric(_)
            | Value::Function(_)
            | Value::Intrinsic(_)
            | Value::ForeignFunction(_) => Ok(()),

            Value::Reference(value) => {
                let value = value.to_string();
                Err(internal(InternalError::ReferenceValueCrossedUnconstrainedBoundary { value }))
            }

            Value::ArrayOrSlice(array_value) => {
                let mut elements = array_value.elements.borrow().to_vec();
                for element in elements.iter_mut() {
                    Self::reset_array_state(element)?;
                }
                array_value.elements = Shared::new(elements);
                array_value.rc = Shared::new(1);
                Ok(())
            }
        }
    }

    fn interpret_allocate(&mut self, result: ValueId) -> IResult<()> {
        let result_type = self.dfg().type_of_value(result);
        let element_type = match result_type {
            Type::Reference(element_type) => element_type,
            other => unreachable!(
                "Result of allocate should always be a reference type, but found {other}"
            ),
        };
        self.define(result, Value::reference(result, element_type))
    }

    fn interpret_load(&mut self, address: ValueId, result: ValueId) -> IResult<()> {
        let address = self.lookup_reference(address, "load")?;

        let element = address.element.borrow();
        let Some(value) = &*element else {
            let value = address.to_string();
            return Err(internal(InternalError::UninitializedReferenceValueLoaded { value }));
        };

        self.define(result, value.clone())?;
        Ok(())
    }

    fn interpret_store(&mut self, address: ValueId, value: ValueId) -> IResult<()> {
        let reference_address = self.lookup_reference(address, "store")?;

        let value = self.lookup(value)?;

        if self.options.trace {
            println!("store {value} at {address}");
        }

        *reference_address.element.borrow_mut() = Some(value);

        Ok(())
    }

    fn interpret_array_get(
        &mut self,
        array: ValueId,
        index: ValueId,
        offset: ArrayOffset,
        result: ValueId,
        side_effects_enabled: bool,
    ) -> IResult<()> {
        let element = if side_effects_enabled {
            let array = self.lookup_array_or_slice(array, "array get")?;
            let index = self.lookup_u32(index, "array get index")?;
            let index = index - offset.to_u32();
            let elements = array.elements.borrow();
            let element = elements.get(index as usize).ok_or_else(|| {
                InterpreterError::IndexOutOfBounds { index, length: elements.len() as u32 }
            })?;
            element.clone()
        } else {
            let typ = self.dfg().type_of_value(result);
            Value::uninitialized(&typ, result)
        };
        self.define(result, element)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn interpret_array_set(
        &mut self,
        array: ValueId,
        index: ValueId,
        value: ValueId,
        mutable: bool,
        offset: ArrayOffset,
        result: ValueId,
        side_effects_enabled: bool,
    ) -> IResult<()> {
        let array = self.lookup_array_or_slice(array, "array set")?;

        let result_array = if side_effects_enabled {
            let index = self.lookup_u32(index, "array set index")?;
            let index = index - offset.to_u32();
            let value = self.lookup(value)?;

            let should_mutate =
                if self.in_unconstrained_context() { *array.rc.borrow() == 1 } else { mutable };

            let len = array.elements.borrow().len();
            if index as usize >= len {
                return Err(InterpreterError::IndexOutOfBounds { index, length: len as u32 });
            }

            if should_mutate {
                array.elements.borrow_mut()[index as usize] = value;
                Value::ArrayOrSlice(array.clone())
            } else {
                let mut elements = array.elements.borrow().to_vec();
                elements[index as usize] = value;
                let elements = Shared::new(elements);
                let rc = Shared::new(1);
                let element_types = array.element_types.clone();
                let is_slice = array.is_slice;
                Value::ArrayOrSlice(ArrayValue { elements, rc, element_types, is_slice })
            }
        } else {
            // Side effects are disabled, return the original array
            Value::ArrayOrSlice(array)
        };
        self.define(result, result_array)?;
        Ok(())
    }

    fn interpret_inc_rc(&self, value_id: ValueId) -> IResult<()> {
        if self.in_unconstrained_context() {
            let array = self.lookup_array_or_slice(value_id, "inc_rc")?;
            let mut rc = array.rc.borrow_mut();
            if *rc == 0 {
                let value = array.to_string();
                return Err(InterpreterError::IncRcRevive { value_id, value });
            }
            *rc += 1;
        }
        Ok(())
    }

    fn interpret_dec_rc(&self, value_id: ValueId) -> IResult<()> {
        if self.in_unconstrained_context() {
            let array = self.lookup_array_or_slice(value_id, "dec_rc")?;
            let mut rc = array.rc.borrow_mut();
            if *rc == 0 {
                let value = array.to_string();
                return Err(InterpreterError::DecRcUnderflow { value_id, value });
            }
            *rc -= 1;
        }
        Ok(())
    }

    fn interpret_if_else(
        &mut self,
        then_condition_id: ValueId,
        then_value: ValueId,
        else_condition_id: ValueId,
        else_value: ValueId,
        result: ValueId,
    ) -> IResult<()> {
        let then_condition = self.lookup_bool(then_condition_id, "then condition")?;
        let else_condition = self.lookup_bool(else_condition_id, "else condition")?;
        let then_value = self.lookup(then_value)?;
        let else_value = self.lookup(else_value)?;

        // Note that `then_condition = !else_condition` doesn't always hold!
        // Notably if this is a nested if expression we could have something like:
        //   then_condition = outer_condition & a
        //   else_condition = outer_condition & !a
        // If `outer_condition` is false, both will be false.
        if then_condition && else_condition {
            return Err(InterpreterError::DoubleTrueIfElse {
                then_condition_id,
                else_condition_id,
            });
        }

        let new_result = if !then_condition && !else_condition {
            // Returning uninitialized/zero if both conditions are false to match
            // the decomposition of `cond * then_value + !cond * else_value` for numeric values.
            let typ = self.dfg().type_of_value(result);
            Value::uninitialized(&typ, result)
        } else if then_condition {
            then_value
        } else {
            else_value
        };

        self.define(result, new_result)
    }

    fn interpret_make_array(
        &mut self,
        elements: &im::Vector<ValueId>,
        result: ValueId,
        result_type: &Type,
    ) -> IResult<()> {
        let elements = try_vecmap(elements, |element| self.lookup(*element))?;
        let is_slice = matches!(&result_type, Type::Slice(..));

        let array = Value::ArrayOrSlice(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types: result_type.clone().element_types(),
            is_slice,
        });
        self.define(result, array)
    }
}

/// Applies an infallible integer binary operation to two `NumericValue`s.
///
/// # Parameters
/// - `$lhs`, `$rhs`: The left hand side and right hand side operands (must be the same variant).
/// - `$binary`: The binary instruction, used for error handling if types mismatch.
/// - `$f`: A function (e.g., `wrapping_add`) that applies the operation on the raw numeric types.
///
/// # Panics
/// - If either operand is a [NumericValue::Field] or [NumericValue::U1] variant, this macro will panic with unreachable.
///
/// # Errors
/// - If the operand types don't match, returns an [InternalError::MismatchedTypesInBinaryOperator].
///
/// # Returns
/// A `NumericValue` containing the result of the operation, matching the original type.
macro_rules! apply_int_binop {
    ($lhs:expr, $rhs:expr, $binary:expr, $f:expr) => {{
        use value::NumericValue::*;
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => {
                unreachable!("Expected only integer values, found field values")
            }
            (U1(_), U1(_)) => unreachable!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U8($f(&lhs, &rhs)),
            (U16(lhs), U16(rhs)) => U16($f(&lhs, &rhs)),
            (U32(lhs), U32(rhs)) => U32($f(&lhs, &rhs)),
            (U64(lhs), U64(rhs)) => U64($f(&lhs, &rhs)),
            (U128(lhs), U128(rhs)) => U128($f(&lhs, &rhs)),
            (I8(lhs), I8(rhs)) => I8($f(&lhs, &rhs)),
            (I16(lhs), I16(rhs)) => I16($f(&lhs, &rhs)),
            (I32(lhs), I32(rhs)) => I32($f(&lhs, &rhs)),
            (I64(lhs), I64(rhs)) => I64($f(&lhs, &rhs)),
            (lhs, rhs) => {
                let binary = $binary;
                return Err(internal(InternalError::MismatchedTypesInBinaryOperator {
                    lhs: lhs.to_string(),
                    rhs: rhs.to_string(),
                    operator: binary.operator,
                    lhs_id: binary.lhs,
                    rhs_id: binary.rhs,
                }));
            }
        }
    }};
}

/// Applies a fallible integer binary operation (e.g., checked arithmetic) to two `NumericValue`s.
///
/// # Parameters
/// - `$dfg`: The data flow graph, used for formatting diagnostic error messages.
/// - `$lhs`, `$rhs`: The left-hand side and right-hand side operands (must be the same variant).
/// - `$binary`: The binary instruction, used for diagnostics and overflow reporting.
/// - `$f`: A fallible operation function that returns an `Option<_>` (e.g., `checked_add`).
///
/// # Panics
/// - If either operand is a [NumericValue::Field]or [NumericValue::U1], this macro panics as those types are not supported.
///
/// # Errors
/// - Returns [InterpreterError::Overflow] if the checked operation returns `None`.
/// - Returns [InterpreterError::DivisionByZero] for `Div` and `Mod` on zero.
/// - Returns [InternalError::MismatchedTypesInBinaryOperator] if the operand types don't match.
///
/// # Returns
/// A `NumericValue` containing the result of the operation, or an `Err` with the appropriate error.
macro_rules! apply_int_binop_opt {
    ($dfg:expr, $lhs:expr, $rhs:expr, $binary:expr, $f:expr) => {{
        use value::NumericValue::*;

        let lhs = $lhs;
        let rhs = $rhs;
        let binary = $binary;
        let operator = binary.operator;

        let overflow = || {
            if matches!(operator, BinaryOp::Div | BinaryOp::Mod) {
                let lhs_id = binary.lhs;
                let rhs_id = binary.rhs;
                let lhs = lhs.to_string();
                let rhs = rhs.to_string();
                InterpreterError::DivisionByZero { lhs_id, lhs, rhs_id, rhs }
            } else {
                let instruction =
                    format!("`{}` ({operator} {lhs}, {rhs})", display_binary(binary, $dfg));
                InterpreterError::Overflow { operator, instruction }
            }
        };

        match (lhs, rhs) {
            (Field(_), Field(_)) => {
                unreachable!("Expected only integer values, found field values")
            }
            (U1(_), U1(_)) => unreachable!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U8($f(&lhs, &rhs).ok_or_else(overflow)?),
            (U16(lhs), U16(rhs)) => U16($f(&lhs, &rhs).ok_or_else(overflow)?),
            (U32(lhs), U32(rhs)) => U32($f(&lhs, &rhs).ok_or_else(overflow)?),
            (U64(lhs), U64(rhs)) => U64($f(&lhs, &rhs).ok_or_else(overflow)?),
            (U128(lhs), U128(rhs)) => U128($f(&lhs, &rhs).ok_or_else(overflow)?),
            (I8(lhs), I8(rhs)) => I8($f(&lhs, &rhs).ok_or_else(overflow)?),
            (I16(lhs), I16(rhs)) => I16($f(&lhs, &rhs).ok_or_else(overflow)?),
            (I32(lhs), I32(rhs)) => I32($f(&lhs, &rhs).ok_or_else(overflow)?),
            (I64(lhs), I64(rhs)) => I64($f(&lhs, &rhs).ok_or_else(overflow)?),
            (lhs, rhs) => {
                return Err(internal(InternalError::MismatchedTypesInBinaryOperator {
                    lhs: lhs.to_string(),
                    rhs: rhs.to_string(),
                    operator,
                    lhs_id: binary.lhs,
                    rhs_id: binary.rhs,
                }));
            }
        }
    }};
}

macro_rules! apply_int_comparison_op {
    ($lhs:expr, $rhs:expr, $binary:expr, $f:expr) => {{
        use NumericValue::*;
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => {
                unreachable!("Expected only integer values, found field values")
            }
            (U1(_), U1(_)) => unreachable!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U1($f(&lhs, &rhs)),
            (U16(lhs), U16(rhs)) => U1($f(&lhs, &rhs)),
            (U32(lhs), U32(rhs)) => U1($f(&lhs, &rhs)),
            (U64(lhs), U64(rhs)) => U1($f(&lhs, &rhs)),
            (U128(lhs), U128(rhs)) => U1($f(&lhs, &rhs)),
            (I8(lhs), I8(rhs)) => U1($f(&lhs, &rhs)),
            (I16(lhs), I16(rhs)) => U1($f(&lhs, &rhs)),
            (I32(lhs), I32(rhs)) => U1($f(&lhs, &rhs)),
            (I64(lhs), I64(rhs)) => U1($f(&lhs, &rhs)),
            (lhs, rhs) => {
                let binary = $binary;
                return Err(internal(InternalError::MismatchedTypesInBinaryOperator {
                    lhs: lhs.to_string(),
                    rhs: rhs.to_string(),
                    operator: binary.operator,
                    lhs_id: binary.lhs,
                    rhs_id: binary.rhs,
                }));
            }
        }
    }};
}

impl<W: Write> Interpreter<'_, W> {
    fn interpret_binary(&mut self, binary: &Binary, side_effects_enabled: bool) -> IResult<Value> {
        let lhs_id = binary.lhs;
        let rhs_id = binary.rhs;
        let lhs = self.lookup_numeric(lhs_id, "binary op lhs")?;
        let rhs = self.lookup_numeric(rhs_id, "binary op rhs")?;

        if lhs.get_type() != rhs.get_type()
            && !matches!(binary.operator, BinaryOp::Shl | BinaryOp::Shr)
        {
            return Err(internal(InternalError::MismatchedTypesInBinaryOperator {
                lhs_id,
                lhs: lhs.to_string(),
                operator: binary.operator,
                rhs_id,
                rhs: rhs.to_string(),
            }));
        }

        // Disable this instruction if it is side-effectual and side effects are disabled.
        if !side_effects_enabled {
            let zero = NumericValue::zero(lhs.get_type());
            return Ok(Value::Numeric(zero));
        }

        if let (Some(lhs), Some(rhs)) = (lhs.as_field(), rhs.as_field()) {
            return self.interpret_field_binary_op(lhs, binary.operator, rhs, lhs_id, rhs_id);
        }

        if let (Some(lhs), Some(rhs)) = (lhs.as_bool(), rhs.as_bool()) {
            return self.interpret_u1_binary_op(lhs, rhs, binary);
        }

        let dfg = self.dfg();
        let result = match binary.operator {
            BinaryOp::Add { unchecked: false } => {
                apply_int_binop_opt!(dfg, lhs, rhs, binary, num_traits::CheckedAdd::checked_add)
            }
            BinaryOp::Add { unchecked: true } => {
                apply_int_binop!(lhs, rhs, binary, num_traits::WrappingAdd::wrapping_add)
            }
            BinaryOp::Sub { unchecked: false } => {
                apply_int_binop_opt!(dfg, lhs, rhs, binary, num_traits::CheckedSub::checked_sub)
            }
            BinaryOp::Sub { unchecked: true } => {
                apply_int_binop!(lhs, rhs, binary, num_traits::WrappingSub::wrapping_sub)
            }
            BinaryOp::Mul { unchecked: false } => {
                // Only unsigned multiplication has side effects
                apply_int_binop_opt!(dfg, lhs, rhs, binary, num_traits::CheckedMul::checked_mul)
            }
            BinaryOp::Mul { unchecked: true } => {
                apply_int_binop!(lhs, rhs, binary, num_traits::WrappingMul::wrapping_mul)
            }
            BinaryOp::Div => {
                apply_int_binop_opt!(dfg, lhs, rhs, binary, num_traits::CheckedDiv::checked_div)
            }
            BinaryOp::Mod => {
                apply_int_binop_opt!(dfg, lhs, rhs, binary, num_traits::CheckedRem::checked_rem)
            }
            BinaryOp::Eq => apply_int_comparison_op!(lhs, rhs, binary, |a, b| a == b),
            BinaryOp::Lt => apply_int_comparison_op!(lhs, rhs, binary, |a, b| a < b),
            BinaryOp::And => {
                apply_int_binop!(lhs, rhs, binary, std::ops::BitAnd::bitand)
            }
            BinaryOp::Or => {
                apply_int_binop!(lhs, rhs, binary, std::ops::BitOr::bitor)
            }
            BinaryOp::Xor => {
                apply_int_binop!(lhs, rhs, binary, std::ops::BitXor::bitxor)
            }
            BinaryOp::Shl => {
                let Some(rhs) = rhs.as_u8() else {
                    let rhs = rhs.to_string();
                    return Err(internal(InternalError::RhsOfBitShiftShouldBeU8 {
                        operator: "<<",
                        rhs_id,
                        rhs,
                    }));
                };

                let rhs = rhs as u32;
                use NumericValue::*;
                match lhs {
                    Field(_) => {
                        return Err(internal(InternalError::UnsupportedOperatorForType {
                            operator: "<<",
                            typ: "Field",
                        }));
                    }
                    U1(value) => U1(if rhs == 0 { value } else { false }),
                    U8(value) => U8(value.checked_shl(rhs).unwrap_or(0)),
                    U16(value) => U16(value.checked_shl(rhs).unwrap_or(0)),
                    U32(value) => U32(value.checked_shl(rhs).unwrap_or(0)),
                    U64(value) => U64(value.checked_shl(rhs).unwrap_or(0)),
                    U128(value) => U128(value.checked_shl(rhs).unwrap_or(0)),
                    I8(value) => I8(value.checked_shl(rhs).unwrap_or(0)),
                    I16(value) => I16(value.checked_shl(rhs).unwrap_or(0)),
                    I32(value) => I32(value.checked_shl(rhs).unwrap_or(0)),
                    I64(value) => I64(value.checked_shl(rhs).unwrap_or(0)),
                }
            }
            BinaryOp::Shr => {
                let fallback = || {
                    if lhs.is_negative() {
                        NumericValue::neg_one(lhs.get_type())
                    } else {
                        NumericValue::zero(lhs.get_type())
                    }
                };

                let Some(rhs) = rhs.as_u8() else {
                    let rhs = rhs.to_string();
                    return Err(internal(InternalError::RhsOfBitShiftShouldBeU8 {
                        operator: ">>",
                        rhs_id,
                        rhs,
                    }));
                };

                let rhs = rhs as u32;
                use NumericValue::*;
                match lhs {
                    Field(_) => {
                        return Err(internal(InternalError::UnsupportedOperatorForType {
                            operator: ">>",
                            typ: "Field",
                        }));
                    }
                    U1(value) => U1(if rhs == 0 { value } else { false }),
                    U8(value) => value.checked_shr(rhs).map(U8).unwrap_or_else(fallback),
                    U16(value) => value.checked_shr(rhs).map(U16).unwrap_or_else(fallback),
                    U32(value) => value.checked_shr(rhs).map(U32).unwrap_or_else(fallback),
                    U64(value) => value.checked_shr(rhs).map(U64).unwrap_or_else(fallback),
                    U128(value) => value.checked_shr(rhs).map(U128).unwrap_or_else(fallback),
                    I8(value) => value.checked_shr(rhs).map(I8).unwrap_or_else(fallback),
                    I16(value) => value.checked_shr(rhs).map(I16).unwrap_or_else(fallback),
                    I32(value) => value.checked_shr(rhs).map(I32).unwrap_or_else(fallback),
                    I64(value) => value.checked_shr(rhs).map(I64).unwrap_or_else(fallback),
                }
            }
        };
        Ok(Value::Numeric(result))
    }

    fn interpret_field_binary_op(
        &mut self,
        lhs: FieldElement,
        operator: BinaryOp,
        rhs: FieldElement,
        lhs_id: ValueId,
        rhs_id: ValueId,
    ) -> IResult<Value> {
        let unsupported_operator = |operator| -> IResult<Value> {
            let typ = "Field";
            Err(internal(InternalError::UnsupportedOperatorForType { operator, typ }))
        };

        let result = match operator {
            BinaryOp::Add { unchecked: _ } => NumericValue::Field(lhs + rhs),
            BinaryOp::Sub { unchecked: _ } => NumericValue::Field(lhs - rhs),
            BinaryOp::Mul { unchecked: _ } => NumericValue::Field(lhs * rhs),
            BinaryOp::Div => {
                if rhs.is_zero() {
                    let lhs = lhs.to_string();
                    let rhs = rhs.to_string();
                    return Err(InterpreterError::DivisionByZero { lhs_id, lhs, rhs_id, rhs });
                }
                NumericValue::Field(lhs / rhs)
            }
            BinaryOp::Mod => return unsupported_operator("%"),
            BinaryOp::Eq => NumericValue::U1(lhs == rhs),
            BinaryOp::Lt => NumericValue::U1(lhs < rhs),
            BinaryOp::And => return unsupported_operator("&"),
            BinaryOp::Or => return unsupported_operator("|"),
            BinaryOp::Xor => return unsupported_operator("^"),
            BinaryOp::Shl => return unsupported_operator("<<"),
            BinaryOp::Shr => return unsupported_operator(">>"),
        };
        Ok(Value::Numeric(result))
    }

    fn interpret_u1_binary_op(&mut self, lhs: bool, rhs: bool, binary: &Binary) -> IResult<Value> {
        let overflow = || {
            let instruction = format!("`{}` ({lhs} << {rhs})", display_binary(binary, self.dfg()));
            let operator = binary.operator;
            InterpreterError::Overflow { operator, instruction }
        };

        let lhs_id = binary.lhs;
        let rhs_id = binary.rhs;

        let result = match binary.operator {
            BinaryOp::Add { unchecked: true } => lhs ^ rhs,
            BinaryOp::Add { unchecked: false } => {
                if lhs && rhs {
                    return Err(overflow());
                } else {
                    lhs ^ rhs
                }
            }
            BinaryOp::Sub { unchecked: true } => {
                // (0, 0) -> 0
                // (0, 1) -> 1  (underflow)
                // (1, 0) -> 1
                // (1, 1) -> 0
                lhs ^ rhs
            }
            BinaryOp::Sub { unchecked: false } => {
                if !lhs && rhs {
                    return Err(overflow());
                } else {
                    lhs ^ rhs
                }
            }
            BinaryOp::Mul { unchecked: _ } => lhs & rhs, // (*) = (&) for u1
            BinaryOp::Div => {
                // (0, 0) -> (division by 0)
                // (0, 1) -> 0
                // (1, 0) -> (division by 0)
                // (1, 1) -> 1
                if !rhs {
                    let lhs = (lhs as u8).to_string();
                    let rhs = (rhs as u8).to_string();
                    return Err(InterpreterError::DivisionByZero { lhs_id, lhs, rhs_id, rhs });
                }
                lhs
            }
            BinaryOp::Mod => {
                // (0, 0) -> (division by 0)
                // (0, 1) -> 0
                // (1, 0) -> (division by 0)
                // (1, 1) -> 0
                if !rhs {
                    let lhs = format!("u1 {}", lhs as u8);
                    let rhs = format!("u1 {}", rhs as u8);
                    return Err(InterpreterError::DivisionByZero { lhs_id, lhs, rhs_id, rhs });
                }
                false
            }
            BinaryOp::Eq => lhs == rhs,
            // clippy complains when you do `lhs < rhs` and recommends this instead
            BinaryOp::Lt => !lhs & rhs,
            BinaryOp::And => lhs & rhs,
            BinaryOp::Or => lhs | rhs,
            BinaryOp::Xor => lhs ^ rhs,
            BinaryOp::Shl => {
                return Err(internal(InternalError::RhsOfBitShiftShouldBeU8 {
                    operator: "<<",
                    rhs_id,
                    rhs: format!("u1 {}", rhs as u8),
                }));
            }
            BinaryOp::Shr => {
                return Err(internal(InternalError::RhsOfBitShiftShouldBeU8 {
                    operator: ">>",
                    rhs_id,
                    rhs: format!("u1 {}", rhs as u8),
                }));
            }
        };
        Ok(Value::Numeric(NumericValue::U1(result)))
    }
}

fn truncate_unsigned<T>(value: T, bit_size: u32) -> IResult<T>
where
    u128: From<T>,
    T: TryFrom<u128>,
    <T as TryFrom<u128>>::Error: std::fmt::Debug,
{
    let value_u128 = u128::from(value);
    let bit_mask = match bit_size.cmp(&MAX_UNSIGNED_BIT_SIZE) {
        Ordering::Less => (1u128 << bit_size) - 1,
        Ordering::Equal => u128::MAX,
        Ordering::Greater => {
            return Err(internal(InternalError::InvalidUnsignedTruncateBitSize { bit_size }));
        }
    };

    let result = value_u128 & bit_mask;
    Ok(T::try_from(result).expect(
        "The truncated result should always be smaller than or equal to the original `value`",
    ))
}

fn internal(error: InternalError) -> InterpreterError {
    InterpreterError::Internal(error)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_truncate_unsigned() {
        assert_eq!(super::truncate_unsigned(57_u32, 8).unwrap(), 57);
        assert_eq!(super::truncate_unsigned(257_u16, 8).unwrap(), 1);
        assert_eq!(super::truncate_unsigned(130_u8, 7).unwrap(), 2);
        assert_eq!(super::truncate_unsigned(u8::MAX, 8).unwrap(), u8::MAX);
        assert_eq!(super::truncate_unsigned(u128::MAX, 128).unwrap(), u128::MAX);
    }

    #[test]
    fn test_truncate_signed() {
        // Signed values roundtrip through truncate_unsigned
        assert_eq!(super::truncate_unsigned(57_i32 as u32, 8).unwrap() as i32, 57);
        assert_eq!(super::truncate_unsigned(257_i16 as u16, 8).unwrap() as i16, 1);
        assert_eq!(super::truncate_unsigned(130_i64 as u64, 7).unwrap() as i64, 2);
        assert_eq!(super::truncate_unsigned(i16::MAX as u16, 16).unwrap() as i16, i16::MAX);

        // For negatives we rely on the `as iN` cast at the end to convert large integers
        // back into negatives. For this reason we don't test bit sizes other than 8, 16, 32, 64
        // although we don't support other bit sizes anyway.
        assert_eq!(super::truncate_unsigned(-57_i32 as u32, 8).unwrap() as i8, -57);
        assert_eq!(super::truncate_unsigned(-258_i16 as u16, 8).unwrap() as i8, -2);
        assert_eq!(super::truncate_unsigned(i8::MIN as u8, 8).unwrap() as i8, i8::MIN);
        assert_eq!(super::truncate_unsigned(-129_i32 as u32, 8).unwrap() as i8, 127);

        // underflow to i16::MAX
        assert_eq!(super::truncate_unsigned(i16::MIN as u32 - 1, 16).unwrap() as i16, 32767);
    }
}
