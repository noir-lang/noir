use noirc_artifacts::debug::ProcedureDebugId;
use serde::{Deserialize, Serialize};

mod array_copy;
mod array_reverse;
mod check_max_stack_depth;
mod error_with_string;
mod mem_copy;
mod prepare_vector_insert;
mod prepare_vector_push;
mod vector_copy;
mod vector_pop_back;
mod vector_pop_front;
mod vector_remove;

use array_copy::compile_array_copy_procedure;
use array_reverse::compile_array_reverse_procedure;
use check_max_stack_depth::compile_check_max_stack_depth_procedure;
use error_with_string::compile_error_with_string_procedure;
use mem_copy::compile_mem_copy_procedure;
use prepare_vector_insert::compile_prepare_vector_insert_procedure;
use prepare_vector_push::compile_prepare_vector_push_procedure;
use vector_copy::compile_vector_copy_procedure;
use vector_pop_back::compile_vector_pop_back_procedure;
use vector_pop_front::compile_vector_pop_front_procedure;
use vector_remove::compile_vector_remove_procedure;

use crate::brillig::{BrilligOptions, brillig_ir::AcirField};

use super::{
    BrilligContext,
    artifact::{BrilligArtifact, Label},
    debug_show::DebugToString,
};

/// Procedures are a set of complex operations that are common in the noir language.
/// Extracting them to reusable procedures allows us to reduce the size of the generated Brillig.
/// Procedures receive their arguments on scratch space to avoid stack dumping&restoring.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub enum ProcedureId {
    /// Conditionally copies a source array to a destination array.
    /// If the reference count of the source array is 1, then we can directly copy the pointer of the source array to the destination array.
    ArrayCopy,
    /// Reverses an array in-place.
    /// It is the responsibility of the caller to ensure the reference count of the array is 1.
    ArrayReverse,
    /// Conditionally copies a source vector to a destination vector.
    /// If the reference count of the source vector is 1, then we can directly copy the pointer of the source vector to the destination vector.
    VectorCopy,
    /// Copy a number of items between two heap addresses.
    MemCopy,
    /// Prepares a vector for pushing a new item. It tries to reuse the source vector,
    /// allocating a new vector with a higher capacity and the copy of the source vector items if necessary.
    ///
    /// If the parameter is `true` it pushes to the back, otherwise to the front.
    PrepareVectorPush(bool),
    /// Pops items from the front of a vector, returning the new vector.
    /// Reuses the source vector if the reference count is 1.
    VectorPopFront,
    /// Pops items from the back of a vector, returning the new vector and the pointer to the popped items.
    /// Reuses the source vector if the reference count is 1.
    VectorPopBack,
    /// Prepare a vector for a insert operation, leaving a hole at the index position, returning a pointer where the item can be written.
    PrepareVectorInsert,
    /// Remove items at a given index from a vector, returning the new vector.
    /// Reuses the source vector if the reference count is 1.
    VectorRemove,
    /// Check that the stack memory has not exceeded the maximum size allowed by the layout.
    CheckMaxStackDepth,
    /// Revert with the given error message.
    RevertWithString(String),
}

impl ProcedureId {
    pub(crate) fn to_debug_id(&self) -> ProcedureDebugId {
        ProcedureDebugId(match self {
            ProcedureId::ArrayCopy => 0,
            ProcedureId::ArrayReverse => 1,
            ProcedureId::VectorCopy => 2,
            ProcedureId::MemCopy => 3,
            ProcedureId::PrepareVectorPush(true) => 4,
            ProcedureId::PrepareVectorPush(false) => 5,
            ProcedureId::VectorPopFront => 6,
            ProcedureId::VectorPopBack => 7,
            ProcedureId::PrepareVectorInsert => 8,
            ProcedureId::VectorRemove => 9,
            ProcedureId::CheckMaxStackDepth => 10,
            ProcedureId::RevertWithString(_) => 11,
        })
    }

