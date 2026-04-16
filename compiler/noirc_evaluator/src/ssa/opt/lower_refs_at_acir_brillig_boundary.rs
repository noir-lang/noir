//! We can't natively pass references from acir into brillig, but this is desired for
//! array ownership reasons, so this pass creates wrappers for brillig functions accepting references
//! which are called from acir. These wrappers accept values instead of references and simply allocate
//! internally before calling the function they wrap.
//!
//! This pass only handles explicit calls to brillig functions from acir with reference arguments.
//! Nested references are not supported, and this pass must be run after defunctionalization but
//! otherwise has no other ordering constraints.
//!
//! Generally, it is good to have this pass earlier so that mem2reg can see the reference(s) in
//! question are no longer passed to function calls and are thus eligible for optimization.
//!
//! # What this pass does
//!
//! For every ACIR function that calls a Brillig function with one or more reference arguments it:
//!
//! 1. Inserts a `Load` from each reference argument immediately before the call, producing the
//!    plain value that was stored into the reference.
//! 2. Creates a thin wrapper Brillig function that
//!    a. accepts a plain value instead of the reference, and
//!    b. allocates a local slot, stores the value, and calls the original Brillig function with
//!    the resulting reference.
//! 3. Replaces the original call in the ACIR function with a call to the wrapper passing the
//!    loaded value directly.
//!
//! # Preconditions
//!
//! - This pass must be run after defunctionalization.

#![allow(unused)]
use std::sync::Arc;

use iter_extended::vecmap;
use noirc_errors::call_stack::CallStackId;
use noirc_frontend::monomorphization::ast::InlineType;

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{
        basic_block::BasicBlockId,
        dfg::GlobalsGraph,
        function::{Function, FunctionId, RuntimeType},
        instruction::{Instruction, InstructionId},
        types::Type,
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Materialize immutable reference arguments at every ACIR→Brillig call boundary.
    ///
    /// See the module documentation for a detailed explanation.
    pub(crate) fn lower_refs_at_acir_brillig_boundary(mut self) -> Self {
        // We're going to be adding new functions so we need to collect
        // pre-existing ids beforehand.
        let acir_functions =
            vecmap(self.functions.iter().filter(|(_, f)| f.runtime().is_acir()), |(&id, _)| id);

        for function in acir_functions {
            for transform in collect_transformations_for(&self, function) {
                let wrapper_id = self.add_fn(|wrapper_id| {
                    build_wrapper(
                        wrapper_id,
                        transform.callee_id,
                        &transform.callee_name,
                        &transform.callee_param_types,
                        transform.call_result_types.clone(),
                        transform.callee_globals.clone(),
                    )
                });

                let acir_func = self.functions.get_mut(&function).unwrap();
                let wrapper_val = acir_func.dfg.import_function(wrapper_id);
                transform.apply(acir_func, wrapper_val);
            }
        }

        self
    }
}

/// Data needed to rewrite one call site within an ACIR function.
struct Transformation {
    block_id: BasicBlockId,
    call_id: InstructionId,
    callee_id: FunctionId,
    callee_name: String,
    callee_param_types: Vec<Type>,
    callee_globals: Arc<GlobalsGraph>,
    original_args: Vec<ValueId>,
    call_result_types: Vec<Type>,
}

/// Find all calls in `func_id` that cross the ACIR→Brillig boundary with reference arguments.
fn collect_transformations_for(ssa: &Ssa, func_id: FunctionId) -> Vec<Transformation> {
    let function = &ssa.functions[&func_id];
    let mut out = Vec::new();

    for block_id in function.reachable_blocks() {
        for &instr_id in function.dfg[block_id].instructions() {
            if let Some(t) = try_collect_transformation(ssa, function, block_id, instr_id) {
                out.push(t);
            }
        }
    }

    out
}

