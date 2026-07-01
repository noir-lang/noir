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
use crate::ssa::ir::{
    instruction::binary::{
        BinaryEvaluationResult, convert_signed_integer_to_field_element, eval_constant_binary_op,
        truncate, truncate_field, try_convert_field_element_to_signed_integer,
    },
    printer::display_binary,
    types::NumericType,
};
use acvm::{AcirField, FieldElement};
use errors::{InternalError, InterpreterError, MAX_UNSIGNED_BIT_SIZE};
use iter_extended::{try_vecmap, vecmap};
use itertools::Itertools;
use noirc_frontend::Shared;
use rustc_hash::FxHashMap as HashMap;
use value::{ArrayValue, NumericValue, ReferenceValue};

pub mod errors;
mod intrinsics;
pub(crate) mod tests;
pub mod value;

use value::Value;

/// Maximum number of recursive calls allowed at comptime.
const MAX_INTERPRETER_CALL_STACK_SIZE: usize = 1000;

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
        interpreter.interpret_function(function, args)
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

    /// Increment the step counter, or return [`InterpreterError::OutOfBudget`].
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
        }
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

            if *expected_type != actual_type {
                // Special case for ZST (Zero-Sized Type) arrays: Allow length mismatches.
                // In early SSA passes, ZST arrays like [(); 3] are represented with empty element lists.
                // Later optimization passes will fix the representation.
                let types_compatible = match (&*expected_type, &actual_type) {
                    (Type::Array(expected_elem, _), Type::Array(actual_elem, actual_len)) => {
                        expected_elem == actual_elem
                            && expected_elem.is_empty()
                            && actual_len.to_usize() == 0
                    }
                    // Reference mutability doesn't affect runtime behavior —
                    // the interpreter treats &T and &mut T identically.
                    (Type::Reference(expected_elem, _), Type::Reference(actual_elem, _)) => {
                        expected_elem == actual_elem
                    }
                    _ => false,
                };

                if !types_compatible {
                    return Err(internal(InternalError::ValueTypeDoesNotMatchReturnType {
                        value_id: id,
                        expected_type: expected_type.to_string(),
                        actual_type: actual_type.to_string(),
                    }));
                }
            }
        }

        self.call_context_mut().scope.insert(id, value);

        Ok(())
    }

    /// Interpret the global instructions.
    ///
    /// Once this is complete, the interpreter can be reused for multiple
    /// function calls within the same SSA.
    pub(crate) fn interpret_globals(&mut self) -> IResult<()> {
        assert_eq!(self.call_stack.len(), 1, "should be in the global context");
        let Some((_, function)) = self.functions.first_key_value() else {
            return Ok(());
        };

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
                super::ir::value::Value::ForeignFunction { name, .. } => {
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

    /// Interpret an entry point, assuming the globals have already been interpreted.
    ///
    /// This resets any previous call stack and step counter.
    pub(crate) fn interpret_function(
        &mut self,
        function_id: FunctionId,
        arguments: Vec<Value>,
    ) -> IResults {
        self.step_counter = 0;
        self.call_stack.truncate(1);
        self.call_function(function_id, arguments)
    }

    /// Interpret a function call.
    ///
    /// Unlike `interpret_function` this does not reset the state;
    /// it is meant to be used for internal calls.
    fn call_function(&mut self, function_id: FunctionId, mut arguments: Vec<Value>) -> IResults {
        self.call_stack.push(CallContext::new(function_id));

        if self.call_stack.len() >= MAX_INTERPRETER_CALL_STACK_SIZE {
            let call_stack = self
                .call_stack
                .iter()
                .skip(1)
                .map(|ctx| {
                    let id = ctx
                        .called_function
                        .expect("all but the first global context has a called function");
                    let name = self.functions[&id].name().to_string();
                    (id, name)
                })
                .collect();
            return Err(InterpreterError::StackOverflow { call_stack });
        }

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

            for (parameter, argument) in block.parameters().iter().zip_eq(arguments) {
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
                    then_arguments,
                    else_destination,
                    else_arguments,
                    call_stack: _,
                }) => {
                    (block_id, arguments) = if self.lookup_bool(*condition, "jmpif condition")? {
                        (*then_destination, self.lookup_all(then_arguments)?)
                    } else {
                        (*else_destination, self.lookup_all(else_arguments)?)
                    };
                    if self.options.trace {
                        println!("jump to {block_id}");
                    }
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

        if self.options.trace
            && let Some(context) = self.call_stack.last()
            && let Some(function_id) = context.called_function
        {
            let function = &self.functions[&function_id];
            println!("back in function {} ({})", function_id, function.name());
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
            super::ir::value::Value::ForeignFunction { name, .. } => {
                Value::ForeignFunction(name.clone())
            }
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

    fn lookup_array_or_vector(
        &self,
        value_id: ValueId,
        instruction: &'static str,
    ) -> IResult<ArrayValue> {
        self.lookup_helper(value_id, instruction, "array or vector", Value::as_array_or_vector)
    }

    /// Look up an array index.
    ///
    /// If the value exists but is an out-of-range `u32`, returns `IndexOutOfBounds`.
    fn lookup_array_index(
        &self,
        value_id: ValueId,
        instruction: &'static str,
        length: u32,
    ) -> IResult<u32> {
        self.lookup_helper(value_id, instruction, "u32", Value::as_u32).map_err(|e| {
            if matches!(e, InterpreterError::Internal(InternalError::TypeError { .. }))
                && let Ok(Value::Numeric(value)) = self.lookup(value_id)
                && value.get_type() == NumericType::unsigned(32)
                && !value.is_in_range()
            {
                return InterpreterError::IndexOutOfBounds { index: value.to_field(), length };
            }
            e
        })
    }

    fn lookup_bytes(&self, value_id: ValueId, instruction: &'static str) -> IResult<Vec<u8>> {
        let array = self.lookup_array_or_vector(value_id, instruction)?;
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
        let array = self.lookup_array_or_vector(value_id, instruction)?;
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
        let array = self.lookup_array_or_vector(value_id, instruction)?;
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
        let array = self.lookup_array_or_vector(value_id, instruction)?;
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
            // Cast in SSA relabels the value's type, keeping its bit pattern. Like ACIR (where a
            // cast is a no-op on the underlying field) it does not range-check: a value that
            // overflowed its source type via unchecked field arithmetic keeps its extended bits.
            // Narrowing casts are preceded by an explicit `truncate`, so no truncation is needed here.
            Instruction::Cast(value, numeric_type) => {
                let value = self.lookup_numeric(*value, "cast")?;
                let field = value.to_field();
                let result = Value::int_from_field(field, *numeric_type)?;
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

        if num_value.as_field().is_some() {
            return Err(internal(InternalError::UnsupportedOperatorForType {
                operator: "!",
                typ: "Field",
            }));
        }
        if let Some(value) = num_value.as_bool() {
            return self.define(result, Value::Numeric(NumericValue::bool(!value)));
        }

        // Based on AcirContext::not_var: `!x == max - x` where `max == 2^bit_size - 1`, computed on
        // the bit pattern reduced into range. The result is always in range.
        let typ = num_value.get_type();
        let bit_size = num_value.bit_size();
        let reduced = truncate_field(num_value.to_field(), bit_size);
        let max =
            FieldElement::from(2u128).pow(&FieldElement::from(bit_size)) - FieldElement::one();
        let new_result = NumericValue::int_from_field(max - reduced, typ)?;
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
        let typ = value.get_type();
        if bit_size == 0 {
            return Err(internal(InternalError::TruncateToZeroBits { value_id, max_bit_size }));
        }

        if value.as_bool().is_some() {
            return self.define(result, Value::Numeric(value));
        }

        let mut field = value.to_field();

        // An out-of-range value produced by a subtraction holds the field `p - delta` (the field
        // negative of the underflow amount), whose low `bit_size` bits are not the wrapped result.
        // Mirroring `acir::Context::convert_ssa_truncate`, add the integer modulus `2^max_bit_size`
        // before truncating so the low bits become the wrapped value. This provenance cannot be
        // recovered from the value alone, so it is keyed on the producing instruction being a `sub`.
        if !value.is_in_range() && self.produced_by_sub(value_id) {
            field += FieldElement::from(2u128).pow(&FieldElement::from(max_bit_size));
        }

        let truncated = NumericValue::int_from_field(truncate_field(field, bit_size), typ)?;
        self.define(result, Value::Numeric(truncated))
    }

    /// Whether `value_id` is the result of a subtraction. An out-of-range value with this provenance
    /// holds the field-negative `p - delta` of an underflow, which needs the modulus added back
    /// before truncation to recover the wrapped result (see [`reduce_operand_to_range`]). The
    /// provenance cannot be recovered from the value alone, hence this instruction lookup.
    fn produced_by_sub(&self, value_id: ValueId) -> bool {
        if let IrValue::Instruction { instruction, .. } = self.dfg()[value_id] {
            matches!(
                self.dfg()[instruction],
                Instruction::Binary(Binary { operator: BinaryOp::Sub { .. }, .. })
            )
        } else {
            false
        }
    }

    fn interpret_range_check(
        &self,
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

        // max_bit_size > 0 so u1 always passes; every other variant stores its bit pattern as a
        // field, so the number of bits is read directly off that field (matching ACIR, which
        // range-checks the field representation).
        if value.as_bool().is_some() {
            return Ok(());
        }
        let bit_count = value.to_field().num_bits();

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
                        for argument in &mut arguments {
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
            self.uninitialized_call_results(&function, argument_ids, results)?
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

        for (result, new_result) in results.iter().zip_eq(new_results) {
            self.define(*result, new_result)?;
        }
        Ok(())
    }

    /// Create uninitialized results for a call that was skipped due to disabled side effects.
    ///
    /// For vector intrinsics, we create properly-sized zeroed vectors rather than empty ones,
    /// to avoid out-of-bounds error after Remove `IfElse` that need to do `array_get` to
    /// merge the vector from a 'side effect disabled' branch.
    fn uninitialized_call_results(
        &self,
        function: &Value,
        argument_ids: &[ValueId],
        results: &[ValueId],
    ) -> IResult<Vec<Value>> {
        use crate::ssa::ir::instruction::Intrinsic;
        // Get the length of the vector
        if let Value::Intrinsic(intrinsic) = function {
            let input_vector_info = match intrinsic {
                Intrinsic::VectorPushBack
                | Intrinsic::VectorPushFront
                | Intrinsic::VectorInsert
                | Intrinsic::VectorPopBack
                | Intrinsic::VectorPopFront
                | Intrinsic::VectorRemove => {
                    let vec = self.lookup_array_or_vector(
                        argument_ids[1],
                        "uninitialized vector intrinsic",
                    )?;
                    Some((vec.elements.borrow().len(), vec.element_types.clone()))
                }
                _ => None,
            };

            if let Some((input_len, element_types)) = input_vector_info {
                let element_count = element_types.len();
                let output_len = match intrinsic {
                    Intrinsic::VectorPushBack
                    | Intrinsic::VectorPushFront
                    | Intrinsic::VectorInsert => input_len + element_count,
                    Intrinsic::VectorPopBack
                    | Intrinsic::VectorPopFront
                    | Intrinsic::VectorRemove => input_len.saturating_sub(element_count),
                    _ => unreachable!(),
                };

                return Ok(vecmap(results, |result| {
                    let typ = self.dfg().type_of_value(*result);
                    if matches!(*typ, Type::Vector(_)) {
                        Value::uninitialized_vector(&element_types, output_len, *result)
                    } else {
                        Value::uninitialized(&typ, *result)
                    }
                }));
            }
        }

        Ok(vecmap(results, |result| {
            let typ = self.dfg().type_of_value(*result);
            Value::uninitialized(&typ, *result)
        }))
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

            // Immutable references are allowed to cross the constrained->unconstrained
            // boundary. Mutable references are rejected earlier by the frontend type check.
            Value::Reference(_) => Ok(()),

            Value::ArrayOrVector(array_value) => {
                let mut elements = array_value.elements.borrow().to_vec();
                for element in &mut elements {
                    Self::reset_array_state(element)?;
                }
                array_value.elements = Shared::new(elements);
                array_value.rc = Shared::new(1);
                Ok(())
            }
        }
    }

    fn interpret_allocate(&mut self, result: ValueId) -> IResult<()> {
        let result_type = self.dfg().type_of_value(result).into_owned();
        let (element_type, mutable) = match result_type {
            Type::Reference(element_type, mutable) => (element_type, mutable),
            other => unreachable!(
                "Result of allocate should always be a reference type, but found {other}"
            ),
        };
        let value = Value::reference(result, element_type, mutable);
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

    fn interpret_store(&self, address: ValueId, value: ValueId) -> IResult<()> {
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
        let array = self.lookup_array_or_vector(array, "array get")?;
        let length = array.elements.borrow().len() as u32;

        // Per `Instruction::requires_acir_gen_predicate`, in Brillig an
        // `array_get` is pure-in-isolation: the OOB check is inserted as a
        // separate constraint, not part of the access itself. Match that here
        // so the interpreter agrees with the Brillig VM on dead/unused gets.
        let oob_is_pure = self.current_function().runtime().is_brillig();

        let index = match self.lookup_array_index(index, "array get index", length) {
            Err(InterpreterError::IndexOutOfBounds { .. })
                if !side_effects_enabled || oob_is_pure =>
            {
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
            if side_effects_enabled && !oob_is_pure {
                return Err(InterpreterError::IndexOutOfBounds { index: index.into(), length });
            } else {
                return uninitialized(self);
            }
        }

        if oob_is_pure && index >= length {
            return uninitialized(self);
        }

        let element = {
            // An array_get with false side_effects_enabled is replaced
            // by a load at a valid index during acir-gen.
            if !side_effects_enabled {
                // Find a valid index
                let typ = self.dfg().type_of_value(result);
                for (i, element) in array.elements.borrow().iter().enumerate() {
                    if element.get_type() == *typ {
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
                if let Some(array) = element.as_array_or_vector() {
                    // In the ACIR runtime we expect fresh arrays when accessing a nested array.
                    // If we do not clone the elements here a mutable array set afterwards could mutate
                    // not just this returned array but the array we are fetching from in this array get.
                    Value::ArrayOrVector(ArrayValue {
                        elements: Shared::new(array.elements.borrow().to_vec()),
                        rc: array.rc,
                        element_types: array.element_types,
                        length: array.length,
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
        let array = self.lookup_array_or_vector(array, "array set")?;

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
                Value::ArrayOrVector(array)
            } else {
                if !is_rc_one {
                    Self::decrement_rc(&array);
                }
                let mut elements = array.elements.borrow().to_vec();
                elements[index as usize] = value;
                let elements = Shared::new(elements);
                let rc = Shared::new(1);
                let element_types = array.element_types.clone();
                let length = array.length;
                Value::ArrayOrVector(ArrayValue { elements, rc, element_types, length })
            }
        } else {
            // Side effects are disabled, return the original array
            Value::ArrayOrVector(array)
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
            let array = self.lookup_array_or_vector(value_id, "inc_rc")?;
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
            let array = self.lookup_array_or_vector(value_id, "dec_rc")?;
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
        let length = if let Type::Array(_, length) = result_type { Some(*length) } else { None };

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
            if !actual_type.canonical_eq(expected_type) {
                return Err(internal(InternalError::MakeArrayElementTypeMismatch {
                    result,
                    index,
                    actual_type: actual_type.to_string(),
                    expected_type: expected_type.to_string(),
                }));
            }
        }

        let array = Value::ArrayOrVector(ArrayValue {
            elements: Shared::new(elements),
            rc: Shared::new(1),
            element_types,
            length,
        });
        self.define(result, array)
    }

    fn interpret_binary(&self, binary: &Binary, side_effects_enabled: bool) -> IResult<Value> {
        let lhs_id = binary.lhs;
        let rhs_id = binary.rhs;
        let lhs = self.lookup_numeric(lhs_id, "binary op lhs")?;
        let rhs = self.lookup_numeric(rhs_id, "binary op rhs")?;
        let is_brillig = self.current_function().runtime().is_brillig();
        let lhs_from_sub = self.produced_by_sub(lhs_id);
        let rhs_from_sub = self.produced_by_sub(rhs_id);
        evaluate_binary(
            binary,
            lhs,
            rhs,
            side_effects_enabled,
            is_brillig,
            lhs_from_sub,
            rhs_from_sub,
            |binary| display_binary(binary, self.dfg()),
        )
        .map(Value::Numeric)
    }
}

/// Evaluate an integer (non-`Field`, non-`u1`) binary operation in the field+type model.
///
/// `lhs`/`rhs` are integer values of the same type whose stored field is the two's-complement bit
/// pattern (which may be out of range in ACIR mode). The runtime selects overflow behaviour:
/// Brillig wraps in fixed-width registers, while ACIR carries the extended field and range-checks at
/// checked operations. Comparisons, bitwise ops, shifts, div/mod and *signed* checked arithmetic
/// reduce their operands and reuse [`eval_constant_binary_op`]; only unchecked arithmetic and
/// *unsigned* checked arithmetic depend on the runtime.
fn evaluate_integer_binary(
    binary: &Binary,
    lhs: NumericValue,
    rhs: NumericValue,
    is_brillig: bool,
    lhs_from_sub: bool,
    rhs_from_sub: bool,
    display_binary: impl Fn(&Binary) -> String,
) -> IResult<NumericValue> {
    use BinaryOp::{Add, Mul, Sub};

    let operator = binary.operator;
    let typ = lhs.get_type();
    let bit_size = lhs.bit_size();
    let lhs_field = lhs.to_field();
    let rhs_field = rhs.to_field();

    let overflow = || overflow_error(binary, &display_binary, lhs, rhs);

    // Field arithmetic for add/sub/mul (no wrapping, no reduction).
    let field_arith = || match operator {
        Add { .. } => lhs_field + rhs_field,
        Sub { .. } => lhs_field - rhs_field,
        Mul { .. } => lhs_field * rhs_field,
        _ => unreachable!("field_arith called on a non-arithmetic operator"),
    };

    match operator {
        // Unchecked arithmetic: Brillig wraps to the bit size; ACIR extends in the field.
        Add { unchecked: true } | Sub { unchecked: true } | Mul { unchecked: true } => {
            let result = if is_brillig {
                let a = bits_u128(lhs_field, bit_size);
                let b = bits_u128(rhs_field, bit_size);
                let wrapped = match operator {
                    Add { .. } => a.wrapping_add(b),
                    Sub { .. } => a.wrapping_sub(b),
                    Mul { .. } => a.wrapping_mul(b),
                    _ => unreachable!(),
                };
                FieldElement::from(truncate(wrapped, bit_size))
            } else {
                field_arith()
            };
            NumericValue::int_from_field(result, typ)
        }

        // Unsigned checked arithmetic. ACIR computes in the field and range-checks the result, so an
        // out-of-range operand or an overflowing result is rejected; Brillig does true fixed-width
        // checked arithmetic. (They only diverge once values can exceed the field modulus, i.e.
        // `u128`, but keeping them separate is faithful to both.)
        Add { unchecked: false } | Sub { unchecked: false } | Mul { unchecked: false }
            if !lhs.is_signed() =>
        {
            if is_brillig {
                eval_via_constant_binary_op(
                    lhs_field,
                    rhs_field,
                    lhs_from_sub,
                    rhs_from_sub,
                    operator,
                    typ,
                    binary,
                    &overflow,
                )
            } else {
                let value = NumericValue::int_from_field(field_arith(), typ)?;
                if value.is_in_range() { Ok(value) } else { Err(overflow()) }
            }
        }

        // Shifts wrap the value to the bit size (a shift amount >= the bit width is an overflow,
        // matching `checked_shl`/`checked_shr`); they are not folded as checked overflows.
        BinaryOp::Shl | BinaryOp::Shr => {
            let value_bits = bits_u128(lhs_field, bit_size);
            let shift = bits_u128(rhs_field, bit_size);
            if shift >= u128::from(bit_size) {
                return Err(overflow());
            }
            let shift = shift as u32;
            let result = match operator {
                BinaryOp::Shl => {
                    FieldElement::from(truncate(value_bits.wrapping_shl(shift), bit_size))
                }
                // Arithmetic shift for signed values, logical shift for unsigned.
                BinaryOp::Shr if lhs.is_signed() => {
                    let signed = try_convert_field_element_to_signed_integer(
                        FieldElement::from(value_bits),
                        bit_size,
                    )
                    .expect("a reduced signed value converts");
                    convert_signed_integer_to_field_element(signed >> shift, bit_size)
                }
                BinaryOp::Shr => FieldElement::from(value_bits >> shift),
                _ => unreachable!(),
            };
            NumericValue::int_from_field(result, typ)
        }

        // Signed checked arithmetic, div/mod, comparisons and bitwise ops all reduce their
        // operands; reuse the constant-folder's semantics.
        _ => eval_via_constant_binary_op(
            lhs_field,
            rhs_field,
            lhs_from_sub,
            rhs_from_sub,
            operator,
            typ,
            binary,
            &overflow,
        ),
    }
}

/// Reduce an operand field to its `bit_size`-bit two's-complement representative.
///
/// A value that overflowed its type via unchecked field arithmetic may be out of range. If it
/// escaped *upward* (add/mul) its low `bit_size` bits are already the wrapped representative, so a
/// plain truncation suffices. A value from a *subtraction* that underflowed holds `p - delta`, whose
/// low bits are contaminated by `p`; adding `2^bit_size` first wraps it (in the field) to the small
/// positive `2^bit_size - delta` so the low bits become the wrapped result. Mirrors the underflow
/// handling in [`Interpreter::interpret_truncate`] / `acir::Context::convert_ssa_truncate`. The
/// provenance cannot be recovered from the value alone, so `from_sub` is supplied by the caller.
fn reduce_operand_to_range(field: FieldElement, bit_size: u32, from_sub: bool) -> FieldElement {
    let out_of_range = field.num_bits() > bit_size;
    let field = if from_sub && out_of_range {
        field + FieldElement::from(2u128).pow(&FieldElement::from(bit_size))
    } else {
        field
    };
    truncate_field(field, bit_size)
}

/// Apply [`eval_constant_binary_op`] to two operand fields and map the result into the interpreter's
/// value/error types. `eval_constant_binary_op` truncates its operands to the type's bit size, so
/// this is the "reduce then compute" path shared by both runtimes.
#[allow(clippy::too_many_arguments)]
fn eval_via_constant_binary_op(
    lhs_field: FieldElement,
    rhs_field: FieldElement,
    lhs_from_sub: bool,
    rhs_from_sub: bool,
    operator: BinaryOp,
    typ: NumericType,
    binary: &Binary,
    overflow: &impl Fn() -> InterpreterError,
) -> IResult<NumericValue> {
    // Reduce the operands to their bit-pattern representatives first. `eval_constant_binary_op`
    // also truncates internally, but its signed conversion requires the field to fit in `u128`,
    // which an extended (out-of-range) ACIR operand such as `p - delta` does not.
    let bit_size = typ.bit_size::<FieldElement>();
    let lhs_field = reduce_operand_to_range(lhs_field, bit_size, lhs_from_sub);
    let rhs_field = reduce_operand_to_range(rhs_field, bit_size, rhs_from_sub);

    match eval_constant_binary_op(lhs_field, rhs_field, operator, typ) {
        BinaryEvaluationResult::Success(field, result_type) => {
            NumericValue::int_from_field(field, result_type)
        }
        BinaryEvaluationResult::Failure(_) => {
            let divisor_is_zero = matches!(operator, BinaryOp::Div | BinaryOp::Mod)
                && truncate_field(rhs_field, typ.bit_size::<FieldElement>()).is_zero();
            if divisor_is_zero {
                Err(division_by_zero(binary.lhs, lhs_field, binary.rhs, rhs_field))
            } else {
                Err(overflow())
            }
        }
        // Only an overflowing `shl` reports this; treat it as an overflow.
        BinaryEvaluationResult::CouldNotEvaluate => Err(overflow()),
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_binary(
    binary: &Binary,
    lhs: NumericValue,
    rhs: NumericValue,
    side_effects_enabled: bool,
    is_brillig: bool,
    lhs_from_sub: bool,
    rhs_from_sub: bool,
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

    evaluate_integer_binary(
        binary,
        lhs,
        rhs,
        is_brillig,
        lhs_from_sub,
        rhs_from_sub,
        display_binary,
    )
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
        BinaryOp::Add { unchecked: _ } => NumericValue::field(lhs + rhs),
        BinaryOp::Sub { unchecked: _ } => NumericValue::field(lhs - rhs),
        BinaryOp::Mul { unchecked: _ } => NumericValue::field(lhs * rhs),
        BinaryOp::Div => {
            if rhs.is_zero() {
                return Err(division_by_zero(lhs_id, lhs, rhs_id, rhs));
            }
            NumericValue::field(lhs / rhs)
        }
        BinaryOp::Mod => return unsupported_operator("%"),
        BinaryOp::Eq => NumericValue::bool(lhs == rhs),
        BinaryOp::Lt => NumericValue::bool(lhs < rhs),
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
    let overflow = || overflow_error(binary, display_binary, lhs, rhs);

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
            if !lhs && rhs {
                return Err(overflow());
            } else {
                lhs ^ rhs
            }
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
                return Err(division_by_zero(lhs_id, u8::from(lhs), rhs_id, u8::from(rhs)));
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
                return Err(division_by_zero(lhs_id, lhs, rhs_id, rhs));
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
    Ok(NumericValue::bool(result))
}

#[cfg_attr(not(test), expect(dead_code))]
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

/// Reduce `field` to its `bit_size`-bit pattern as a `u128`.
///
/// ACIR operands may carry an extended (out-of-range) field; truncating to the bit size yields the
/// canonical bit pattern, which always fits in `u128` for the integer widths the interpreter models.
fn bits_u128(field: FieldElement, bit_size: u32) -> u128 {
    truncate_field(field, bit_size).try_into_u128().expect("a reduced value fits in u128")
}

/// Build an `Overflow` error for `binary`, rendering the instruction as
/// `` `<ssa instruction>` (<operator> <lhs>, <rhs>) ``.
fn overflow_error(
    binary: &Binary,
    display_binary: impl Fn(&Binary) -> String,
    lhs: impl std::fmt::Display,
    rhs: impl std::fmt::Display,
) -> InterpreterError {
    let operator = binary.operator;
    let instruction = format!("`{}` ({operator} {lhs}, {rhs})", display_binary(binary));
    InterpreterError::Overflow { operator, instruction }
}

/// Build a `DivisionByZero` error, rendering each operand with its `Display`.
fn division_by_zero(
    lhs_id: ValueId,
    lhs: impl std::fmt::Display,
    rhs_id: ValueId,
    rhs: impl std::fmt::Display,
) -> InterpreterError {
    InterpreterError::DivisionByZero { lhs_id, lhs: lhs.to_string(), rhs_id, rhs: rhs.to_string() }
}

fn internal(error: InternalError) -> InterpreterError {
    InterpreterError::Internal(error)
}
