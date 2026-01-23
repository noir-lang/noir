//! Removes parameters from `std::hint::black_box` which are best left ignored.
//!
//! The purpose of the `black_box` function is to give a _hint_ to the
//! compiler to minimize optimizations and try to preserve the inputs.
//! It is primarily meant to be used with numeric values, but accepts any input.
//!
//! When some inputs are being prevented from being simplified away, it can cause
//! unintended issues with other SSA passes. Since the `black_box` function has
//! no guarantees about what it does, other than not introducing undefined behavior,
//! this pass allows us to _ignore_ parameters where it's not clear what `black_box`
//! should do, but it is causing issues.
//!
//! This way the user is free to give a hint to the compiler to try to apply the `black_box` effect,
//! and the compiler is free to ignore it, without having to reject the code.

use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        dfg::InsertInstructionResult,
        function::Function,
        instruction::{Hint, Instruction, Intrinsic},
        types::Type,
        value::Value,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Removes parameters from calls to the `black_box` builtin which are best left ignored.
    ///
    /// Currently these are just function values, but we might expand them in the future.
    ///
    /// This step should come before defunctionalization.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn black_box_bypass(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.black_box_bypass();
        }
        self
    }
}

impl Function {
    /// Removes parameters from calls to the `black_box` builtin which are best left ignored:
    /// * function values, so the passing of a (constrained, unconstrained) pair to `black_box`:
    ///     * doesn't prevent the returned values from being inlined as the only relevant call target
    ///     * doesn't prevent the otherwise unused unconstrained variant from being removed
    fn black_box_bypass(&mut self) {
        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            let Instruction::Call { func, arguments } = instruction else {
                return;
            };
            if !matches!(context.dfg[*func], Value::Intrinsic(Intrinsic::Hint(Hint::BlackBox))) {
                return;
            }

            let arg_types = vecmap(arguments, |arg| context.dfg.type_of_value(*arg));

            if !arg_types.iter().any(black_box_should_ignore) {
                return;
            }

            let func = *func;
            let arguments = arguments.clone();
            let results = Vec::from(context.dfg.instruction_results(instruction_id));
            assert_eq!(results.len(), arg_types.len(), "black_box is an identity function");

            // We won't use the current instruction; insert a new one instead with fewer results.
            context.remove_current_instruction();

            // Collect results we don't want to ignore.
            let mut old_results = Vec::new();
            let mut new_args = Vec::new();
            let mut new_arg_types = Vec::new();

            for (i, typ) in arg_types.into_iter().enumerate() {
                if black_box_should_ignore(&typ) {
                    // Bypass the black_box by making the result equal the arg.
                    context.replace_value(results[i], arguments[i]);
                } else {
                    // Keep this input and output pair.
                    old_results.push(results[i]);
                    new_args.push(arguments[i]);
                    new_arg_types.push(typ);
                }
            }

            // Insert the reduced call with the filtered args, unless there was none left.
            if new_args.is_empty() {
                return;
            }
            let new_call = Instruction::Call { func, arguments: new_args };
            let InsertInstructionResult::Results(_, new_results) =
                context.insert_instruction(new_call, Some(new_arg_types))
            else {
                unreachable!("black_box should not be simplified");
            };
            assert_eq!(old_results.len(), new_results.len(), "black_box remaining results");

            // Redirect the old results to equal the new ones, which still go through the black box.
            let new_results = Vec::from(new_results);
            for (old_result, new_result) in old_results.into_iter().zip(new_results) {
                context.replace_value(old_result, new_result);
            }
        });
    }
}

/// Whether the a parameter with a certain type should be ignored by the `black_box` function.
fn black_box_should_ignore(typ: &Type) -> bool {
    typ.contains_function()
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn removes_parameters_to_ignore() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v4, v5, v6 = call black_box(Field 10, f1, f2) -> (Field, function, function)
            v7 = call v5(v4) -> Field
            return v7
        }
        acir(inline) fn lambda f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        brillig(inline) fn lambda f2 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.black_box_bypass();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = call black_box(Field 10) -> Field
            v4 = call f1(v2) -> Field
            return v4
        }
        acir(inline) fn lambda f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        brillig(inline) fn lambda f2 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ");
    }

    #[test]
    fn removes_instruction_if_all_ignored() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v1, v2 = call black_box(f1, f2) -> (function, function)
            v3 = call v1(Field 10) -> Field
            return v3
        }
        acir(inline) fn lambda f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        brillig(inline) fn lambda f2 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.black_box_bypass();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = call f1(Field 10) -> Field
            return v2
        }
        acir(inline) fn lambda f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        brillig(inline) fn lambda f2 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ");
    }

    #[test]
    fn removes_arrays_params_with_types_to_ignore() {
        let src = "
        acir(inline) fn main f0 {
        b0():
            v3 = make_array [Field 1, f1, f2] : [(Field, function, function); 1]
            v5 = call black_box(v3) -> [(Field, function, function); 1]
            v7 = array_get v5, index u32 0 -> Field
            v9 = array_get v5, index u32 1 -> function
            v11 = array_get v5, index u32 2 -> function
            v12 = call v9(v7) -> Field
            return v12
        }
        acir(inline) fn lambda f1 {
        b0(v0: Field):
            v2 = add v0, Field 10
            return v2
        }
        brillig(inline) fn lambda f2 {
        b0(v0: Field):
            v2 = add v0, Field 10
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.black_box_bypass();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 1, f1, f2] : [(Field, function, function); 1]
            v4 = call f1(Field 1) -> Field
            return v4
        }
        acir(inline) fn lambda f1 {
          b0(v0: Field):
            v2 = add v0, Field 10
            return v2
        }
        brillig(inline) fn lambda f2 {
          b0(v0: Field):
            v2 = add v0, Field 10
            return v2
        }
        ");
    }
}