/// Inspect a single instruction and return a `Transformation` if it is a call
/// to a Brillig function that passes reference arguments.
fn try_collect_transformation(
    ssa: &Ssa,
    function: &Function,
    block_id: BasicBlockId,
    call_id: InstructionId,
) -> Option<Transformation> {
    let Instruction::Call { func: callee_id, arguments } = &function.dfg[call_id] else {
        return None;
    };
    let Value::Function(callee_id) = function.dfg[*callee_id] else {
        return None;
    };
    let callee = &ssa.functions[&callee_id];
    if !callee.runtime().is_brillig() {
        return None;
    }
    if !arguments.iter().any(|&arg| matches!(*function.dfg.type_of_value(arg), Type::Reference(..)))
    {
        return None;
    }

    Some(Transformation {
        block_id,
        call_id,
        callee_id,
        callee_name: callee.name().to_string(),
        callee_param_types: vecmap(callee.parameters(), |&p| {
            callee.dfg.type_of_value(p).into_owned()
        }),
        callee_globals: callee.dfg.globals.clone(),
        original_args: arguments.clone(),
        call_result_types: vecmap(function.dfg.instruction_results(call_id), |&v| {
            function.dfg.type_of_value(v).into_owned()
        }),
    })
}

impl Transformation {
    /// Rewrite a single call site in `acir_func`:
    /// - Inserts a `Load` for each reference argument immediately before the call.
    /// - Replaces the call instruction to target `wrapper` with the loaded values.
    fn apply(self, acir_func: &mut Function, wrapper: ValueId) {
        let call_stack = acir_func.dfg.get_instruction_call_stack_id(self.call_id);

        let instructions = acir_func.dfg[self.block_id].take_instructions();
        let mut new_args = self.original_args;

        for &instr_id in &instructions {
            if instr_id == self.call_id {
                insert_loads_for_ref_args(acir_func, self.block_id, call_stack, &mut new_args);

                let arguments = std::mem::take(&mut new_args);
                acir_func.dfg[instr_id] = Instruction::Call { func: wrapper, arguments };
            }
            acir_func.dfg[self.block_id].instructions_mut().push(instr_id);
        }
    }
}

/// For each reference-typed argument in `args`, insert a `Load` instruction into `block`
/// and replace the argument with the loaded value.
fn insert_loads_for_ref_args(
    function: &mut Function,
    block_id: BasicBlockId,
    call_stack: CallStackId,
    args: &mut [ValueId],
) {
    for arg in args {
        let Type::Reference(inner_type, _) = function.dfg.type_of_value(*arg).into_owned() else {
            continue;
        };
        let load = function.dfg.insert_instruction_and_results_without_simplification(
            Instruction::Load { address: *arg },
            block_id,
            Some(vec![(*inner_type).clone()]),
            call_stack,
        );
        *arg = load.first();
    }
}

