use std::{cmp::Ordering, collections::BTreeMap, io::Write};

use super::{
    Ssa,
    ir::{
        dfg::DataFlowGraph,
        function::{Function, FunctionId, RuntimeType},
        instruction::{Binary, BinaryOp, ConstrainError, Instruction, TerminatorInstruction},
        types::Type,
        value::Value as IrValue,
        value::ValueId,
    },
};
use crate::ssa::{
    interpreter::value::Fitted,
    ir::{instruction::binary::truncate_field, printer::display_binary, types::NumericType},
};
use acvm::{AcirField, FieldElement};
use errors::{InternalError, InterpreterError, MAX_UNSIGNED_BIT_SIZE};
use iter_extended::{try_vecmap, vecmap};
use noirc_frontend::Shared;
use num_traits::{CheckedShl, CheckedShr};
use rustc_hash::FxHashMap as HashMap;
use value::{ArrayValue, NumericValue, ReferenceValue};

pub mod errors;
mod intrinsics;
pub(crate) mod tests;
pub mod value;

use value::Value;

pub(crate) struct Interpreter<'ssa, W> {
    /// Contains each function called with `main` (or the first called function if
    /// the interpreter was manually invoked on a different function) at
    /// the front of the Vec.
    call_stack: Vec<CallContext>,

    functions: &'ssa BTreeMap<FunctionId, Function>,

    /// The options the interpreter was created with.
    options: InterpreterOptions,

    /// Print output.
    output: W,

    /// Number of instructions and terminators executed.
    step_counter: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InterpreterOptions {
    /// If true, the interpreter will trace its execution.
    pub trace: bool,
    /// If true, the interpreter treats all foreign function calls (e.g., `print`) as unknown
    pub no_foreign_calls: bool,
    /// Optional limit on the number of executed instructions and terminators, to avoid infinite loops.
    pub step_limit: Option<usize>,
}

struct CallContext {
    /// The function that was called. This is `None` only for the top-level global
    /// scope where global instructions are evaluated.
    called_function: Option<FunctionId>,

    /// Contains each value currently defined and visible to the current function.
    scope: HashMap<ValueId, Value>,

    /// This variable can be modified by `enable_side_effects_if` instructions and is
    /// expected to have no effect if there are no such instructions or if the code
    /// being executed is an unconstrained function.
    side_effects_enabled: bool,
}

impl CallContext {
    fn new(called_function: FunctionId) -> Self {
        Self {
            called_function: Some(called_function),
            scope: Default::default(),
            side_effects_enabled: true,
        }
    }

    fn global_context() -> Self {
        Self { called_function: None, scope: Default::default(), side_effects_enabled: true }
    }
}

