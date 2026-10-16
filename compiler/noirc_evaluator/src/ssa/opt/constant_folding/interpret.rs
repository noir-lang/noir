use std::io::Empty;

use acvm::FieldElement;
use imbl::Vector;
use iter_extended::vecmap;
use rustc_hash::FxHashSet as HashSet;

use crate::ssa::{
    interpreter::{Interpreter, value::Value as InterpreterValue},
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        function::FunctionId,
        instruction::Instruction,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
};

/// Constant-argument Brillig calls that the interpreter failed to evaluate.
///
/// Many of these calls run fine when the program executes. The callee may make an oracle or
/// `print` call, which the interpreter does not perform, recurse deeper than the interpreter's
/// call stack allows, or need more steps than its step limit. Others would fail if they ran, but
/// sit on a branch the program never takes. In every case the failure is decided by the callee
/// and its arguments alone, since the interpreter is deterministic. This set lives on
/// [`Ssa`][crate::ssa::ssa_gen::Ssa] so that every Constant Folding pass of the pipeline skips
/// those calls, instead of interpreting each of them again up to the point where it fails.
///
/// Passes between two Constant Folding runs can change a callee, for instance by making it cheaper
/// or removing a `print`, so a call skipped here might be foldable later. Giving up that fold
/// bounds the work to one attempt per callee and arguments for the whole compilation.
#[derive(Debug, Default, Clone)]
pub(crate) struct UnfoldableCalls(pub(super) HashSet<(FunctionId, Vec<ConstantArgument>)>);

/// The interpreter steps Constant Folding may spend on each constant-argument Brillig call, and
/// the steps it has spent on calls that failed.
///
/// [`UnfoldableCalls`] keeps a failing call from being interpreted more than once, but distinct
/// call sites still each pay up to the full step limit. So the steps spent on failing calls are
/// also capped: once they add up to `step_limit`, every later call may only take
/// `step_limit / REDUCED_STEP_LIMIT_DIVISOR` steps. Failing calls then cost at most `step_limit`,
/// plus that reduced limit per further call site, for the whole compilation.
///
/// Once the cap is reached, a call that needs more than the reduced limit is not folded, even if it
/// would have succeeded.
#[derive(Debug, Clone)]
pub(crate) struct InterpreterBudget {
    /// Most interpreter steps a single call may take.
    pub(super) step_limit: usize,
    /// Total steps spent on calls that failed.
    pub(super) steps_wasted: usize,
}

impl Default for InterpreterBudget {
    fn default() -> Self {
        Self::with_step_limit(super::DEFAULT_INTERPRETER_STEP_LIMIT)
    }
}

/// Once failing calls have spent `step_limit` steps in total, each later call may take
/// `step_limit / REDUCED_STEP_LIMIT_DIVISOR` steps.
const REDUCED_STEP_LIMIT_DIVISOR: usize = 100;

impl InterpreterBudget {
    pub(super) fn with_step_limit(step_limit: usize) -> Self {
        Self { step_limit, steps_wasted: 0 }
    }

    /// Most steps the next call may take.
    fn call_step_limit(&self) -> usize {
        if self.steps_wasted < self.step_limit {
            self.step_limit
        } else {
            self.step_limit / REDUCED_STEP_LIMIT_DIVISOR
        }
    }
}

/// A constant argument of a Brillig call, in a form that can be hashed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ConstantArgument {
    Numeric(FieldElement, NumericType),
    Array(Vec<ConstantArgument>, Type),
}

impl ConstantArgument {
    fn from_value(value_id: ValueId, dfg: &DataFlowGraph) -> Self {
        let typ = dfg.type_of_value(value_id);
        match &*typ {
            Type::Numeric(numeric_type) => {
                let constant =
                    dfg.get_numeric_constant(value_id).expect("Should have a numeric constant");
                ConstantArgument::Numeric(constant, *numeric_type)
            }
            Type::Array(..) | Type::Vector(..) => {
                let (elements, _) =
                    dfg.get_array_constant(value_id).expect("Should have an array constant");
                let elements = elements.iter().map(|e| Self::from_value(*e, dfg)).collect();
                ConstantArgument::Array(elements, (*typ).clone())
            }
            Type::Reference(..) | Type::Function => {
                unreachable!("Cannot be a constant value")
            }
        }
    }
}

/// Evaluates Brillig calls with constant arguments so that Constant Folding can replace them
/// with their results.
pub(crate) struct ConstantCallEvaluator<'ssa> {
    interpreter: Interpreter<'ssa, Empty>,
    unfoldable: UnfoldableCalls,
    budget: InterpreterBudget,
}

