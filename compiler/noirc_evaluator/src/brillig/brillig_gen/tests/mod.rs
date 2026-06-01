//! The tests in the following files are simple tests checking the result of generating brillig bytecode
//! from a simple brillig function in SSA form. The brillig function is doing one elementary operation to
//! show how it is generated on the Brillig bytecode level.
//!
//! Every unit test will be checking against against an individual Brillig artifact
//! that has not yet undergone linking. This means any calls, to both external functions or procedures,
//! and jumps are expected to be unresolved. Thus any call/jump is expected to still have a label of `0`.

use acvm::{
    FieldElement,
    acir::brillig::Opcode as BrilligOpcode,
    brillig_vm::{VM, VMStatus},
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;

use crate::{
    brillig::{
        Brillig, BrilligOptions,
        brillig_gen::{brillig_fn::FunctionContext, gen_brillig_for},
    },
    ssa::ssa_gen::Ssa,
};

mod binary;
mod black_box;
mod call;
mod coalescing;
mod memory;
mod spill;

pub(crate) fn ssa_to_brillig_artifacts(src: &str) -> Brillig {
    ssa_to_brillig_artifacts_with_options(src, &BrilligOptions::default())
}

pub(crate) fn ssa_to_brillig_artifacts_with_options(
    src: &str,
    options: &BrilligOptions,
) -> Brillig {
    let ssa = Ssa::from_str(src).unwrap();
    ssa.to_brillig(options)
}

/// Compile SSA source to a fully linked entry point and execute it with the given calldata.
/// Returns the return data from the VM.
pub(crate) fn execute_brillig_from_ssa(
    src: &str,
    calldata: Vec<FieldElement>,
) -> Vec<FieldElement> {
    execute_brillig_from_ssa_with_options(src, calldata, &BrilligOptions::default())
}

pub(crate) fn execute_brillig_from_ssa_with_options(
    src: &str,
    calldata: Vec<FieldElement>,
    options: &BrilligOptions,
) -> Vec<FieldElement> {
    let ssa = Ssa::from_str(src).unwrap();
    let brillig = ssa.to_brillig(options);
    let func = ssa.main();
    let arguments: Vec<_> = func
        .parameters()
        .iter()
        .map(|&value_id| {
            let typ = func.dfg.type_of_value(value_id);
            FunctionContext::ssa_type_to_parameter(&typ)
        })
        .collect();
    let generated = gen_brillig_for(func, arguments, &brillig, options).unwrap();
    execute_bytecode(&generated.byte_code, calldata)
}

fn execute_bytecode(
    byte_code: &[BrilligOpcode<FieldElement>],
    calldata: Vec<FieldElement>,
) -> Vec<FieldElement> {
    let solver = Bn254BlackBoxSolver;
    let mut vm = VM::new(calldata, byte_code, &solver, false, None);
    let status = vm.process_opcodes();
    match status {
        VMStatus::Finished { return_data_offset, return_data_size } => {
            let memory = vm.take_memory();
            (return_data_offset as usize..return_data_offset as usize + return_data_size as usize)
                .map(|i| {
                    memory.read(acvm::acir::brillig::MemoryAddress::direct(i as u32)).to_field()
                })
                .collect()
        }
        VMStatus::ForeignCallWait { .. } => {
            panic!("Unexpected foreign call")
        }
        VMStatus::Failure { reason, .. } => {
            panic!("Brillig execution failed: {reason:?}")
        }
        VMStatus::InProgress => {
            panic!("VM did not complete")
        }
    }
}

#[macro_export]
macro_rules! assert_artifact_snapshot {
    ($artifact:expr, $($arg:tt)*) => {
        #[allow(unused_mut)]
        let artifact_string = $artifact.to_string();
        insta::assert_snapshot!(artifact_string, $($arg)*)
    };
}