type IResult<T> = Result<T, InterpreterError>;
pub type IResults = IResult<Vec<Value>>;

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
        Self::new_from_functions(&ssa.functions, options, output)
    }

    pub(crate) fn new_from_functions(
        functions: &'ssa BTreeMap<FunctionId, Function>,
        options: InterpreterOptions,
        output: W,
    ) -> Self {
        let call_stack = vec![CallContext::global_context()];
        Self { functions, call_stack, options, output, step_counter: 0 }
    }

    pub(crate) fn functions(&self) -> &BTreeMap<FunctionId, Function> {
        self.functions
    }

    /// Resets the step counter to 0.
    ///
    /// This resets the step counter, to reset the budget before
    /// interpreting the next entry point.
    pub(crate) fn reset_step_counter(&mut self) {
        self.step_counter = 0;
    }

    /// Increment the step counter, or return [InterpreterError::OutOfBudget].
    ///
    /// If there is no step limit, then it doesn't increment the counter.
    fn inc_step_counter(&mut self) -> IResult<()> {
        if let Some(limit) = self.options.step_limit {
            if self.step_counter >= limit {
                return Err(InterpreterError::OutOfBudget { steps: self.step_counter });
            }
            // With a limit we shouldn't wrap around, but just in case we wanted move this outside,
            // use a safe wrap-around increment.
            self.step_counter = self.step_counter.wrapping_add(1);
        };
        Ok(())
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
        current_function_id.map(|current_function_id| &self.functions[&current_function_id])
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
                return Err(internal(InternalError::ValueTypeDoesNotMatchReturnType {
                    value_id: id,
                    expected_type: expected_type.to_string(),
                    actual_type: actual_type.to_string(),
                }));
            }
        }

        self.call_context_mut().scope.insert(id, value);

        Ok(())
    }

    pub(crate) fn interpret_globals(&mut self) -> IResult<()> {
        let (_, function) = self.functions.first_key_value().unwrap();
        let globals = &function.dfg.globals;
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

    pub(crate) fn call_function(
        &mut self,
        function_id: FunctionId,
        mut arguments: Vec<Value>,
    ) -> IResults {
        self.call_stack.push(CallContext::new(function_id));

        let function = &self.functions[&function_id];
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

            // Account for the terminator; a function might not have actual instructions, other than jumping around.
            self.inc_step_counter()?;

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
                    let function = &self.functions[&function_id];
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

    /// Look up an array index.
    ///
    /// If the value exists but it's `Unfit`, returns `IndexOutOfBounds`.
    fn lookup_array_index(
        &self,
        value_id: ValueId,
        instruction: &'static str,
        length: u32,
    ) -> IResult<u32> {
        self.lookup_helper(value_id, instruction, "u32", Value::as_u32).map_err(|e| {
            if matches!(e, InterpreterError::Internal(InternalError::TypeError { .. })) {
                if let Ok(Value::Numeric(NumericValue::U32(Fitted::Unfit(index)))) =
                    self.lookup(value_id)
                {
                    return InterpreterError::IndexOutOfBounds { index, length };
                }
            }
            e
        })
    }

    fn lookup_bytes(&self, value_id: ValueId, instruction: &'static str) -> IResult<Vec<u8>> {
        let array = self.lookup_array_or_slice(value_id, instruction)?;
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
        let array = self.lookup_array_or_slice(value_id, instruction)?;
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
        let array = self.lookup_array_or_slice(value_id, instruction)?;
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
        let array = self.lookup_array_or_slice(value_id, instruction)?;
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
                self.call_context().side_effects_enabled
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
        self.inc_step_counter()?;

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
                self.call_context_mut().side_effects_enabled =
                    self.lookup_bool(*condition, "enable_side_effects")?;
                Ok(())
            }
            Instruction::ArrayGet { array, index } => {
                self.interpret_array_get(*array, *index, results[0], side_effects_enabled)
            }
            Instruction::ArraySet { array, index, value, mutable } => self.interpret_array_set(
                *array,
                *index,
                *value,
                *mutable,
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
        let num_value = self.lookup_numeric(id, "not instruction")?;
        let bit_size = num_value.get_type().bit_size::<FieldElement>();

        // Based on AcirContext::not_var
        fn fitted_not<T: std::ops::Not<Output = T>>(value: Fitted<T>, bit_size: u32) -> Fitted<T> {
            value.map(
                |value| !value,
                |value| {
                    // Based on AcirContext::not_var
                    let bit_size = FieldElement::from(bit_size);
                    let max = FieldElement::from(2u128).pow(&bit_size) - FieldElement::one();
                    max - value
                },
            )
        }

        let new_result = match num_value {
            NumericValue::Field(_) => {
                return Err(internal(InternalError::UnsupportedOperatorForType {
                    operator: "!",
                    typ: "Field",
                }));
            }
            NumericValue::U1(value) => NumericValue::U1(!value),
            NumericValue::U8(value) => NumericValue::U8(fitted_not(value, bit_size)),
            NumericValue::U16(value) => NumericValue::U16(fitted_not(value, bit_size)),
            NumericValue::U32(value) => NumericValue::U32(fitted_not(value, bit_size)),
            NumericValue::U64(value) => NumericValue::U64(fitted_not(value, bit_size)),
            NumericValue::U128(value) => NumericValue::U128(fitted_not(value, bit_size)),
            NumericValue::I8(value) => NumericValue::I8(fitted_not(value, bit_size)),
            NumericValue::I16(value) => NumericValue::I16(fitted_not(value, bit_size)),
            NumericValue::I32(value) => NumericValue::I32(fitted_not(value, bit_size)),
            NumericValue::I64(value) => NumericValue::I64(fitted_not(value, bit_size)),
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
        use Fitted::*;
        use NumericValue::*;

        let value = self.lookup_numeric(value_id, "truncate")?;
        let typ = value.get_type();
        if bit_size == 0 {
            return Err(internal(InternalError::TruncateToZeroBits { value_id, max_bit_size }));
        }

        let is_sub = if let IrValue::Instruction { instruction, .. } = self.dfg()[value_id] {
            matches!(
                self.dfg()[instruction],
                Instruction::Binary(Binary { operator: BinaryOp::Sub { .. }, .. })
            )
        } else {
            false
        };

        // Based on acir::Context::convert_ssa_truncate: subtractions must first have the integer modulus added to avoid underflow.
        fn truncate_unfit(
            mut value: FieldElement,
            bit_size: u32,
            max_bit_size: u32,
            is_sub: bool,
        ) -> FieldElement {
            if is_sub {
                let max_bit_size = FieldElement::from(max_bit_size);
                let integer_modulus = FieldElement::from(2u128).pow(&max_bit_size);
                value += integer_modulus;
            }
            truncate_field(value, bit_size)
        }

        // Truncate an unsigned value.
        fn truncate_fitted<F, T>(
            cons: F,
            typ: NumericType,
            value: Fitted<T>,
            bit_size: u32,
            max_bit_size: u32,
            is_sub: bool,
        ) -> IResult<NumericValue>
        where
            T: TryFrom<u128>,
            u128: From<T>,
            <T as TryFrom<u128>>::Error: std::fmt::Debug,
            F: Fn(Fitted<T>) -> NumericValue,
        {
            match value {
                Fit(value) => Ok(cons(Fit(truncate_unsigned(value, bit_size)?))),
                Unfit(value) => {
                    let truncated = truncate_unfit(value, bit_size, max_bit_size, is_sub);
                    NumericValue::from_constant(truncated, typ)
                        .or_else(|_| Ok(cons(Unfit(truncated))))
                }
            }
        }

        // Truncate a signed value via unsigned cast and back.
        macro_rules! truncate_via {
            ($cons:expr, $typ:expr, $value:ident, $bit_size:ident, $max_bit_size:ident, $is_sub:ident, $signed:ty, $unsigned:ty) => {
                match $value {
                    Fit(value) => {
                        $cons(Fit(truncate_unsigned(value as $unsigned, $bit_size)? as $signed))
                    }
                    Unfit(value) => {
                        let truncated = truncate_unfit(value, bit_size, max_bit_size, is_sub);
                        NumericValue::from_constant(truncated, typ)
                            .unwrap_or_else(|_| $cons(Unfit(truncated)))
                    }
                }
            };
        }

        let truncated = match value {
            Field(value) => Field(truncate_field(value, bit_size)),
            U1(value) => U1(value),
            U8(value) => truncate_fitted(U8, typ, value, bit_size, max_bit_size, is_sub)?,
            U16(value) => truncate_fitted(U16, typ, value, bit_size, max_bit_size, is_sub)?,
            U32(value) => truncate_fitted(U32, typ, value, bit_size, max_bit_size, is_sub)?,
            U64(value) => truncate_fitted(U64, typ, value, bit_size, max_bit_size, is_sub)?,
            U128(value) => truncate_fitted(U128, typ, value, bit_size, max_bit_size, is_sub)?,
            I8(value) => truncate_via!(I8, typ, value, bit_size, max_bit_size, is_sub, i8, u8),
            I16(value) => truncate_via!(I16, typ, value, bit_size, max_bit_size, is_sub, i16, u16),
            I32(value) => truncate_via!(I32, typ, value, bit_size, max_bit_size, is_sub, i32, u32),
            I64(value) => truncate_via!(I64, typ, value, bit_size, max_bit_size, is_sub, i64, u64),
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

        fn fitted_bit_count<T: Into<f64>>(value: Fitted<T>) -> u32 {
            value.apply(|value| bit_count(value), |value| value.num_bits())
        }

        let bit_count = match value {
            NumericValue::Field(value) => value.num_bits(),
            // max_bit_size > 0 so u1 should always pass these checks
            NumericValue::U1(_) => return Ok(()),
            NumericValue::U8(value) => fitted_bit_count(value),
            NumericValue::U16(value) => fitted_bit_count(value),
            NumericValue::U32(value) => fitted_bit_count(value),
            NumericValue::U64(value) => {
                // u64, u128, and i64 don't impl Into<f64>
                value.apply(
                    |value| if value == 0 { 0 } else { value.ilog2() + 1 },
                    |value| value.num_bits(),
                )
            }
            NumericValue::U128(value) => value.apply(
                |value| if value == 0 { 0 } else { value.ilog2() + 1 },
                |value| value.num_bits(),
            ),
            NumericValue::I8(value) => fitted_bit_count(value),
            NumericValue::I16(value) => fitted_bit_count(value),
            NumericValue::I32(value) => fitted_bit_count(value),
            NumericValue::I64(value) => value.apply(
                |value| if value == 0 { 0 } else { value.ilog2() + 1 },
                |value| value.num_bits(),
            ),
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
                        && self.functions[&id].runtime().is_brillig()
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
                Value::ForeignFunction(name) if self.options.no_foreign_calls => {
                    return Err(InterpreterError::UnknownForeignFunctionCall { name });
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
            Ok(Value::Function(id)) => match self.functions.get(&id) {
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
        let value = Value::reference(result, element_type);
        self.define(result, value)
    }

    fn interpret_load(&mut self, address: ValueId, result: ValueId) -> IResult<()> {
        let address = self.lookup_reference(address, "load")?;

        let element = address.element.borrow();
        let Some(value) = element.as_ref() else {
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
        result: ValueId,
        side_effects_enabled: bool,
    ) -> IResult<()> {
        // When there is a problem indexing the array, but side effects are disabled,
        // define the value as uninitialized.
        let uninitialized = |this: &mut Self| {
            let typ = this.dfg().type_of_value(result);
            let value = Value::uninitialized(&typ, result);
            this.define(result, value)
        };

        let offset = self.dfg().array_offset(array, index);
        let array = self.lookup_array_or_slice(array, "array get")?;
        let length = array.elements.borrow().len() as u32;

        let index = match self.lookup_array_index(index, "array get index", length) {
            Err(InterpreterError::IndexOutOfBounds { .. }) if !side_effects_enabled => {
                return uninitialized(self);
            }
            other => other?,
        };
        let mut index = index - offset.to_u32();

        if length == 0 {
            // Accessing an array of 0-len is replaced by asserting
            // the branch is not-taken during acir-gen and
            // a zeroed type is used in case of array get
            // So we can simply replace it with uninitialized value
            if side_effects_enabled {
                return Err(InterpreterError::IndexOutOfBounds { index: index.into(), length });
            } else {
                return uninitialized(self);
            }
        }

        let element = {
            // An array_get with false side_effects_enabled is replaced
            // by a load at a valid index during acir-gen.
            if !side_effects_enabled {
                // Find a valid index
                let typ = self.dfg().type_of_value(result);
                for (i, element) in array.elements.borrow().iter().enumerate() {
                    if element.get_type() == typ {
                        index = i as u32;
                        break;
                    }
                }
            }
            let elements = array.elements.borrow();
            let element = elements
                .get(index as usize)
                .ok_or(InterpreterError::IndexOutOfBounds { index: index.into(), length })?;

            // Either return a fresh nested array (in constrained context) or just clone the element.
            if !self.in_unconstrained_context() {
                if let Some(array) = element.as_array_or_slice() {
                    // In the ACIR runtime we expect fresh arrays when accessing a nested array.
                    // If we do not clone the elements here a mutable array set afterwards could mutate
                    // not just this returned array but the array we are fetching from in this array get.
                    Value::ArrayOrSlice(ArrayValue {
                        elements: Shared::new(array.elements.borrow().to_vec()),
                        rc: array.rc,
                        element_types: array.element_types,
                        is_slice: array.is_slice,
                    })
                } else {
                    element.clone()
                }
            } else {
                element.clone()
            }
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
        result: ValueId,
        side_effects_enabled: bool,
    ) -> IResult<()> {
        let offset = self.dfg().array_offset(array, index);
        let array = self.lookup_array_or_slice(array, "array set")?;

        let result_array = if side_effects_enabled {
            let length = array.elements.borrow().len() as u32;
            let index = self.lookup_array_index(index, "array set index", length)?;
            let index = index - offset.to_u32();
            let value = self.lookup(value)?;

            let is_rc_one = *array.rc.borrow() == 1;
            let should_mutate = if self.in_unconstrained_context() { is_rc_one } else { mutable };

            if index >= length {
                return Err(InterpreterError::IndexOutOfBounds { index: index.into(), length });
            }

            if should_mutate {
                array.elements.borrow_mut()[index as usize] = value;
                Value::ArrayOrSlice(array.clone())
            } else {
                if !is_rc_one {
                    Self::decrement_rc(&array);
                }
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

    /// Decrement the ref-count of an array by 1.
    fn decrement_rc(_array: &ArrayValue) {
        // The decrement of the ref-count is currently disabled in SSA as well as the Brillig codegen,
        // but we might re-enable it in the future if the ownership optimizations change.
        // *array.rc.borrow_mut() -= 1;
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

        // The number of elements in the array must be a multiple of the number of element types
        let element_types = result_type.element_types();
        if element_types.is_empty() {
            if !elements.is_empty() {
                return Err(internal(InternalError::MakeArrayElementCountMismatch {
                    result,
                    elements_count: elements.len(),
                    types_count: element_types.len(),
                }));
            }
        } else if elements.len() % element_types.len() != 0 {
            return Err(internal(InternalError::MakeArrayElementCountMismatch {
                result,
                elements_count: elements.len(),
                types_count: element_types.len(),
            }));
        }

        // Make sure each element's type matches the one in element_types
        for (index, (element, expected_type)) in
            elements.iter().zip(element_types.iter().cycle()).enumerate()
        {
            let actual_type = element.get_type();
            if &actual_type != expected_type {
                return Err(internal(InternalError::MakeArrayElementTypeMismatch {
                    result,
                    index,
                    actual_type: actual_type.to_string(),
                    expected_type: expected_type.to_string(),
                }));
            }
        }

        let array = Value::ArrayOrSlice(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types,
            is_slice,
        });
        self.define(result, array)
    }

    fn interpret_binary(&self, binary: &Binary, side_effects_enabled: bool) -> IResult<Value> {
        let lhs_id = binary.lhs;
        let rhs_id = binary.rhs;
        let lhs = self.lookup_numeric(lhs_id, "binary op lhs")?;
        let rhs = self.lookup_numeric(rhs_id, "binary op rhs")?;
        evaluate_binary(binary, lhs, rhs, side_effects_enabled, |binary| {
            display_binary(binary, self.dfg())
        })
        .map(Value::Numeric)
    }
}

/// Applies a fallible integer binary operation on `Fitted` values, or returns an overflow error.
///
/// If one of the values are already `Unfit`, the result is an overflow.
macro_rules! apply_fit_binop_opt {
    ($lhs:expr, $rhs:expr, $f:expr, $overflow:expr) => {
        match ($lhs, $rhs) {
            (Fitted::Fit(lhs), Fitted::Fit(rhs)) => {
                $f(&lhs, &rhs).map(Fitted::Fit).ok_or_else($overflow)
            }
            _ => Err($overflow()),
        }
    };
}

/// Applies a fallible integer binary operation on `Fitted` values, promoting values to `Field` in
/// case there is an overflow, thus turning the operation infallible.
///
/// If the result is an overflow, it promotes the values to `Field` and performs the operation there.
/// If the operation is applied on `Unfit` values, and the result fits in the original numeric type,
/// it is converted back to a `Fit` value.
///
/// For example we would normally have an infallible `wrapped_add`, but we want to match ACIR
/// by not wrapping around but extending into larger bit sizes.
///
/// # Parameters
/// - `$cons`: Constructor for a `NumericValue`
/// - `$lhs`, `$rhs`: The `Fitted` values in the left-hand side and right-hand side operands.
/// - `$f`: The function to apply on the integer values if both are `Fit`; returns `None` on overflow.
/// - `$g`: The function to apply on `Field` values.
/// - `$lhs_num`, `$rhs_num`: The original `NumericValue`s.
macro_rules! apply_fit_binop {
    ($cons:expr, $lhs:expr, $rhs:expr, $f:expr, $g:expr, $lhs_num:expr, $rhs_num:expr) => {
        if let (Fitted::Fit(lhs), Fitted::Fit(rhs)) = ($lhs, $rhs) {
            let fitted = $f(&lhs, &rhs).map(Fitted::Fit).unwrap_or_else(|| {
                Fitted::Unfit($g($lhs_num.convert_to_field(), $rhs_num.convert_to_field()))
            });
            $cons(fitted)
        } else {
            let field = $g($lhs_num.convert_to_field(), $rhs_num.convert_to_field());
            let typ = $lhs_num.get_type();
            NumericValue::from_constant(field, typ).unwrap_or_else(|_| {
                let fitted = Fitted::Unfit(field);
                $cons(fitted)
            })
        }
    };
}

/// Apply a comparison operator on `Fitted` values, returning a `bool`.
///
/// This is here for the sake of `apply_int_comparison_op`, but comparing `Field` is only meaningful for equality.
/// For anything else it's best to panic, or return an error; we'll see if it comes up.
macro_rules! apply_fit_comparison_op {
    ($lhs:expr, $rhs:expr, $f:expr, $g:expr, $lhs_num:expr, $rhs_num:expr) => {{
        if let (Fitted::Fit(lhs), Fitted::Fit(rhs)) = ($lhs, $rhs) {
            $f(lhs, rhs)
        } else {
            $g($lhs_num.convert_to_field(), $rhs_num.convert_to_field())
        }
    }};
}

/// Applies an infallible integer binary operation to two `NumericValue`s.
///
/// # Parameters
/// - `$lhs`, `$rhs`: The left hand side and right hand side operands (must be the same variant).
/// - `$binary`: The binary instruction, used for error handling if types mismatch.
/// - `$f`: A function (e.g., `checked_add`) that applies the operation on the raw numeric types.
/// - `$g`: A function that performs the equivalent of `$f` on `Field` values.
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
    ($lhs:expr, $rhs:expr, $binary:expr, $f:expr, $g:expr) => {{
        use value::NumericValue::*;
        let lhs_num: value::NumericValue = $lhs;
        let rhs_num: value::NumericValue = $rhs;
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => {
                unreachable!("Expected only integer values, found field values")
            }
            (U1(_), U1(_)) => unreachable!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => apply_fit_binop!(U8, lhs, rhs, $f, $g, lhs_num, rhs_num),
            (U16(lhs), U16(rhs)) => apply_fit_binop!(U16, lhs, rhs, $f, $g, lhs_num, rhs_num),
            (U32(lhs), U32(rhs)) => apply_fit_binop!(U32, lhs, rhs, $f, $g, lhs_num, rhs_num),
            (U64(lhs), U64(rhs)) => apply_fit_binop!(U64, lhs, rhs, $f, $g, lhs_num, rhs_num),
            (U128(lhs), U128(rhs)) => apply_fit_binop!(U128, lhs, rhs, $f, $g, lhs_num, rhs_num),
            (I8(lhs), I8(rhs)) => apply_fit_binop!(I8, lhs, rhs, $f, $g, lhs_num, rhs_num),
            (I16(lhs), I16(rhs)) => apply_fit_binop!(I16, lhs, rhs, $f, $g, lhs_num, rhs_num),
            (I32(lhs), I32(rhs)) => apply_fit_binop!(I32, lhs, rhs, $f, $g, lhs_num, rhs_num),
            (I64(lhs), I64(rhs)) => apply_fit_binop!(I64, lhs, rhs, $f, $g, lhs_num, rhs_num),
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
/// - `$lhs`, `$rhs`: The left-hand side and right-hand side operands (must be the same variant).
/// - `$binary`: The binary instruction, used for diagnostics and overflow reporting.
/// - `$f`: A fallible operation function that returns an `Option<_>` (e.g., `checked_add`).
/// - `$display_binary`: A function to display the binary operation for diagnostic purposes.
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
    ($lhs:expr, $rhs:expr, $binary:expr, $f:expr, $display_binary:expr) => {{
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
                    format!("`{}` ({operator} {lhs}, {rhs})", $display_binary(binary));
                InterpreterError::Overflow { operator, instruction }
            }
        };

        match (lhs, rhs) {
            (Field(_), Field(_)) => {
                unreachable!("Expected only integer values, found field values")
            }
            (U1(_), U1(_)) => unreachable!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U8(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
            (U16(lhs), U16(rhs)) => U16(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
            (U32(lhs), U32(rhs)) => U32(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
            (U64(lhs), U64(rhs)) => U64(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
            (U128(lhs), U128(rhs)) => U128(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
            (I8(lhs), I8(rhs)) => I8(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
            (I16(lhs), I16(rhs)) => I16(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
            (I32(lhs), I32(rhs)) => I32(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
            (I64(lhs), I64(rhs)) => I64(apply_fit_binop_opt!(lhs, rhs, $f, overflow)?),
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
    ($lhs:expr, $rhs:expr, $binary:expr, $f:expr, $g:expr) => {{
        use NumericValue::*;
        let lhs_num: NumericValue = $lhs;
        let rhs_num: NumericValue = $rhs;
        match ($lhs, $rhs) {
            (Field(_), Field(_)) => {
                unreachable!("Expected only integer values, found field values")
            }
            (U1(_), U1(_)) => unreachable!("Expected only large integer values, found u1"),
            (U8(lhs), U8(rhs)) => U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num)),
            (U16(lhs), U16(rhs)) => {
                U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num))
            }
            (U32(lhs), U32(rhs)) => {
                U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num))
            }
            (U64(lhs), U64(rhs)) => {
                U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num))
            }
            (U128(lhs), U128(rhs)) => {
                U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num))
            }
            (I8(lhs), I8(rhs)) => U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num)),
            (I16(lhs), I16(rhs)) => {
                U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num))
            }
            (I32(lhs), I32(rhs)) => {
                U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num))
            }
            (I64(lhs), I64(rhs)) => {
                U1(apply_fit_comparison_op!(lhs, rhs, $f, $g, lhs_num, rhs_num))
            }
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

fn evaluate_binary(
    binary: &Binary,
    lhs: NumericValue,
    rhs: NumericValue,
    side_effects_enabled: bool,
    display_binary: impl Fn(&Binary) -> String,
) -> IResult<NumericValue> {
    let lhs_id = binary.lhs;
    let rhs_id = binary.rhs;

    if lhs.get_type() != rhs.get_type() {
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
        return Ok(NumericValue::zero(lhs.get_type()));
    }

    if let (Some(lhs), Some(rhs)) = (lhs.as_field(), rhs.as_field()) {
        return interpret_field_binary_op(lhs, binary.operator, rhs, lhs_id, rhs_id);
    }

    if let (Some(lhs), Some(rhs)) = (lhs.as_bool(), rhs.as_bool()) {
        return interpret_u1_binary_op(lhs, rhs, binary, &display_binary);
    }

    let result = match binary.operator {
        BinaryOp::Add { unchecked: false } => {
            apply_int_binop_opt!(
                lhs,
                rhs,
                binary,
                num_traits::CheckedAdd::checked_add,
                display_binary
            )
        }
        BinaryOp::Add { unchecked: true } => {
            apply_int_binop!(lhs, rhs, binary, num_traits::CheckedAdd::checked_add, |a, b| a + b)
        }
        BinaryOp::Sub { unchecked: false } => {
            apply_int_binop_opt!(
                lhs,
                rhs,
                binary,
                num_traits::CheckedSub::checked_sub,
                display_binary
            )
        }
        BinaryOp::Sub { unchecked: true } => {
            apply_int_binop!(lhs, rhs, binary, num_traits::CheckedSub::checked_sub, |a, b| a - b)
        }
        BinaryOp::Mul { unchecked: false } => {
            // Only unsigned multiplication has side effects
            apply_int_binop_opt!(
                lhs,
                rhs,
                binary,
                num_traits::CheckedMul::checked_mul,
                display_binary
            )
        }
        BinaryOp::Mul { unchecked: true } => {
            apply_int_binop!(lhs, rhs, binary, num_traits::CheckedMul::checked_mul, |a, b| a * b)
        }
        BinaryOp::Div => apply_int_binop_opt!(
            lhs,
            rhs,
            binary,
            num_traits::CheckedDiv::checked_div,
            display_binary
        ),
        BinaryOp::Mod => apply_int_binop_opt!(
            lhs,
            rhs,
            binary,
            num_traits::CheckedRem::checked_rem,
            display_binary
        ),
        BinaryOp::Eq => apply_int_comparison_op!(lhs, rhs, binary, |a, b| a == b, |a, b| a == b),
        BinaryOp::Lt => {
            apply_int_comparison_op!(lhs, rhs, binary, |a, b| a < b, |_, _| {
                // This could be the result of the DIE pass removing an `ArrayGet` and leaving a `LessThan`
                // and a `Constrain` in its place. `LessThan` implicitly includes a `RangeCheck` on
                // the operands during ACIR generation, which an `Unfit` value would fail, so we
                // cannot treat them differently here, even if we could compare the values as `u128` or `i128`.
                //
                // Instead we `Cast` the values in SSA, which should have converted our `Unfit` value
                // back to a `Fit` one with an acceptable number of bits.
                //
                // If we still hit this case, we have a problem.
                unreachable!("unfit 'lt': fit types should have been restored already")
            })
        }
        BinaryOp::And => {
            apply_int_binop!(lhs, rhs, binary, |a, b| Some(a & b), |_, _| unreachable!(
                "unfit 'and': fit types should have been restored already"
            ))
        }
        BinaryOp::Or => {
            apply_int_binop!(lhs, rhs, binary, |a, b| Some(a | b), |_, _| unreachable!(
                "unfit 'or': fit types should have been restored already"
            ))
        }
        BinaryOp::Xor => {
            apply_int_binop!(lhs, rhs, binary, |a, b| Some(a ^ b), |_, _| unreachable!(
                "unfit 'xor': fit types should have been restored already"
            ))
        }
        BinaryOp::Shl => {
            use NumericValue::*;
            let instruction = format!("`{}` ({lhs} << {rhs})", display_binary(binary));
            let over = || InterpreterError::Overflow { operator: BinaryOp::Shl, instruction };

            fn shl<A: CheckedShl>(a: &A, b: &u32) -> Option<A> {
                a.checked_shl(*b)
            }
            fn shl_into<A: CheckedShl, B: Into<u32> + Copy>(a: &A, b: &B) -> Option<A> {
                shl(a, &(*b).into())
            }
            fn shl_try<A: CheckedShl, B: TryInto<u32> + Copy>(a: &A, b: &B) -> Option<A> {
                shl(a, &(*b).try_into().ok()?)
            }

            match (lhs, rhs) {
                (Field(_), _) | (_, Field(_)) => {
                    return Err(internal(InternalError::UnsupportedOperatorForType {
                        operator: "<<",
                        typ: "Field",
                    }));
                }
                (U1(lhs), U1(rhs)) => U1(if !rhs { lhs } else { false }),
                (U8(lhs), U8(rhs)) => U8(apply_fit_binop_opt!(lhs, rhs, shl_into, over)?),
                (U16(lhs), U16(rhs)) => U16(apply_fit_binop_opt!(lhs, rhs, shl_into, over)?),
                (U32(lhs), U32(rhs)) => U32(apply_fit_binop_opt!(lhs, rhs, shl, over)?),
                (U64(lhs), U64(rhs)) => U64(apply_fit_binop_opt!(lhs, rhs, shl_try, over)?),
                (U128(lhs), U128(rhs)) => U128(apply_fit_binop_opt!(lhs, rhs, shl_try, over)?),
                (I8(lhs), I8(rhs)) => I8(apply_fit_binop_opt!(lhs, rhs, shl_try, over)?),
                (I16(lhs), I16(rhs)) => I16(apply_fit_binop_opt!(lhs, rhs, shl_try, over)?),
                (I32(lhs), I32(rhs)) => I32(apply_fit_binop_opt!(lhs, rhs, shl_try, over)?),
                (I64(lhs), I64(rhs)) => I64(apply_fit_binop_opt!(lhs, rhs, shl_try, over)?),
                _ => {
                    return Err(internal(InternalError::MismatchedTypesInBinaryOperator {
                        lhs: lhs.to_string(),
                        rhs: rhs.to_string(),
                        operator: binary.operator,
                        lhs_id: binary.lhs,
                        rhs_id: binary.rhs,
                    }));
                }
            }
        }
        BinaryOp::Shr => {
            use NumericValue::*;

            let instruction = format!("`{}` ({lhs} >> {rhs})", display_binary(binary));
            let over = || InterpreterError::Overflow { operator: BinaryOp::Shr, instruction };

            fn shr<A: CheckedShr>(a: &A, b: &u32) -> Option<A> {
                a.checked_shr(*b)
            }
            fn shr_into<A: CheckedShr, B: Into<u32> + Copy>(a: &A, b: &B) -> Option<A> {
                shr(a, &(*b).into())
            }
            fn shr_try<A: CheckedShr, B: TryInto<u32> + Copy>(a: &A, b: &B) -> Option<A> {
                shr(a, &(*b).try_into().ok()?)
            }

            match (lhs, rhs) {
                (Field(_), _) | (_, Field(_)) => {
                    return Err(internal(InternalError::UnsupportedOperatorForType {
                        operator: "<<",
                        typ: "Field",
                    }));
                }
                (U1(lhs), U1(rhs)) => U1(if !rhs { lhs } else { false }),
                (U8(lhs), U8(rhs)) => U8(apply_fit_binop_opt!(lhs, rhs, shr_into, over)?),
                (U16(lhs), U16(rhs)) => U16(apply_fit_binop_opt!(lhs, rhs, shr_into, over)?),
                (U32(lhs), U32(rhs)) => U32(apply_fit_binop_opt!(lhs, rhs, shr, over)?),
                (U64(lhs), U64(rhs)) => U64(apply_fit_binop_opt!(lhs, rhs, shr_try, over)?),
                (U128(lhs), U128(rhs)) => U128(apply_fit_binop_opt!(lhs, rhs, shr_try, over)?),
                (I8(lhs), I8(rhs)) => I8(apply_fit_binop_opt!(lhs, rhs, shr_try, over)?),
                (I16(lhs), I16(rhs)) => I16(apply_fit_binop_opt!(lhs, rhs, shr_try, over)?),
                (I32(lhs), I32(rhs)) => I32(apply_fit_binop_opt!(lhs, rhs, shr_try, over)?),
                (I64(lhs), I64(rhs)) => I64(apply_fit_binop_opt!(lhs, rhs, shr_try, over)?),
                _ => {
                    return Err(internal(InternalError::MismatchedTypesInBinaryOperator {
                        lhs: lhs.to_string(),
                        rhs: rhs.to_string(),
                        operator: binary.operator,
                        lhs_id: binary.lhs,
                        rhs_id: binary.rhs,
                    }));
                }
            }
        }
    };
    Ok(result)
}

fn interpret_field_binary_op(
    lhs: FieldElement,
    operator: BinaryOp,
    rhs: FieldElement,
    lhs_id: ValueId,
    rhs_id: ValueId,
) -> IResult<NumericValue> {
    let unsupported_operator = |operator| -> IResult<NumericValue> {
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
    Ok(result)
}

fn interpret_u1_binary_op(
    lhs: bool,
    rhs: bool,
    binary: &Binary,
    display_binary: &impl Fn(&Binary) -> String,
) -> IResult<NumericValue> {
    let overflow = || {
        let instruction = format!("`{}` ({lhs} << {rhs})", display_binary(binary));
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
                let lhs = u8::from(lhs).to_string();
                let rhs = u8::from(rhs).to_string();
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
                let lhs = format!("u1 {}", u8::from(lhs));
                let rhs = format!("u1 {}", u8::from(rhs));
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
            if rhs {
                return Err(overflow());
            } else {
                lhs
            }
        }
        BinaryOp::Shr => {
            if rhs {
                return Err(overflow());
            } else {
                lhs
            }
        }
    };
    Ok(NumericValue::U1(result))
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
    use crate::ssa::{
        interpreter::{IResult, errors::InterpreterError, value::NumericValue},
        ir::{
            instruction::{Binary, BinaryOp},
            value::ValueId,
        },
    };

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

    #[test]
    fn test_shl() {
        let binary = Binary { lhs: ValueId::new(0), rhs: ValueId::new(1), operator: BinaryOp::Shl };

        fn display(_: &Binary) -> String {
            String::new()
        }

        let i8_testcases: Vec<((i8, i8), IResult<i8>)> = vec![
            ((1, 7), Ok(-128)),
            ((2, 6), Ok(-128)),
            ((4, 5), Ok(-128)),
            ((8, 4), Ok(-128)),
            ((16, 3), Ok(-128)),
            ((32, 2), Ok(-128)),
            ((64, 1), Ok(-128)),
            ((3, 7), Ok(-128)),
            (
                (1, 8),
                Err(InterpreterError::Overflow {
                    operator: BinaryOp::Shl,
                    instruction: "`` (i8 1 << i8 8)".to_string(),
                }),
            ),
        ];

        for ((lhs, rhs), expected_result) in i8_testcases {
            assert_eq!(
                super::evaluate_binary(
                    &binary,
                    NumericValue::I8(lhs.into()),
                    NumericValue::I8(rhs.into()),
                    true,
                    display
                ),
                expected_result.map(|i| NumericValue::I8(i.into())),
                "{lhs} << {rhs}",
            );
        }
    }
}