    pub fn from_debug_id(debug_id: ProcedureDebugId) -> Self {
        let inner = debug_id.0;
        match inner {
            0 => ProcedureId::ArrayCopy,
            1 => ProcedureId::ArrayReverse,
            2 => ProcedureId::VectorCopy,
            3 => ProcedureId::MemCopy,
            4 => ProcedureId::PrepareVectorPush(true),
            5 => ProcedureId::PrepareVectorPush(false),
            6 => ProcedureId::VectorPopFront,
            7 => ProcedureId::VectorPopBack,
            8 => ProcedureId::PrepareVectorInsert,
            9 => ProcedureId::VectorRemove,
            10 => ProcedureId::CheckMaxStackDepth,
            11 => ProcedureId::RevertWithString("".to_string()),
            _ => panic!("Unsupported procedure debug ID of {inner} was supplied"),
        }
    }
}

impl std::fmt::Display for ProcedureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcedureId::ArrayCopy => write!(f, "ArrayCopy"),
            ProcedureId::ArrayReverse => write!(f, "ArrayReverse"),
            ProcedureId::VectorCopy => write!(f, "VectorCopy"),
            ProcedureId::MemCopy => write!(f, "MemCopy"),
            ProcedureId::PrepareVectorPush(flag) => write!(f, "PrepareVectorPush({flag})"),
            ProcedureId::VectorPopFront => write!(f, "VectorPopFront"),
            ProcedureId::VectorPopBack => write!(f, "VectorPopBack"),
            ProcedureId::PrepareVectorInsert => write!(f, "PrepareVectorInsert"),
            ProcedureId::VectorRemove => write!(f, "VectorRemove"),
            ProcedureId::CheckMaxStackDepth => write!(f, "CheckMaxStackDepth"),
            ProcedureId::RevertWithString(_) => write!(f, "ErrorWithString"),
        }
    }
}

/// Compile a procedure as a stand-alone Brillig artifact, generating byte code for a specific operation,
/// reading and returning arguments through the [`ScratchSpace`][crate::brillig::brillig_ir::registers::ScratchSpace].
pub(crate) fn compile_procedure<F: AcirField + DebugToString>(
    procedure_id: ProcedureId,
    options: &BrilligOptions,
    stack_start: usize,
) -> BrilligArtifact<F> {
    let mut brillig_context = BrilligContext::new_for_procedure(procedure_id.clone(), options);
    brillig_context.enter_context(Label::procedure(procedure_id.clone()));

    match procedure_id {
        ProcedureId::MemCopy => compile_mem_copy_procedure(&mut brillig_context),
        ProcedureId::ArrayCopy => compile_array_copy_procedure(&mut brillig_context),
        ProcedureId::ArrayReverse => compile_array_reverse_procedure(&mut brillig_context),
        ProcedureId::VectorCopy => compile_vector_copy_procedure(&mut brillig_context),
        ProcedureId::PrepareVectorPush(push_back) => {
            compile_prepare_vector_push_procedure(&mut brillig_context, push_back);
        }
        ProcedureId::VectorPopFront => {
            compile_vector_pop_front_procedure(&mut brillig_context);
        }
        ProcedureId::VectorPopBack => {
            compile_vector_pop_back_procedure(&mut brillig_context);
        }
        ProcedureId::PrepareVectorInsert => {
            compile_prepare_vector_insert_procedure(&mut brillig_context);
        }
        ProcedureId::VectorRemove => compile_vector_remove_procedure(&mut brillig_context),
        ProcedureId::CheckMaxStackDepth => {
            compile_check_max_stack_depth_procedure(&mut brillig_context, stack_start);
        }
        ProcedureId::RevertWithString(error_string) => {
            compile_error_with_string_procedure(&mut brillig_context, error_string);
        }
    }

    brillig_context.return_instruction();
    brillig_context.into_artifact()
}

#[cfg(test)]
mod tests {
    use acvm::{FieldElement, acir::brillig::Opcode as BrilligOpcode};

    use super::{ProcedureId, compile_procedure};
    use crate::brillig::BrilligOptions;

    /// Every procedure the compiler can emit. `RevertWithString` carries an arbitrary payload.
    fn all_procedures() -> Vec<ProcedureId> {
        vec![
            ProcedureId::ArrayCopy,
            ProcedureId::ArrayReverse,
            ProcedureId::VectorCopy,
            ProcedureId::MemCopy,
            ProcedureId::PrepareVectorPush(true),
            ProcedureId::PrepareVectorPush(false),
            ProcedureId::VectorPopFront,
            ProcedureId::VectorPopBack,
            ProcedureId::PrepareVectorInsert,
            ProcedureId::VectorRemove,
            ProcedureId::CheckMaxStackDepth,
            ProcedureId::RevertWithString("x".to_string()),
        ]
    }

