//! The code generation logic for converting [crate::ssa] objects into their respective [Brillig] artifacts.
pub(crate) mod brillig_block;
pub(crate) mod brillig_block_variables;
mod brillig_call;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_globals;
mod brillig_instructions;
pub(crate) mod constant_allocation;
pub(crate) mod spill_manager;
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
use crate::{
    errors::{InternalError, RuntimeError},
    ssa::ir::function::Function,
};

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
/// - Err([RuntimeError]): If the return value exceeds the witness limit or linking fails
///
/// # Panics
/// - If the global memory size for the function has not been precomputed.
pub(crate) fn gen_brillig_for(
    func: &Function,
    arguments: Vec<BrilligParameter>,
    brillig: &Brillig,
    options: &BrilligOptions,
) -> Result<GeneratedBrillig<FieldElement>, RuntimeError> {
    let return_parameters = FunctionContext::return_values(func);

    // Check if the return value size exceeds the limit before generating the entry point.
    // This is done early to avoid the expensive entry point codegen which iterates over
    // each element in the return arrays.
    func.dfg.get_num_return_witnesses(func)?;

    // Create the entry point artifact
    let globals_memory_size = brillig
        .globals_memory_size
        .get(&func.id())
        .copied()
        .expect("Should have the globals memory size specified for an entry point");

    let options = BrilligOptions { enable_debug_trace: false, ..*options };

    // The entry point must use the same spill_support as the callee so that
    // parameter offsets (start_offset) align with the function body's register layout.
    let callee_spill_support =
        brillig.ssa_function_to_brillig.get(&func.id()).map_or(false, |a| a.spill_support);

    let (mut entry_point, stack_start) = BrilligContext::new_entry_point_artifact(
        arguments,
        return_parameters,
        func.id(),
        true,
        globals_memory_size,
        func.name(),
        &options,
        callee_spill_support,
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
                }
                .into());
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
            BrilligOptions,
            brillig_gen::gen_brillig_for,
            brillig_ir::{LayoutConfig, artifact::BrilligParameter, registers::MAX_SCRATCH_SPACE},
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
         3: call 16
         4: sp[3] = const u32 2
         5: sp[4] = const u32 0
         6: @68 = calldata copy [sp[4]; sp[3]]
         7: @68 = cast @68 to u32
         8: @69 = cast @69 to u32
         9: sp[1] = @68
        10: sp[2] = @69
        11: call 17
        12: @70 = sp[1]
        13: sp[2] = const u32 70
        14: sp[3] = const u32 1
        15: stop @[sp[2]; sp[3]]
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
        28: trap @[@1; @2]
        29: return
        30: @1 = indirect const u64 14990209321349310352
        31: trap @[@1; @2]
        32: return
        ");
    }

    /// Snapshot of full entry point compiled with a small frame that forces spilling.
    /// Verifies that the entry point uses spill_support=true (matching the callee):
    /// - Parameters placed at sp[2],sp[3] (start_offset=2, slot sp[1] reserved for spill base)
    /// - Calldata copy targets sp[2],sp[3] (not sp[1],sp[2])
    #[test]
    fn entry_point_setup_with_spill_support() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v2 = unchecked_add v0, v1
            v3 = unchecked_add v0, u32 2
            v4 = unchecked_add v0, u32 3
            v5 = unchecked_add v1, u32 4
            v6 = unchecked_add v1, u32 5
            v7 = unchecked_add v0, u32 6
            v8 = unchecked_add v1, u32 7
            v9 = unchecked_add v0, u32 8
            v10 = unchecked_add v1, u32 9
            v11 = unchecked_add v0, u32 10
            v12 = unchecked_add v2, v3
            v13 = unchecked_add v12, v4
            v14 = unchecked_add v13, v5
            v15 = unchecked_add v14, v6
            v16 = unchecked_add v15, v7
            v17 = unchecked_add v16, v8
            v18 = unchecked_add v17, v9
            v19 = unchecked_add v18, v10
            v20 = unchecked_add v19, v11
            return v20
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let layout = LayoutConfig::new(12, 16, MAX_SCRATCH_SPACE);
        let options = BrilligOptions { layout, ..Default::default() };
        let brillig = ssa.to_brillig(&options);

        let args = vec![BrilligParameter::SingleAddr(32), BrilligParameter::SingleAddr(32)];
        let entry = gen_brillig_for(ssa.main(), args, &brillig, &options).unwrap();

        // Snapshot verifies:
        // - Parameters at sp[2],sp[3] (start_offset=2, matching callee's spill_support=true)
        // - Spill base allocation at sp[1]
        // - Correct calldata copy to sp[2],sp[3] (not sp[1],sp[2])
        assert_artifact_snapshot!(entry, @r"
        fn main
         0: @2 = const u32 1
         1: @1 = const u32 263
         2: @0 = const u32 71
         3: call 16
         4: sp[4] = const u32 2
         5: sp[5] = const u32 0
         6: @68 = calldata copy [sp[5]; sp[4]]
         7: @68 = cast @68 to u32
         8: @69 = cast @69 to u32
         9: sp[2] = @68
        10: sp[3] = @69
        11: call 17
        12: @70 = sp[2]
        13: sp[3] = const u32 70
        14: sp[4] = const u32 1
        15: stop @[sp[3]; sp[4]]
        16: return
        17: call 74
        18: sp[1] = @1
        19: @3 = const u32 3
        20: @1 = u32 add @1, @3
        21: sp[4] = u32 add sp[2], sp[3]
        22: sp[5] = const u32 2
        23: sp[6] = u32 add sp[2], sp[5]
        24: sp[5] = const u32 3
        25: sp[7] = u32 add sp[2], sp[5]
        26: sp[5] = const u32 4
        27: sp[8] = u32 add sp[3], sp[5]
        28: sp[5] = const u32 5
        29: sp[9] = u32 add sp[3], sp[5]
        30: sp[5] = const u32 6
        31: sp[10] = u32 add sp[2], sp[5]
        32: sp[5] = const u32 7
        33: sp[11] = u32 add sp[3], sp[5]
        34: sp[5] = const u32 8
        35: @3 = sp[1]
        36: @4 = const u32 0
        37: @3 = u32 add @3, @4
        38: store sp[4] at @3
        39: sp[4] = u32 add sp[2], sp[5]
        40: sp[5] = const u32 9
        41: @3 = sp[1]
        42: @4 = const u32 1
        43: @3 = u32 add @3, @4
        44: store sp[6] at @3
        45: sp[6] = u32 add sp[3], sp[5]
        46: sp[3] = const u32 10
        47: sp[5] = u32 add sp[2], sp[3]
        48: @3 = sp[1]
        49: @4 = const u32 0
        50: @3 = u32 add @3, @4
        51: sp[3] = load @3
        52: @3 = sp[1]
        53: @4 = const u32 2
        54: @3 = u32 add @3, @4
        55: store sp[7] at @3
        56: @3 = sp[1]
        57: @4 = const u32 1
        58: @3 = u32 add @3, @4
        59: sp[7] = load @3
        60: sp[2] = u32 add sp[3], sp[7]
        61: @3 = sp[1]
        62: @4 = const u32 2
        63: @3 = u32 add @3, @4
        64: sp[7] = load @3
        65: sp[3] = u32 add sp[2], sp[7]
        66: sp[2] = u32 add sp[3], sp[8]
        67: sp[3] = u32 add sp[2], sp[9]
        68: sp[2] = u32 add sp[3], sp[10]
        69: sp[3] = u32 add sp[2], sp[11]
        70: sp[2] = u32 add sp[3], sp[4]
        71: sp[3] = u32 add sp[2], sp[6]
        72: sp[2] = u32 add sp[3], sp[5]
        73: return
        74: @4 = const u32 251
        75: @3 = u32 lt @0, @4
        76: jump if @3 to 79
        77: @1 = indirect const u64 15764276373176857197
        78: trap @[@1; @2]
        79: return
        ");
    }
}
