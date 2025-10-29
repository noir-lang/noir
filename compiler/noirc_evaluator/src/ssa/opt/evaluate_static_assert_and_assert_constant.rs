//! A simple SSA pass to go through each instruction and evaluate:
//! - each call to `assert_constant`, issuing an error if any arguments to the function
//!   are not constants
//! - each call to `static_assert`, issuing an error if the assertion does not hold,
//!   the value to test is not a constant, or the message is dynamic.
//!
//! Note that this pass must be placed directly before [`loop unrolling`](super::unrolling) to be
//! useful. Any optimization passes between this and loop unrolling will cause
//! the constants that this pass sees to be potentially different than the constants
//! seen by loop unrolling. Furthermore, this pass cannot be a part of loop unrolling
//! since we must go through every instruction to find all references to `assert_constant`
//! while loop unrolling only touches blocks with loops in them.
use acvm::{FieldElement, acir::brillig::ForeignCallParam};
use iter_extended::vecmap;
use noirc_printable_type::{PrintableValueDisplay, TryFromParamsError};
use rustc_hash::FxHashSet as HashSet;

use crate::{
    errors::RuntimeError,
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            cfg::ControlFlowGraph,
            dfg::DataFlowGraph,
            function::Function,
            instruction::{Instruction, InstructionId, Intrinsic},
            value::ValueId,
        },
        opt::Loops,
        ssa_gen::Ssa,
    },
};

impl Ssa {
    /// See [`evaluate_static_assert_and_assert_constant`][self] module for more information.
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
    fn evaluate_static_assert_and_assert_constant(&mut self) -> Result<(), RuntimeError> {
        let assert_constant_id = self.dfg.get_intrinsic(Intrinsic::AssertConstant).copied();
        let static_assert_id = self.dfg.get_intrinsic(Intrinsic::StaticAssert).copied();
        if assert_constant_id.is_none() && static_assert_id.is_none() {
            // If there are no calls to either intrinsic there's nothing to evaluate
            return Ok(());
        }

        let blocks_within_empty_loop = get_blocks_within_empty_loop(self);

        for block in self.reachable_blocks() {
            // Unfortunately we can't just use instructions.retain(...) here since
            // check_instruction can also return an error
            let instructions = self.dfg[block].take_instructions();
            let mut filtered_instructions = Vec::with_capacity(instructions.len());

            let inside_empty_loop = blocks_within_empty_loop.contains(&block);
            for instruction in instructions {
                if check_instruction(
                    self,
                    instruction,
                    assert_constant_id,
                    static_assert_id,
                    inside_empty_loop,
                )? {
                    filtered_instructions.push(instruction);
                }
            }

            *self.dfg[block].instructions_mut() = filtered_instructions;
        }
        Ok(())
    }
}

