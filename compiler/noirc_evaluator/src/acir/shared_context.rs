//! The `shared_context` module contains any shared state needed while lowering SSA → ACIR.
//! Specifically, it manages Brillig function pointers, stdlib calls, and global usage.
//!
//! ## Design
//! - Keep a single [SharedContext] struct across all ACIR artifacts so Brillig
//!   functions are deduplicated and pointers remain consistent.
//! - Track both generated Brillig artifacts and stdlib Brillig functions
//!   (the latter do not originate from SSA function IDs).
//! - The Brillig stdlib will have its IDs resolved along with user-defined functions.
//!   However, it is up to the caller of this module to appropriately rewrite the stdlib
//!   call sites in the generated ACIR.
//!
//! ## Preconditions
//! - Caller must provide a valid [BrilligStdLib] and a global usage map.
//!
//! ## Post-conditions
//! - Brillig artifacts are deduplicated, callable by their [function IDs][BrilligFunctionId].
//!   All stdlib function IDs have also been resolved.
//! - Each Brillig function has a unique ID and the IDs resolved by this pass should never be
//!   greater than the number of Brillig functions + stdlib functions.
use std::collections::BTreeMap;

use acvm::{
    AcirField,
    acir::circuit::{OpcodeLocation, brillig::BrilligFunctionId},
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::ir::value::ValueId;
use crate::{
    acir::acir_context::{BrilligStdLib, BrilligStdlibFunc},
    brillig::brillig_ir::artifact::{BrilligParameter, GeneratedBrillig},
    ssa::ir::function::FunctionId,
};

/// Holds state shared across all ACIR codegen contexts.
/// Tracks user-defined Brillig functions, stdlib Brillig functions,
/// unresolved stdlib call sites, and global usage information.
#[derive(Default)]
pub(super) struct SharedContext<F: AcirField> {
    brillig_stdlib: BrilligStdLib<F>,

    /// Final list of Brillig functions which will be part of the final program
    /// This is shared across `Context` structs as we want one list of Brillig
    /// functions across all ACIR artifacts
    generated_brillig: Vec<GeneratedBrillig<F>>,

    /// Maps SSA function index -> Final generated Brillig artifact index.
    /// There can be Brillig functions specified in SSA which do not act as
    /// entry points in ACIR (e.g. only called by other Brillig functions)
    /// This mapping is necessary to use the correct function pointer for a Brillig call.
    /// This uses the brillig parameters in the map since using slices with different lengths
    /// needs to create different brillig entrypoints
    brillig_generated_func_pointers:
        BTreeMap<(FunctionId, Vec<BrilligParameter>), BrilligFunctionId>,

    /// Maps a Brillig std lib function (a handwritten primitive such as for inversion) -> Final generated Brillig artifact index.
    /// A separate mapping from normal Brillig calls is necessary as these methods do not have an associated function id from SSA.
    brillig_stdlib_func_pointer: HashMap<BrilligStdlibFunc, BrilligFunctionId>,

    /// Keeps track of Brillig std lib calls per function that need to still be resolved
    /// with the correct function pointer from the `brillig_stdlib_func_pointer` map.
    brillig_stdlib_calls_to_resolve: HashMap<FunctionId, Vec<(OpcodeLocation, BrilligFunctionId)>>,

    /// `used_globals` needs to be built from a function call graph.
    ///
    /// Maps an ACIR function to the globals used in that function.
    /// This includes all globals used in functions called internally.
    used_globals: HashMap<FunctionId, HashSet<ValueId>>,
}

impl<F: AcirField> SharedContext<F> {
    /// Create a new [SharedContext] with a stdlib and a precomputed global usage map.
    pub(super) fn new(
        brillig_stdlib: BrilligStdLib<F>,
        used_globals: HashMap<FunctionId, HashSet<ValueId>>,
    ) -> Self {
        Self { brillig_stdlib, used_globals, ..Default::default() }
    }

    /// Lookup a previously generated Brillig function pointer by ([FunctionId], Vec<[BrilligParameter]>).
    /// Returns `None` if a pointer has not yet been registered.
    pub(super) fn generated_brillig_pointer(
        &self,
        func_id: FunctionId,
        arguments: Vec<BrilligParameter>,
    ) -> Option<&BrilligFunctionId> {
        self.brillig_generated_func_pointers.get(&(func_id, arguments))
    }

    /// Get the generated Brillig function artifact by raw pointer index.
    ///
    /// # Panics
    /// If the pointer index is out of bounds.
    pub(super) fn generated_brillig(&self, func_pointer: usize) -> &GeneratedBrillig<F> {
        &self.generated_brillig[func_pointer]
    }

    /// Finalize this context, consuming it and returning all generated Brillig functions.
    pub(super) fn finish(self) -> Vec<GeneratedBrillig<F>> {
        self.generated_brillig
    }

    /// Insert a newly generated Brillig function into the context.
    pub(super) fn insert_generated_brillig(
        &mut self,
        func_id: FunctionId,
        arguments: Vec<BrilligParameter>,
        generated_pointer: BrilligFunctionId,
        code: GeneratedBrillig<F>,
    ) {
        self.brillig_generated_func_pointers.insert((func_id, arguments), generated_pointer);
        self.generated_brillig.push(code);
    }

    /// Allocate a fresh Brillig function pointer
    pub(super) fn new_generated_pointer(&self) -> BrilligFunctionId {
        BrilligFunctionId(self.generated_brillig.len() as u32)
    }

    /// Register a stdlib Brillig call for later resolution.
    /// If the stdlib function has already been emitted, reuse its pointer.
    /// Otherwise, generate and insert it into the context.
    pub(super) fn generate_brillig_calls_to_resolve(
        &mut self,
        brillig_stdlib_func: &BrilligStdlibFunc,
        func_id: FunctionId,
        opcode_location: OpcodeLocation,
    ) {
        if let Some(generated_pointer) =
            self.brillig_stdlib_func_pointer.get(brillig_stdlib_func).copied()
        {
            self.add_call_to_resolve(func_id, (opcode_location, generated_pointer));
        } else {
            let code = self.brillig_stdlib.get_code(*brillig_stdlib_func);
            let generated_pointer = self.new_generated_pointer();
            self.insert_generated_brillig_stdlib(
                *brillig_stdlib_func,
                generated_pointer,
                func_id,
                opcode_location,
                code.clone(),
            );
        }
    }

    /// Insert a newly generated Brillig stdlib function
    fn insert_generated_brillig_stdlib(
        &mut self,
        brillig_stdlib_func: BrilligStdlibFunc,
        generated_pointer: BrilligFunctionId,
        func_id: FunctionId,
        opcode_location: OpcodeLocation,
        code: GeneratedBrillig<F>,
    ) {
        self.brillig_stdlib_func_pointer.insert(brillig_stdlib_func, generated_pointer);
        self.add_call_to_resolve(func_id, (opcode_location, generated_pointer));
        self.generated_brillig.push(code);
    }

    /// Track a new stdlib call site that must later be patched with its function pointer.
    fn add_call_to_resolve(
        &mut self,
        func_id: FunctionId,
        call_to_resolve: (OpcodeLocation, BrilligFunctionId),
    ) {
        self.brillig_stdlib_calls_to_resolve.entry(func_id).or_default().push(call_to_resolve);
    }

    /// Get the list of unresolved stdlib call sites for a given function
    pub(super) fn get_call_to_resolve(
        &self,
        func_id: FunctionId,
    ) -> Option<&Vec<(OpcodeLocation, BrilligFunctionId)>> {
        self.brillig_stdlib_calls_to_resolve.get(&func_id)
    }

    /// Remove and return the set of globals used by the given function,
    /// or an empty set if the function had no globals recorded.
    ///
    /// We remove as an entry point should only go through ACIR generation a single time.
    pub(super) fn get_used_globals_set(&mut self, func_id: FunctionId) -> HashSet<ValueId> {
        self.used_globals.remove(&func_id).unwrap_or_default()
    }
}