impl<'ssa> ConstantCallEvaluator<'ssa> {
    /// `interpreter` must already have interpreted the program's globals.
    pub(crate) fn new(
        interpreter: Interpreter<'ssa, Empty>,
        unfoldable: UnfoldableCalls,
        budget: InterpreterBudget,
    ) -> Self {
        Self { interpreter, unfoldable, budget }
    }

    /// The unfoldable calls and the budget, including what this evaluator found and spent.
    pub(crate) fn into_parts(self) -> (UnfoldableCalls, InterpreterBudget) {
        (self.unfoldable, self.budget)
    }
}

/// Checks if the given instruction is a call to a function with all constant arguments.
/// If so, we can try to evaluate that function and replace the results with the evaluation results.
pub(super) fn try_interpret_call(
    instruction: &Instruction,
    block: BasicBlockId,
    dfg: &mut DataFlowGraph,
    evaluator: &mut ConstantCallEvaluator,
) -> Option<Vec<ValueId>> {
    let evaluation_result = evaluate_const_argument_call(instruction, evaluator, dfg);

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
    evaluator: &mut ConstantCallEvaluator,
    dfg: &DataFlowGraph,
) -> EvaluationResult {
    let interpreter = &mut evaluator.interpreter;
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

    let call = (*func_id, vecmap(arguments, |arg| ConstantArgument::from_value(*arg, dfg)));
    if evaluator.unfoldable.0.contains(&call) {
        return EvaluationResult::CannotEvaluate;
    }

    let interpreter_args =
        arguments.iter().map(|arg| const_ir_value_to_interpreter_value(*arg, dfg)).collect();

    interpreter.set_step_limit(Some(evaluator.budget.call_step_limit()));
    match interpreter.interpret_function(*func_id, interpreter_args) {
        Ok(result_values) => EvaluationResult::Evaluated(result_values),
        Err(_) => {
            evaluator.unfoldable.0.insert(call);
            evaluator.budget.steps_wasted += interpreter.steps_taken();
            EvaluationResult::CannotEvaluate
        }
    }
}

/// Converts a constant [SSA value][Value] into an [interpreter value][InterpreterValue] for execution.
fn const_ir_value_to_interpreter_value(value_id: ValueId, dfg: &DataFlowGraph) -> InterpreterValue {
    let typ = dfg.type_of_value(value_id);
    match &*typ {
        Type::Numeric(numeric_type) => {
            let constant =
                dfg.get_numeric_constant(value_id).expect("Should have a numeric constant");
            InterpreterValue::from_constant(constant, *numeric_type)
                .expect("Should be a valid constant")
        }
        Type::Reference(..) => unreachable!("References cannot be constant values"),
        Type::Array(element_types, length) => {
            let (array_constant, _) =
                dfg.get_array_constant(value_id).expect("Should have an array constant");
            let mut elements = Vec::new();
            for element in array_constant {
                elements.push(const_ir_value_to_interpreter_value(element, dfg));
            }
            InterpreterValue::array_with_length(elements, element_types.to_vec(), *length)
        }
        Type::Vector(element_types) => {
            let (array_constant, _) =
                dfg.get_array_constant(value_id).expect("Should have an array constant");
            let mut elements = Vec::new();
            for element in array_constant {
                elements.push(const_ir_value_to_interpreter_value(element, dfg));
            }
            InterpreterValue::vector(elements, element_types.clone())
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
            let constant = value.as_numeric().expect("Should be numeric").to_field();
            dfg.make_constant(constant, numeric_type)
        }
        Type::Array(element_types, length) => {
            let InterpreterValue::ArrayOrVector(array) = value else {
                unreachable!("Expected an ArrayOrVector");
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
        Type::Vector(element_types) => {
            let InterpreterValue::ArrayOrVector(array) = value else {
                unreachable!("Expected an ArrayOrVector");
            };

            let mut elements = Vector::new();
            for element in array.elements.unwrap_or_clone() {
                elements.push_back(interpreter_value_to_ir_value(element, dfg, block_id));
            }

            let instruction = Instruction::MakeArray { elements, typ: Type::Vector(element_types) };

            let instruction_id = dfg.make_instruction(instruction, None);
            dfg[block_id].instructions_mut().push(instruction_id);
            dfg.instruction_result::<1>(instruction_id)[0]
        }
        Type::Function | Type::Reference(..) => unreachable!("Cannot be a constant value"),
    }
}