/// Returns all of a function's block that are part of empty loops.
fn get_blocks_within_empty_loop(function: &Function) -> HashSet<BasicBlockId> {
    let loops = Loops::find_all(function);

    let cfg = ControlFlowGraph::with_function(function);
    let mut blocks_within_empty_loop = HashSet::default();
    for loop_ in loops.yet_to_unroll {
        let Ok(pre_header) = loop_.get_pre_header(function, &cfg) else {
            // If the loop does not have a preheader we skip checking whether the loop is empty
            continue;
        };
        let const_bounds = loop_.get_const_bounds(&function.dfg, pre_header);

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

    blocks_within_empty_loop
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
    assert_constant_id: Option<ValueId>,
    static_assert_id: Option<ValueId>,
    inside_empty_loop: bool,
) -> Result<bool, RuntimeError> {
    match &function.dfg[instruction] {
        Instruction::Call { func, arguments } => {
            let is_assert_constant = Some(*func) == assert_constant_id;
            let is_static_assert = Some(*func) == static_assert_id;

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
        append_foreign_call_param(*arg, &function.dfg, instruction, &mut foreign_call_params)?;
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
                panic!("ICE: could not decode printable type {error:?}")
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
    instruction: InstructionId,
    foreign_call_params: &mut Vec<ForeignCallParam<FieldElement>>,
) -> Result<(), RuntimeError> {
    if let Some(field) = dfg.get_numeric_constant(value) {
        foreign_call_params.push(ForeignCallParam::Single(field));
        Ok(())
    } else if let Some((values, _typ)) = dfg.get_array_constant(value) {
        let values = vecmap(values, |value| {
            dfg.get_numeric_constant(value).expect("ICE: expected constant value")
        });
        foreign_call_params.push(ForeignCallParam::Array(values));
        Ok(())
    } else {
        let call_stack = dfg.get_instruction_call_stack(instruction);
        Err(RuntimeError::StaticAssertDynamicMessage { call_stack })
    }
}

#[cfg(test)]
mod test {
    use crate::{assert_ssa_snapshot, errors::RuntimeError, ssa::ssa_gen::Ssa};

    #[test]
    fn do_not_fail_on_successful_assert_constant() {
        let src = r"
        acir(inline) fn main f0 {
          b0():
            call assert_constant(Field 1)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.evaluate_static_assert_and_assert_constant().unwrap();

        // The assertion held and it was removed from the SSA.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return
        }
        ");
    }

    #[test]
    fn fail_on_unsuccessful_assert_constant() {
        let src = r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            call assert_constant(v0)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        assert!(matches!(
            ssa.evaluate_static_assert_and_assert_constant().err().unwrap(),
            RuntimeError::AssertConstantFailed { .. }
        ));
    }

    #[test]
    fn do_not_fail_on_assert_constant_in_empty_loop() {
        let src = r"
        acir(inline) fn main f0 {
          b0():
            v2 = call f1() -> [Field; 0]
            jmp b1(u32 0)
          b1(v0: u32):
            v4 = lt v0, u32 0
            jmpif v4 then: b2, else: b3
          b2():
            call assert_constant(v2)
            v7 = unchecked_add v0, u32 1
            jmp b1(v7)
          b3():
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
            jmp b1(u32 0)
          b1(v0: u32):
            v4 = lt v0, u32 0
            jmpif v4 then: b2, else: b3
          b2():
            v6 = unchecked_add v0, u32 1
            jmp b1(v6)
          b3():
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

    #[test]
    fn do_not_fail_on_successful_static_assert() {
        let src = r#"
        acir(inline) fn main f0 {
        b0():
            v13 = make_array b"Assertion failed"
            v24 = make_array b"{\"kind\":\"string\",\"length\":16}"
            call static_assert(u1 1, v13, v24, u1 0)
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.evaluate_static_assert_and_assert_constant().unwrap();

        // The assertion held and it was removed from the SSA.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0():
            v13 = make_array b"Assertion failed"
            v24 = make_array b"{\"kind\":\"string\",\"length\":16}"
            return
        }
        "#);
    }

    #[test]
    fn fail_on_unsuccessful_static_assert() {
        let src = r#"
        acir(inline) fn main f0 {
        b0():
            v13 = make_array b"Assertion failed"
            v24 = make_array b"{\"kind\":\"string\",\"length\":16}"
            call static_assert(u1 0, v13, v24, u1 0)
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let Err(RuntimeError::StaticAssertFailed { message, .. }) =
            ssa.evaluate_static_assert_and_assert_constant()
        else {
            panic!("Expected a static assert failure");
        };
        assert_eq!(message, "Assertion failed");
    }

    #[test]
    fn fail_on_static_assert_without_a_constant_value() {
        let src = r#"
        acir(inline) fn main f0 {
        b0(v0: u1):
            v13 = make_array b"Assertion failed"
            v24 = make_array b"{\"kind\":\"string\",\"length\":16}"
            call static_assert(v0, v13, v24, u1 0)
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let Err(RuntimeError::StaticAssertDynamicPredicate { message, .. }) =
            ssa.evaluate_static_assert_and_assert_constant()
        else {
            panic!("Expected a static assert dynamic predicate failure");
        };
        assert_eq!(message, "Assertion failed");
    }

    #[test]
    fn fail_on_static_assert_with_a_dynamic_message() {
        let src = r#"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v18 = make_array b"Assertion failed: {x}"
            v21 = make_array b"{\"kind\":\"field\"}"
            call static_assert(u1 0, v18, Field 1, v0, v21, u1 1)
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let Err(RuntimeError::StaticAssertDynamicMessage { .. }) =
            ssa.evaluate_static_assert_and_assert_constant()
        else {
            panic!("Expected a static assert dynamic message failure");
        };
    }
}