/// Build a thin Brillig wrapper function:
/// - Accepts plain values in place of reference parameters.
/// - Allocates a local slot for each reference parameter, stores the value.
/// - Calls the original Brillig function with the resulting references.
fn build_wrapper(
    wrapper_id: FunctionId,
    callee_id: FunctionId,
    callee_name: &str,
    callee_param_types: &[Type],
    call_result_types: Vec<Type>,
    callee_globals: Arc<GlobalsGraph>,
) -> Function {
    let mut builder = FunctionBuilder::new(format!("{callee_name}_ref_wrapper"), wrapper_id);

    builder.set_runtime(RuntimeType::Brillig(InlineType::Inline));
    builder.set_globals(callee_globals);

    // Change each reference parameter to a non-reference parameter holding its element type.
    let params = vecmap(callee_param_types, |typ| {
        let param_type = match typ {
            Type::Reference(inner, _) => (**inner).clone(),
            _ => typ.clone(),
        };
        builder.add_parameter(param_type)
    });

    // Build the argument list for the inner call.
    let inner_args = vecmap(params.iter().copied().enumerate(), |(pos, param_val)| {
        if let Type::Reference(inner, mutable) = &callee_param_types[pos] {
            let alloc = builder.insert_allocate_with_mutability(inner.as_ref().clone(), *mutable);
            builder.insert_store(alloc, param_val);
            alloc
        } else {
            param_val
        }
    });

    let callee_val = builder.import_function(callee_id);
    let results = builder.insert_call(callee_val, inner_args, call_result_types).to_vec();
    builder.terminate_with_return(results);

    builder.current_function
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    /// Single reference argument: a load must be emitted immediately before the
    /// call (not at the end of the block), and the call must be redirected to a
    /// freshly created wrapper.
    #[test]
    fn test_single_ref_arg() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            call f1(v0)
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: &mut Field):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_refs_at_acir_brillig_boundary();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v2 = load v0 -> Field
            call f2(v2)
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: &mut Field):
            return
        }
        brillig(inline) fn bar_ref_wrapper f2 {
          b0(v0: Field):
            v1 = allocate -> &mut Field
            store v0 at v1
            call f1(v1)
            return
        }
        ");
    }

    /// Multiple reference arguments: every reference gets its own load, all
    /// placed before the (rewritten) call.
    #[test]
    fn test_multiple_ref_args() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v1 = allocate -> &mut Field
            store Field 2 at v1
            call f1(v0, v1)
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: &mut Field, v1: &mut Field):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_refs_at_acir_brillig_boundary();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v2 = allocate -> &mut Field
            store Field 2 at v2
            v4 = load v0 -> Field
            v5 = load v2 -> Field
            call f2(v4, v5)
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: &mut Field, v1: &mut Field):
            return
        }
        brillig(inline) fn bar_ref_wrapper f2 {
          b0(v0: Field, v1: Field):
            v2 = allocate -> &mut Field
            store v0 at v2
            v3 = allocate -> &mut Field
            store v1 at v3
            call f1(v2, v3)
            return
        }
        ");
    }

    /// Mixed arguments: only the reference arguments receive loads; plain-value
    /// arguments are forwarded unchanged.
    #[test]
    fn test_mixed_args() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = allocate -> &mut Field
            store Field 1 at v1
            call f1(v1, v0)
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: &mut Field, v1: Field):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_refs_at_acir_brillig_boundary();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = allocate -> &mut Field
            store Field 1 at v1
            v3 = load v1 -> Field
            call f2(v3, v0)
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: &mut Field, v1: Field):
            return
        }
        brillig(inline) fn bar_ref_wrapper f2 {
          b0(v0: Field, v1: Field):
            v2 = allocate -> &mut Field
            store v0 at v2
            call f1(v2, v1)
            return
        }
        ");
    }

    /// When no reference arguments are present the pass must be a no-op.
    #[test]
    fn test_no_ref_args_is_noop() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            call f1(v0)
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: Field):
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::lower_refs_at_acir_brillig_boundary);
    }

    /// The call site is in a non-entry block; the load must still be inserted
    /// immediately before the call (not somewhere else in the function).
    #[test]
    fn test_call_in_non_entry_block() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            jmpif v0 then: b1(), else: b2()
          b1():
            store Field 1 at v1
            call f1(v1)
            jmp b2()
          b2():
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: &mut Field):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.lower_refs_at_acir_brillig_boundary();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            jmpif v0 then: b1(), else: b2()
          b1():
            store Field 1 at v1
            v4 = load v1 -> Field
            call f2(v4)
            jmp b2()
          b2():
            return
        }
        brillig(inline) fn bar f1 {
          b0(v0: &mut Field):
            return
        }
        brillig(inline) fn bar_ref_wrapper f2 {
          b0(v0: Field):
            v1 = allocate -> &mut Field
            store v0 at v1
            call f1(v1)
            return
        }
        ");
    }

    /// Running defunctionalize before this pass converts first-class function
    /// calls into direct calls. When the resulting ACIR→Brillig direct call
    /// passes a reference, the pass must still lower it correctly.
    #[test]
    fn test_after_defunctionalize() {
        // An ACIR caller takes a first-class Brillig function and calls it
        // with a reference argument.  After defunctionalization the indirect
        // call becomes a direct ACIR→Brillig call that our pass must fix.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            call f2(f3, v0)
            return
        }
        acir(inline) fn caller f2 {
          b0(v0: function, v1: &mut Field):
            call v0(v1)
            return
        }
        brillig(inline) fn consumer f3 {
          b0(v0: &mut Field):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.defunctionalize();
        let ssa = ssa.lower_refs_at_acir_brillig_boundary();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            call f1(Field 2, v0)
            return
        }
        acir(inline) fn caller f1 {
          b0(v0: Field, v1: &mut Field):
            call f3(v1)
            return
        }
        brillig(inline) fn consumer f2 {
          b0(v0: &mut Field):
            return
        }
        acir(inline_always) pure fn apply_dummy f3 {
          b0(v0: &mut Field):
            return
        }
        ");
    }
}
