use std::str;

use acvm::{AcirField, FieldElement, acir::brillig::ForeignCallParam};
use iter_extended::vecmap;
use noirc_printable_type::{
    PrintableType, PrintableValueDisplay, decode_value as decode_printable_value,
};

use crate::{
    errors::RuntimeError,
    ssa::{
        ir::{
            dfg::DataFlowGraph,
            function::Function,
            instruction::{Instruction, InstructionId, Intrinsic},
            value::ValueId,
        },
        ssa_gen::Ssa,
    },
};

impl Ssa {
    /// A simple SSA pass to go through each instruction and evaluate each call
    /// to `assert_constant`, issuing an error if any arguments to the function are
    /// not constants.
    ///
    /// Note that this pass must be placed directly before loop unrolling to be
    /// useful. Any optimization passes between this and loop unrolling will cause
    /// the constants that this pass sees to be potentially different than the constants
    /// seen by loop unrolling. Furthermore, this pass cannot be a part of loop unrolling
    /// since we must go through every instruction to find all references to `assert_constant`
    /// while loop unrolling only touches blocks with loops in them.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn evaluate_static_assert_and_assert_constant(
        mut self,
    ) -> Result<Ssa, RuntimeError> {
        for function in self.functions.values_mut() {
            function.evaluate_static_assert_and_assert_constant()?;
        }
        Ok(self)
    }
}

impl Function {
    pub(crate) fn evaluate_static_assert_and_assert_constant(
        &mut self,
    ) -> Result<(), RuntimeError> {
        for block in self.reachable_blocks() {
            // Unfortunately we can't just use instructions.retain(...) here since
            // check_instruction can also return an error
            let instructions = self.dfg[block].take_instructions();
            let mut filtered_instructions = Vec::with_capacity(instructions.len());

            for instruction in instructions {
                if check_instruction(self, instruction)? {
                    filtered_instructions.push(instruction);
                }
            }

            *self.dfg[block].instructions_mut() = filtered_instructions;
        }
        Ok(())
    }
}

/// During the loop unrolling pass we also evaluate calls to `assert_constant`.
/// This is done in this pass because loop unrolling is the only pass that will error
/// if a value (the loop bounds) are not known constants.
///
/// This returns Ok(true) if the given instruction should be kept in the block and
/// Ok(false) if it should be removed.
fn check_instruction(
    function: &mut Function,
    instruction: InstructionId,
) -> Result<bool, RuntimeError> {
    let assert_constant_id = function.dfg.import_intrinsic(Intrinsic::AssertConstant);
    let static_assert_id = function.dfg.import_intrinsic(Intrinsic::StaticAssert);
    match &function.dfg[instruction] {
        Instruction::Call { func, arguments } => {
            if *func == assert_constant_id {
                evaluate_assert_constant(function, instruction, arguments)
            } else if *func == static_assert_id {
                evaluate_static_assert(function, instruction, arguments)
            } else {
                Ok(true)
            }
        }
        _ => Ok(true),
    }
}

/// Evaluate a call to `assert_constant`, returning an error if any of the elements are not
/// constants. If all of the elements are constants, Ok(false) is returned. This signifies a
/// success but also that the instruction need not be reinserted into the block being unrolled
/// since it has already been evaluated.
fn evaluate_assert_constant(
    function: &Function,
    instruction: InstructionId,
    arguments: &[ValueId],
) -> Result<bool, RuntimeError> {
    if arguments.iter().all(|arg| function.dfg.is_constant(*arg)) {
        Ok(false)
    } else {
        let call_stack = function.dfg.get_instruction_call_stack(instruction);
        Err(RuntimeError::AssertConstantFailed { call_stack })
    }
}

/// Evaluate a call to `static_assert`, returning an error if the value is false
/// or not constant (see assert_constant).
///
/// When it passes, Ok(false) is returned. This signifies a
/// success but also that the instruction need not be reinserted into the block being unrolled
/// since it has already been evaluated.
fn evaluate_static_assert(
    function: &Function,
    instruction: InstructionId,
    arguments: &[ValueId],
) -> Result<bool, RuntimeError> {
    if arguments.len() < 2 {
        panic!("ICE: static_assert called with wrong number of arguments")
    }

    // All of the arguments representing the message must be constants
    for arg in arguments.iter().skip(1) {
        if !function.dfg.is_constant(*arg) {
            let call_stack = function.dfg.get_instruction_call_stack(instruction);
            return Err(RuntimeError::StaticAssertDynamicMessage { call_stack });
        }
    }

    if function.dfg.is_constant_true(arguments[0]) {
        return Ok(false);
    }

    let call_stack = function.dfg.get_instruction_call_stack(instruction);
    if !function.dfg.is_constant(arguments[0]) {
        return Err(RuntimeError::StaticAssertDynamicPredicate { call_stack });
    }

    // To turn the arguments into a string we do the same as we'd do if the arguments
    // were passed to the built-in foreign call "print" functions.
    let mut foreign_call_params = Vec::with_capacity(arguments.len() - 1);
    for arg in arguments.iter().skip(1) {
        if !function.dfg.is_constant(*arg) {
            let call_stack = function.dfg.get_instruction_call_stack(instruction);
            return Err(RuntimeError::StaticAssertDynamicMessage { call_stack });
        }
        append_foreign_call_param(*arg, &function.dfg, &mut foreign_call_params);
    }

    let display_values = try_from_params(&foreign_call_params);
    let message = display_values.to_string();

    Err(RuntimeError::StaticAssertFailed { message, call_stack })
}