    fn compile_procedure_to_string(procedure_id: ProcedureId) -> String {
        let options = BrilligOptions::default();
        let artifact = compile_procedure::<FieldElement>(procedure_id, &options, 0);
        artifact.finish().to_string()
    }

    /// Enforces the "procedures cannot call procedures" safety guarantee: a procedure is compiled
    /// with `can_call_procedures = false`, so every call site must fall back to inline codegen and
    /// no procedure's bytecode may contain a `Call` opcode. This keeps scratch "arenas" from nesting.
    #[test]
    fn procedures_do_not_call_procedures() {
        let options = BrilligOptions::default();
        for procedure_id in all_procedures() {
            let artifact = compile_procedure::<FieldElement>(procedure_id.clone(), &options, 0);
            let has_call =
                artifact.byte_code.iter().any(|op| matches!(op, BrilligOpcode::Call { .. }));
            assert!(
                !has_call,
                "procedure {procedure_id} emitted a Call opcode; procedures must not call procedures"
            );
        }
    }

    /// Highest Direct memory address (`@N`) referenced anywhere in a procedure's bytecode.
    ///
    /// A procedure is compiled in a [`ScratchSpace`][crate::brillig::brillig_ir::registers::ScratchSpace]
    /// context, so it never touches the stack: every operand address is `Direct` and prints as
    /// `@N` (a `Relative` address would print as `sp[N]`). The maximum `N` is therefore the peak
    /// reserved+scratch slot the procedure occupies. Scratch begins right after the reserved
    /// registers, so the number of scratch slots used is `max_direct_address - ReservedRegisters::len() + 1`.
    fn max_direct_address(procedure_id: ProcedureId) -> usize {
        let dump = compile_procedure_to_string(procedure_id);
        assert!(!dump.contains("sp["), "a procedure unexpectedly used a stack-relative address");
        dump.split('@')
            .skip(1)
            .filter_map(|s| {
                let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
                digits.parse::<usize>().ok()
            })
            .max()
            .unwrap_or(0)
    }

    /// Pins the peak scratch-space demand across every procedure.
    ///
    /// Procedures pass arguments/returns *and* allocate all of their local temporaries in scratch
    /// space, so the true peak is much larger than the argument count alone. This measures the
    /// high-water mark directly from the generated bytecode and guards two things:
    /// 1. the concrete peak stays what we expect, and
    /// 2. it stays comfortably below `MAX_SCRATCH_SPACE`, the hard ceiling the allocator enforces.
    ///
    /// If a procedure change moves the peak, update the expected number here.
    #[test]
    fn peak_scratch_demand_across_procedures() {
        use crate::brillig::brillig_ir::{ReservedRegisters, registers::MAX_SCRATCH_SPACE};

        let peak_address = all_procedures().into_iter().map(max_direct_address).max().unwrap();
        let scratch_slots = peak_address - ReservedRegisters::len() + 1;

        // VectorRemove currently drives the peak at Direct address @20.
        assert_eq!(peak_address, 20, "peak Direct address across procedures changed");
        assert_eq!(scratch_slots, 18, "peak scratch-slot demand across procedures changed");

        // The peak must fit under the scratch ceiling the allocator enforces at runtime.
        assert!(
            scratch_slots <= MAX_SCRATCH_SPACE,
            "peak scratch demand {scratch_slots} exceeds MAX_SCRATCH_SPACE {MAX_SCRATCH_SPACE}"
        );
    }

    #[test]
    fn array_copy_procedure() {
        let artifact_string = compile_procedure_to_string(ProcedureId::ArrayCopy);
        insta::assert_snapshot!(artifact_string, @r"
        fn ArrayCopy
         0: @6 = load @3
         1: @7 = u32 eq @6, @2
         2: jump if @7 to 17
         3: @5 = @1
         4: @1 = u32 add @1, @4
         5: @9 = u32 add @3, @4
         6: @10 = @3
         7: @11 = @5
         8: jump to 13
         9: @8 = load @10
        10: store @8 at @11
        11: @10 = u32 add @10, @2
        12: @11 = u32 add @11, @2
        13: @12 = u32 lt @10, @9
        14: jump if @12 to 9
        15: @5 = indirect const u32 1
        16: jump to 18
        17: @5 = @3
        18: return
        ");
    }
}
