//! The code generation logic for converting [crate::ssa] objects into their respective [Brillig] artifacts.
pub(crate) mod brillig_block;
pub(crate) mod brillig_block_variables;
mod brillig_call;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_globals;
mod brillig_instructions;
pub(crate) mod constant_allocation;
#[cfg(test)]
mod tests;
mod variable_liveness;

use acvm::FieldElement;
use noirc_errors::call_stack::CallStack;

use self::brillig_fn::FunctionContext;
use super::{
    Brillig, BrilligOptions, BrilligVariable, ValueId,
    brillig_ir::{
        BrilligContext,
        artifact::{BrilligParameter, GeneratedBrillig},
    },
};
use crate::{errors::InternalError, ssa::ir::function::Function};

/// Generates a complete Brillig entry point artifact for a given SSA-level [Function], linking all dependencies.
///
/// This function is responsible for generating a final Brillig artifact corresponding to a compiled SSA [Function].
/// It sets up the entry point context, registers input/output parameters, and recursively resolves and links
/// all transitive Brillig function dependencies.
///
/// # Parameters
/// - func: The SSA [Function] to compile as the entry point.
/// - arguments: Brillig-compatible [BrilligParameter] inputs to the function
/// - brillig: The [context structure][Brillig] of all known Brillig artifacts for dependency resolution.
/// - options: Brillig compilation options (e.g., debug trace settings).
///
/// # Returns
/// - Ok([GeneratedBrillig]): Fully linked artifact for the entry point that can be executed as a Brillig program.
/// - Err([InternalError]): If linking fails to find a dependency
///
/// # Panics
/// - If the global memory size for the function has not been precomputed.
pub(crate) fn gen_brillig_for(
    func: &Function,
    arguments: Vec<BrilligParameter>,
    brillig: &Brillig,
    options: &BrilligOptions,
) -> Result<GeneratedBrillig<FieldElement>, InternalError> {
    // Create the entry point artifact
    let globals_memory_size = brillig
        .globals_memory_size
        .get(&func.id())
        .copied()
        .expect("Should have the globals memory size specified for an entry point");

    let options = BrilligOptions { enable_debug_trace: false, ..*options };

    let (mut entry_point, stack_start) = BrilligContext::new_entry_point_artifact(
        arguments,
        FunctionContext::return_values(func),
        func.id(),
        true,
        globals_memory_size,
        func.name(),
        &options,
    );

    // Link the entry point with all dependencies
    while let Some(unresolved_fn_label) = entry_point.first_unresolved_function_call() {
        let artifact = &brillig.find_by_label(unresolved_fn_label.clone(), &options, stack_start);
        let artifact = match artifact {
            Some(artifact) => artifact,
            None => {
                return Err(InternalError::General {
                    message: format!("Cannot find linked fn {unresolved_fn_label}"),
                    call_stack: CallStack::new(),
                });
            }
        };
        entry_point.link_with(artifact);
        // Insert the range of opcode locations occupied by a procedure
        if let Some(procedure_id) = &artifact.procedure {
            let num_opcodes = entry_point.byte_code.len();
            let previous_num_opcodes = entry_point.byte_code.len() - artifact.byte_code.len();
            // We subtract one as to keep the range inclusive on both ends
            entry_point
                .procedure_locations
                .insert(procedure_id.clone(), (previous_num_opcodes, num_opcodes - 1));
        }
    }
    // Generate the final bytecode
    Ok(entry_point.finish())
}

#[cfg(test)]
mod entry_point {
    use crate::{
        assert_artifact_snapshot,
        brillig::{
            BrilligOptions, brillig_gen::gen_brillig_for, brillig_ir::artifact::BrilligParameter,
        },
        ssa::ssa_gen::Ssa,
    };

    #[test]
    fn entry_point_setup_basic() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = add v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let options = BrilligOptions::default();
        let brillig = ssa.to_brillig(&options);

        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];
        let entry = gen_brillig_for(ssa.main(), args, &brillig, &options).unwrap();

        // foo is a very simple function returning the addition of its inputs, which is done at line 18.
        // The rest of the code is some overhead added to every brillig function for handling inputs, outputs, globals, ...
        //
        // Here is a breakdown of the code:
        // Line 1-9: The function starts with the handling of the inputs via the 'calldata copy'
        // The inputs are put in registers 1 and 2
        // Line 10: Calling the part that deals with brillig globals. In this simple example, there are no globals so
        // it just returns immediately.
        // Line 11: Calling the check_max_stack_depth_procedure (Lines 24-29), via the call 24. Returning from
        // this procedure is going to line 18 (right after the call 24 on line 17)
        // Line 18: the actual body of the function, which is a single add instruction.
        // Line 19-21: An overflow check on the addition, which will trap if the addition overflowed.
        // Line 22: Moving the result of the addition into the return register (register 1)
        // Line 23: Returning from the function body, which goes back to line 12
        // Line 12-15: Handling the return data, moving the return value into the right place in memory and stopping execution.
        assert_artifact_snapshot!(entry, @r"
        fn main
         0: @2 = const u32 1
         1: @1 = const u32 32839
         2: @0 = const u32 71
         3: sp[3] = const u32 2
         4: sp[4] = const u32 0
         5: @68 = calldata copy [sp[4]; sp[3]]
         6: @68 = cast @68 to u32
         7: @69 = cast @69 to u32
         8: sp[1] = @68
         9: sp[2] = @69
        10: call 16
        11: call 17
        12: @70 = sp[1]
        13: sp[2] = const u32 70
        14: sp[3] = const u32 1
        15: stop &[sp[2]; sp[3]]
        16: return
        17: call 24
        18: sp[3] = u32 add sp[1], sp[2]
        19: sp[4] = u32 lt_eq sp[1], sp[3]
        20: jump if sp[4] to 22
        21: call 30
        22: sp[1] = sp[3]
        23: return
        24: @4 = const u32 30791
        25: @3 = u32 lt @0, @4
        26: jump if @3 to 29
        27: @1 = indirect const u64 15764276373176857197
        28: trap &[@1; @2]
        29: return
        30: @1 = indirect const u64 14990209321349310352
        31: trap &[@1; @2]
        32: return
        ");
    }
}