fn append_foreign_call_param(
    value: ValueId,
    dfg: &DataFlowGraph,
    foreign_call_params: &mut Vec<ForeignCallParam<FieldElement>>,
) {
    if let Some(field) = dfg.get_numeric_constant(value) {
        foreign_call_params.push(ForeignCallParam::Single(field));
    } else if let Some((values, _typ)) = dfg.get_array_constant(value) {
        let values = vecmap(values, |value| {
            dfg.get_numeric_constant(value).expect("ICE: expected constant value")
        });
        foreign_call_params.push(ForeignCallParam::Array(values));
    } else {
        panic!("ICE: expected constant value");
    }
}

// NOTE: the code here is a copy of tooling/nargo/src/foreign_calls/print.rs
// It currently isn't imported because that would end up in a circular dependency.
fn try_from_params(
    foreign_call_inputs: &[ForeignCallParam<FieldElement>],
) -> PrintableValueDisplay<FieldElement> {
    let (is_fmt_str, foreign_call_inputs) =
        foreign_call_inputs.split_last().expect("Missing foreign call inputs");

    if is_fmt_str.unwrap_field().is_one() {
        convert_fmt_string_inputs(foreign_call_inputs)
    } else {
        convert_string_inputs(foreign_call_inputs)
    }
}

fn convert_string_inputs(
    foreign_call_inputs: &[ForeignCallParam<FieldElement>],
) -> PrintableValueDisplay<FieldElement> {
    // Fetch the PrintableType from the foreign call input
    // The remaining input values should hold what is to be printed
    let (printable_type_as_values, input_values) =
        foreign_call_inputs.split_last().expect("Missing foreign call inputs");
    let printable_type = fetch_printable_type(printable_type_as_values);

    // We must use a flat map here as each value in a struct will be in a separate input value
    let mut input_values_as_fields = input_values.iter().flat_map(|param| param.fields());

    let value = decode_printable_value(&mut input_values_as_fields, &printable_type);

    PrintableValueDisplay::Plain(value, printable_type)
}

fn convert_fmt_string_inputs(
    foreign_call_inputs: &[ForeignCallParam<FieldElement>],
) -> PrintableValueDisplay<FieldElement> {
    let (message, input_and_printable_types) =
        foreign_call_inputs.split_first().expect("Missing foreign call inputs");

    let message_as_fields = message.fields();
    let message_as_string = decode_string_value(&message_as_fields);

    let (num_values, input_and_printable_types) =
        input_and_printable_types.split_first().expect("Missing foreign call inputs");

    let mut output = Vec::new();
    let num_values = num_values.unwrap_field().to_u128() as usize;

    let types_start_at = input_and_printable_types.len() - num_values;

    let mut input_iter =
        input_and_printable_types[0..types_start_at].iter().flat_map(|param| param.fields());
    for printable_type in input_and_printable_types.iter().skip(types_start_at) {
        let printable_type = fetch_printable_type(printable_type);
        let value = decode_printable_value(&mut input_iter, &printable_type);

        output.push((value, printable_type));
    }

    PrintableValueDisplay::FmtString(message_as_string, output)
}

fn fetch_printable_type(printable_type: &ForeignCallParam<FieldElement>) -> PrintableType {
    let printable_type_as_fields = printable_type.fields();
    let printable_type_as_string = decode_string_value(&printable_type_as_fields);
    let printable_type: PrintableType =
        serde_json::from_str(&printable_type_as_string).expect("Could not decode printable type");

    printable_type
}

fn decode_string_value<F: AcirField>(field_elements: &[F]) -> String {
    let string_as_slice = vecmap(field_elements, |e| {
        let mut field_as_bytes = e.to_be_bytes();
        let char_byte = field_as_bytes.pop().unwrap(); // A character in a string is represented by a u8, thus we just want the last byte of the element
        assert!(field_as_bytes.into_iter().all(|b| b == 0)); // Assert that the rest of the field element's bytes are empty
        char_byte
    });

    let final_string = str::from_utf8(&string_as_slice).unwrap();
    final_string.to_owned()
}
