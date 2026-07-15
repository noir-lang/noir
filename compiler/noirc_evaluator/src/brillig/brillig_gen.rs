//! The code generation logic for converting [`crate::ssa`] objects into their respective [Brillig] artifacts.
pub(crate) mod brillig_block;
pub(crate) mod brillig_block_variables;
mod brillig_call;
pub(crate) mod brillig_fn;
pub(crate) mod brillig_globals;
mod brillig_instructions;
mod coalescing;
pub(crate) mod constant_allocation;
mod live_intervals;
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
/// - arguments: Brillig-compatible [`BrilligParameter`] inputs to the function
/// - brillig: The [context structure][Brillig] of all known Brillig artifacts for dependency resolution.
/// - options: Brillig compilation options (e.g., debug trace settings).
///
/// # Returns
/// - Ok([`GeneratedBrillig`]): Fully linked artifact for the entry point that can be executed as a Brillig program.
/// - Err([`RuntimeError`]): If the return value exceeds the witness limit or linking fails
///
/// # Panics
/// - If the global memory size for the function has not been precomputed.
pub(crate) fn gen_brillig_for(
    func: &Function,
    arguments: &[BrilligParameter],
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

    let options = BrilligOptions {
        enable_debug_trace: false,
        copy_site_registry: options.copy_site_registry.clone(),
        ..*options
    };

    let (mut entry_point, stack_start) = BrilligContext::new_entry_point_artifact(
        arguments,
        &return_parameters,
        func.id(),
        globals_memory_size > 0,
        globals_memory_size,
        func.name(),
        &options,
    );

    // Link the entry point with all dependencies
    while let Some(unresolved_fn_label) = entry_point.first_unresolved_function_call() {
        let artifact = &brillig.find_by_label(unresolved_fn_label.clone(), &options, stack_start);
        let Some(artifact) = artifact else {
            return Err(InternalError::General {
                message: format!("Cannot find linked fn {unresolved_fn_label}"),
                call_stack: CallStack::empty(),
            }
            .into());
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
        let entry = gen_brillig_for(ssa.main(), &args, &brillig, &options).unwrap();

        // A simple function returning the addition of its inputs (line 25).
        // The rest is entry point overhead for handling inputs, outputs, and stack checks.
        //
        // Lines 0-2: Reserved register setup (usize_one, free_memory_pointer, stack_pointer).
        // Lines 3-5: Calldata copy.
        // Lines 6-17: Range checks and casts for the two u32 inputs.
        // Lines 18-19: Moves the inputs into sp[2] and sp[3].
        // Line 20: call 25 — jumps to the function body.
        // Line 21: Moves the return value out of the function's register.
        // Lines 22-24: Return data handling — places the result in memory and stops execution.
        // Line 25: The actual add instruction.
        // Lines 26-28: Overflow check on the addition; traps via lines 31-33 if it overflowed.
        // Line 29: Moves the result into the return register.
        // Line 30: Returns from the function body back to line 21.
        assert_artifact_snapshot!(entry, @r"
        fn main
         0: @2 = const u32 1
         1: @1 = const u32 32838
         2: @0 = const u32 70
         3: sp[4] = const u32 2
         4: sp[5] = const u32 0
         5: @67 = calldata copy [sp[5]; sp[4]]
         6: @3 = const field 4294967296
         7: @4 = field lt @67, @3
         8: jump if @4 to 11
         9: @3 = const u32 0
        10: trap @[@1; @3]
        11: @67 = cast @67 to u32
        12: @3 = const field 4294967296
        13: @4 = field lt @68, @3
        14: jump if @4 to 17
        15: @3 = const u32 0
        16: trap @[@1; @3]
        17: @68 = cast @68 to u32
        18: sp[2] = @67
        19: sp[3] = @68
        20: call 25
        21: @69 = sp[2]
        22: sp[3] = const u32 69
        23: sp[4] = const u32 1
        24: stop @[sp[3]; sp[4]]
        25: sp[4] = u32 add sp[2], sp[3]
        26: sp[5] = u32 lt_eq sp[2], sp[4]
        27: jump if sp[5] to 29
        28: call 31
        29: sp[2] = sp[4]
        30: return
        31: @1 = indirect const u64 14990209321349310352
        32: trap @[@1; @2]
        33: return
        ");
    }

    /// Snapshot of full entry point compiled with a small frame that forces spilling.
    /// Verifies the uniform `start_offset=2` (sp[0] = prev stack pointer, sp[1] = spill base):
    /// - Parameters placed at sp[2],sp[3]
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
        let entry = gen_brillig_for(ssa.main(), &args, &brillig, &options).unwrap();

        // Snapshot verifies:
        // - Parameters at sp[2],sp[3] (start_offset=2, uniform across all functions)
        // - Spill base allocation at sp[1]
        // - Correct calldata copy to sp[2],sp[3] (not sp[1],sp[2])
        assert_artifact_snapshot!(entry, @r"
        fn main
         0: @2 = const u32 1
         1: @1 = const u32 262
         2: @0 = const u32 70
         3: sp[4] = const u32 2
         4: sp[5] = const u32 0
         5: @67 = calldata copy [sp[5]; sp[4]]
         6: @3 = const field 4294967296
         7: @4 = field lt @67, @3
         8: jump if @4 to 11
         9: @3 = const u32 0
        10: trap @[@1; @3]
        11: @67 = cast @67 to u32
        12: @3 = const field 4294967296
        13: @4 = field lt @68, @3
        14: jump if @4 to 17
        15: @3 = const u32 0
        16: trap @[@1; @3]
        17: @68 = cast @68 to u32
        18: sp[2] = @67
        19: sp[3] = @68
        20: call 25
        21: @69 = sp[2]
        22: sp[3] = const u32 69
        23: sp[4] = const u32 1
        24: stop @[sp[3]; sp[4]]
        25: sp[1] = @1
        26: @3 = const u32 3
        27: @1 = u32 add @1, @3
        28: sp[4] = u32 add sp[2], sp[3]
        29: sp[5] = const u32 2
        30: sp[6] = u32 add sp[2], sp[5]
        31: sp[5] = const u32 3
        32: sp[7] = u32 add sp[2], sp[5]
        33: sp[5] = const u32 4
        34: sp[8] = u32 add sp[3], sp[5]
        35: sp[5] = const u32 5
        36: sp[9] = u32 add sp[3], sp[5]
        37: sp[5] = const u32 6
        38: sp[10] = u32 add sp[2], sp[5]
        39: sp[5] = const u32 7
        40: sp[11] = u32 add sp[3], sp[5]
        41: sp[5] = const u32 8
        42: store sp[4] at sp[1]
        43: sp[4] = u32 add sp[2], sp[5]
        44: sp[5] = const u32 9
        45: @4 = const u32 1
        46: @3 = u32 add sp[1], @4
        47: store sp[6] at @3
        48: sp[6] = u32 add sp[3], sp[5]
        49: sp[3] = const u32 10
        50: sp[5] = u32 add sp[2], sp[3]
        51: sp[3] = load sp[1]
        52: @4 = const u32 2
        53: @3 = u32 add sp[1], @4
        54: store sp[7] at @3
        55: @4 = const u32 1
        56: @3 = u32 add sp[1], @4
        57: sp[7] = load @3
        58: sp[2] = u32 add sp[3], sp[7]
        59: @4 = const u32 2
        60: @3 = u32 add sp[1], @4
        61: sp[7] = load @3
        62: sp[3] = u32 add sp[2], sp[7]
        63: sp[2] = u32 add sp[3], sp[8]
        64: sp[3] = u32 add sp[2], sp[9]
        65: sp[2] = u32 add sp[3], sp[10]
        66: sp[3] = u32 add sp[2], sp[11]
        67: sp[2] = u32 add sp[3], sp[4]
        68: sp[3] = u32 add sp[2], sp[6]
        69: sp[2] = u32 add sp[3], sp[5]
        70: return
        ");
    }
}
