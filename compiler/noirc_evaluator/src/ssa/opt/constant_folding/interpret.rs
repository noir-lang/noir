use std::io::Empty;

use im::Vector;
use iter_extended::vecmap;

use crate::ssa::{
    interpreter::{Interpreter, value::Value as InterpreterValue},
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        instruction::Instruction,
        types::Type,
        value::{Value, ValueId},
    },
};

/// Checks if the given instruction is a call to a function with all constant arguments.
/// If so, we can try to evaluate that function and replace the results with the evaluation results.
pub(super) fn try_interpret_call(
    instruction: &Instruction,
    block: BasicBlockId,
    dfg: &mut DataFlowGraph,
    interpreter: Option<&mut Interpreter<Empty>>,
) -> Option<Vec<ValueId>> {
    let evaluation_result = evaluate_const_argument_call(instruction, interpreter?, dfg);

    match evaluation_result {
        EvaluationResult::NotABrilligCall | EvaluationResult::CannotEvaluate => None,
        EvaluationResult::Evaluated(const_results) => {
            let new_results = vecmap(const_results, |const_result| {
                interpreter_value_to_ir_value(const_result, dfg, block)
            });
            Some(new_results)
        }
    }
}

/// Result of trying to evaluate an instruction (any instruction) in this pass.
enum EvaluationResult {
    /// Nothing was done because the instruction wasn't a call to a brillig function,
    /// or some arguments to it were not constants.
    NotABrilligCall,
    /// The instruction was a call to a brillig function, but we couldn't evaluate it.
    /// This can occur in the situation where the brillig function reaches a "trap" or a foreign call opcode.
    CannotEvaluate,
    /// The instruction was a call to a brillig function and we were able to evaluate it,
    /// returning [SSA interpreter][Interpreter] [values][InterpreterValue].
    Evaluated(Vec<InterpreterValue>),
}

/// Tries to evaluate an instruction if it's a call where all its arguments are constant.
/// We do this by interpreting the function's SSA to calculate the result.
fn evaluate_const_argument_call(
    instruction: &Instruction,
    interpreter: &mut Interpreter<Empty>,
    dfg: &mut DataFlowGraph,
) -> EvaluationResult {
    let Instruction::Call { func: func_id, arguments } = instruction else {
        return EvaluationResult::NotABrilligCall;
    };

    let func_value = &dfg[*func_id];
    let Value::Function(func_id) = func_value else {
        return EvaluationResult::NotABrilligCall;
    };

    let Some(_) = interpreter.functions().get(func_id) else {
        return EvaluationResult::NotABrilligCall;
    };

    // Ensure all arguments to the call are constant
    if !arguments.iter().all(|argument| dfg.is_constant(*argument)) {
        return EvaluationResult::CannotEvaluate;
    }

    let interpreter_args =
        arguments.iter().map(|arg| const_ir_value_to_interpreter_value(*arg, dfg)).collect();

    let Ok(result_values) = interpreter.interpret_function(*func_id, interpreter_args) else {
        return EvaluationResult::CannotEvaluate;
    };

    EvaluationResult::Evaluated(result_values)
}

/// Converts a constant [SSA value][Value] into an [interpreter value][InterpreterValue] for execution.
fn const_ir_value_to_interpreter_value(value_id: ValueId, dfg: &DataFlowGraph) -> InterpreterValue {
    let typ = dfg.type_of_value(value_id);
    match typ {
        Type::Numeric(numeric_type) => {
            let constant =
                dfg.get_numeric_constant(value_id).expect("Should have a numeric constant");
            InterpreterValue::from_constant(constant, numeric_type)
                .expect("Should be a valid constant")
        }
        Type::Reference(_) => unreachable!("References cannot be constant values"),
        Type::Array(element_types, _) => {
            let (array_constant, _) =
                dfg.get_array_constant(value_id).expect("Should have an array constant");
            let mut elements = Vec::new();
            for element in array_constant {
                elements.push(const_ir_value_to_interpreter_value(element, dfg));
            }
            InterpreterValue::array(elements, element_types.to_vec())
        }
        Type::List(element_types) => {
            let (array_constant, _) =
                dfg.get_array_constant(value_id).expect("Should have an array constant");
            let mut elements = Vec::new();
            for element in array_constant {
                elements.push(const_ir_value_to_interpreter_value(element, dfg));
            }
            InterpreterValue::list(elements, element_types)
        }
        Type::Function => unreachable!("Functions cannot be constant values"),
    }
}

/// Converts a constant [interpreter value][InterpreterValue] back into an SSA constant.
fn interpreter_value_to_ir_value(
    value: InterpreterValue,
    dfg: &mut DataFlowGraph,
    block_id: BasicBlockId,
) -> ValueId {
    let typ = value.get_type();
    match typ {
        Type::Numeric(numeric_type) => {
            let constant = value.as_numeric().expect("Should be numeric").convert_to_field();
            dfg.make_constant(constant, numeric_type)
        }
        Type::Array(element_types, length) => {
            let array = match value {
                InterpreterValue::ArrayOrList(array) => array,
                _ => unreachable!("Expected an ArrayOrList"),
            };

            let mut elements = Vector::new();
            for element in array.elements.unwrap_or_clone() {
                elements.push_back(interpreter_value_to_ir_value(element, dfg, block_id));
            }

            let instruction =
                Instruction::MakeArray { elements, typ: Type::Array(element_types, length) };

            let instruction_id = dfg.make_instruction(instruction, None);
            dfg[block_id].instructions_mut().push(instruction_id);
            dfg.instruction_result::<1>(instruction_id)[0]
        }
        Type::List(element_types) => {
            let array = match value {
                InterpreterValue::ArrayOrList(array) => array,
                _ => unreachable!("Expected an ArrayOrList"),
            };

            let mut elements = Vector::new();
            for element in array.elements.unwrap_or_clone() {
                elements.push_back(interpreter_value_to_ir_value(element, dfg, block_id));
            }

            let instruction = Instruction::MakeArray { elements, typ: Type::List(element_types) };

            let instruction_id = dfg.make_instruction(instruction, None);
            dfg[block_id].instructions_mut().push(instruction_id);
            dfg.instruction_result::<1>(instruction_id)[0]
        }
        Type::Function | Type::Reference(_) => unreachable!("Cannot be a constant value"),
    }
}
