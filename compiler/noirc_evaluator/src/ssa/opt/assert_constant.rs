use acvm::{FieldElement, acir::brillig::ForeignCallParam};
use fxhash::FxHashSet as HashSet;
use iter_extended::vecmap;
use noirc_printable_type::{PrintableValueDisplay, TryFromParamsError};

use crate::{
    errors::RuntimeError,
    ssa::{
        ir::{
            cfg::ControlFlowGraph,
            dfg::DataFlowGraph,
            function::Function,
            instruction::{Instruction, InstructionId, Intrinsic},
            value::ValueId,
        },
        ssa_gen::Ssa,
    },
};

use super::unrolling::Loops;

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
        let loops = Loops::find_all(self);

        let cfg = ControlFlowGraph::with_function(self);
        let mut blocks_within_empty_loop = HashSet::default();
        for loop_ in loops.yet_to_unroll {
            let Ok(pre_header) = loop_.get_pre_header(self, &cfg) else {
                // If the loop does not have a preheader we skip hoisting loop invariants for this loop
                continue;
            };
            let const_bounds = loop_.get_const_bounds(self, pre_header);

            let does_execute = const_bounds
                .and_then(|(lower_bound, upper_bound)| {
                    upper_bound.reduce(lower_bound, |u, l| u > l, |u, l| u > l)
                })
                // We default to `true` if the bounds are dynamic so that we still
                // evaluate static assertion in dynamic loops.
                .unwrap_or(true);

            if !does_execute {
                blocks_within_empty_loop.extend(loop_.blocks);
            }
        }

        for block in self.reachable_blocks() {
            // Unfortunately we can't just use instructions.retain(...) here since
            // check_instruction can also return an error
            let instructions = self.dfg[block].take_instructions();
            let mut filtered_instructions = Vec::with_capacity(instructions.len());

            let inside_empty_loop = blocks_within_empty_loop.contains(&block);
            for instruction in instructions {
                if check_instruction(self, instruction, inside_empty_loop)? {
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
    inside_empty_loop: bool,
) -> Result<bool, RuntimeError> {
    let assert_constant_id = function.dfg.get_intrinsic(Intrinsic::AssertConstant);
    let static_assert_id = function.dfg.get_intrinsic(Intrinsic::StaticAssert);
    if assert_constant_id.is_none() && static_assert_id.is_none() {
        return Ok(true);
    }

    match &function.dfg[instruction] {
        Instruction::Call { func, arguments } => {
            let is_assert_constant = Some(*func) == assert_constant_id.copied();
            let is_static_assert = Some(*func) == static_assert_id.copied();

            // Skip assertions inside known empty loops
            if inside_empty_loop && (is_assert_constant || is_static_assert) {
                return Ok(false);
            }

            if is_assert_constant {
                evaluate_assert_constant(function, instruction, arguments)
            } else if is_static_assert {
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

    if function.dfg.is_constant_true(arguments[0]) {
        return Ok(false);
    }

    let message = match PrintableValueDisplay::<FieldElement>::try_from_params(&foreign_call_params)
    {
        Ok(display_values) => display_values.to_string(),
        Err(err) => match err {
            TryFromParamsError::MissingForeignCallInputs => {
                panic!("ICE: missing foreign call inputs")
            }
            TryFromParamsError::ParsingError(error) => {
                panic!("ICE: could not decode printable type {:?}", error)
            }
        },
    };

    let call_stack = function.dfg.get_instruction_call_stack(instruction);
    if !function.dfg.is_constant(arguments[0]) {
        return Err(RuntimeError::StaticAssertDynamicPredicate { message, call_stack });
    }

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

#[cfg(test)]
mod test {
    use crate::{assert_ssa_snapshot, errors::RuntimeError, ssa::ssa_gen::Ssa};

    #[test]
    fn do_not_fail_on_assert_constant_in_empty_loop() {
        let src = r"
        acir(inline) fn main f0 {
          b0():
            v2 = call f1() -> [Field; 0]
            v4, v5 = call as_slice(v2) -> (u32, [Field])
            v6 = make_array [] : [Field]
            v7 = allocate -> &mut u32
            store u32 0 at v7
            v9 = allocate -> &mut [Field]
            store v6 at v9
            jmp b1(u32 0)
          b1(v0: u32):
            v10 = lt v0, u32 0
            jmpif v10 then: b2, else: b3
          b2():
            v13 = lt v0, u32 0
            constrain v13 == u1 1
            v15 = load v7 -> u32
            v16 = load v9 -> [Field]
            call assert_constant(v2)
            v19, v20 = call slice_push_back(v15, v16) -> (u32, [Field])
            store v19 at v7
            store v20 at v9
            v22 = unchecked_add v0, u32 1
            jmp b1(v22)
          b3():
            v11 = load v7 -> u32
            v12 = load v9 -> [Field]
            return
        }
        brillig(inline) fn foo f1 {
          b0():
            v0 = make_array [] : [Field; 0]
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.evaluate_static_assert_and_assert_constant().unwrap();

        // We expected the assert constant which would have otherwise returned a runtime error
        // to be removed from the SSA.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = call f1() -> [Field; 0]
            v4, v5 = call as_slice(v2) -> (u32, [Field])
            v6 = make_array [] : [Field]
            v7 = allocate -> &mut u32
            store u32 0 at v7
            v9 = allocate -> &mut [Field]
            store v6 at v9
            jmp b1(u32 0)
          b1(v0: u32):
            v10 = lt v0, u32 0
            jmpif v10 then: b2, else: b3
          b2():
            v13 = lt v0, u32 0
            constrain v13 == u1 1
            v15 = load v7 -> u32
            v16 = load v9 -> [Field]
            v18, v19 = call slice_push_back(v15, v16) -> (u32, [Field])
            store v18 at v7
            store v19 at v9
            v21 = unchecked_add v0, u32 1
            jmp b1(v21)
          b3():
            v11 = load v7 -> u32
            v12 = load v9 -> [Field]
            return
        }
        brillig(inline) fn foo f1 {
          b0():
            v0 = make_array [] : [Field; 0]
            return v0
        }
        ");
    }

    #[test]
    fn fail_on_assert_constant_in_dynamic_loop() {
        let src = r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            jmp b1(v0)
          b1(v2: u32):
            v3 = lt v2, v1                                   
            jmpif v3 then: b2, else: b3
          b2():
            call assert_constant(v0)                          
            v6 = unchecked_add v2, u32 1                      
            jmp b1(v6)
          b3():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        assert!(matches!(
            ssa.evaluate_static_assert_and_assert_constant().err().unwrap(),
            RuntimeError::AssertConstantFailed { .. }
        ));
    }
}
