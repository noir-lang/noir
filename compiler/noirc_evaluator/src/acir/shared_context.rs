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

    /// Final vector of Brillig functions which will be part of the final program
    /// This is shared across `Context` structs as we want one vector of Brillig
    /// functions across all ACIR artifacts
    generated_brillig: Vec<GeneratedBrillig<F>>,

    /// Maps SSA function index -> Final generated Brillig artifact index.
    /// There can be Brillig functions specified in SSA which do not act as
    /// entry points in ACIR (e.g. only called by other Brillig functions)
    /// This mapping is necessary to use the correct function pointer for a Brillig call.
    /// This uses the brillig parameters in the map since using vectors with different lengths
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
        assert_eq!(
            self.brillig_stdlib_calls_to_resolve.len(),
            0,
            "expected zero remaining 'brillig_stdlib_calls_to_resolve'"
        );
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
        let previous_pointer =
            self.brillig_generated_func_pointers.insert((func_id, arguments), generated_pointer);
        assert!(
            previous_pointer.is_none(),
            "Attempting to override Brillig pointer for function {func_id} which already exists"
        );
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
            // Now we can insert a newly generated Brillig stdlib function
            let code = self.brillig_stdlib.get_code(*brillig_stdlib_func).clone();
            let generated_pointer = self.new_generated_pointer();
            self.brillig_stdlib_func_pointer.insert(*brillig_stdlib_func, generated_pointer);
            self.add_call_to_resolve(func_id, (opcode_location, generated_pointer));
            self.generated_brillig.push(code);
        }
    }

    /// Track a new stdlib call site that must later be patched with its function pointer.
    fn add_call_to_resolve(
        &mut self,
        func_id: FunctionId,
        call_to_resolve: (OpcodeLocation, BrilligFunctionId),
    ) {
        self.brillig_stdlib_calls_to_resolve.entry(func_id).or_default().push(call_to_resolve);
    }

    /// Get and remove the vector of unresolved stdlib call sites for a given function
    pub(super) fn remove_call_to_resolve(
        &mut self,
        func_id: FunctionId,
    ) -> Option<Vec<(OpcodeLocation, BrilligFunctionId)>> {
        self.brillig_stdlib_calls_to_resolve.remove(&func_id)
    }

    /// Remove and return the set of globals used by the given function,
    /// or an empty set if the function had no globals recorded.
    ///
    /// We remove as an entry point should only go through ACIR generation a single time.
    pub(super) fn get_and_remove_used_globals_set(
        &mut self,
        func_id: FunctionId,
    ) -> HashSet<ValueId> {
        self.used_globals.remove(&func_id).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acir::acir_context::{BrilligStdLib, BrilligStdlibFunc};
    use crate::brillig::brillig_ir::artifact::{BrilligParameter, GeneratedBrillig};
    use crate::ssa::ir::map::Id;
    use acvm::FieldElement;
    use acvm::acir::brillig::Opcode;
    use acvm::acir::circuit::OpcodeLocation;

    #[test]
    #[should_panic(
        expected = "Attempting to override Brillig pointer for function f0 which already exists"
    )]
    fn override_brillig_function_pointer() {
        let mut context =
            SharedContext::<FieldElement>::new(BrilligStdLib::default(), Default::default());
        let func_id = Id::test_new(0);
        let args = vec![];

        let ptr1 = context.new_generated_pointer();
        context.insert_generated_brillig(
            func_id,
            args.clone(),
            ptr1,
            GeneratedBrillig { byte_code: vec![], ..Default::default() },
        );

        let ptr2 = context.new_generated_pointer();
        context.insert_generated_brillig(
            func_id,
            args.clone(),
            ptr2,
            GeneratedBrillig { byte_code: vec![], ..Default::default() },
        );
    }

    /// Test that generating the same Brillig function twice reuses the pointer and stores the artifact.
    #[test]
    fn reuse_or_insert_generated_brillig() {
        let mut context =
            SharedContext::<FieldElement>::new(BrilligStdLib::default(), Default::default());
        let f0 = Id::test_new(0);
        let args = vec![BrilligParameter::SingleAddr(0)];

        // Simulate first generation
        let code1 = GeneratedBrillig {
            // This byte code is gibberish, we just want it to be distinct from the
            // next Brillig byte code we insert for testing purposes.
            byte_code: vec![Opcode::Call { location: 5 }],
            ..Default::default()
        };
        let ptr1 = context.new_generated_pointer();
        context.insert_generated_brillig(f0, args.clone(), ptr1, code1.clone());

        // Simulate another call to the same function. We would expect the caller of the shared context to check
        // whether a pointer already has been generated for a given (id, arguments) pair.
        // Here we simply do a sanity check here that the pointer gives us the code we expect.
        let generated_pointer = context.generated_brillig_pointer(f0, args.clone()).unwrap();
        let reused_code = context.generated_brillig(generated_pointer.as_usize());

        assert_eq!(*generated_pointer, ptr1);
        assert_eq!(reused_code.byte_code, code1.byte_code);

        // Explicitly insert a second Brillig artifact with a different key
        let code2 = GeneratedBrillig { byte_code: vec![Opcode::Return], ..Default::default() };
        let f1 = Id::test_new(1);
        let ptr2 = context.new_generated_pointer();
        context.insert_generated_brillig(f1, args.clone(), ptr2, code2.clone());

        // Check the pointers of both Brillig functions
        let f0_pointer = context.generated_brillig_pointer(f0, args.clone()).unwrap();
        assert_eq!(*f0_pointer, ptr1);
        let f1_pointer = context.generated_brillig_pointer(f1, args).unwrap();
        assert_eq!(*f1_pointer, ptr2);

        assert_eq!(context.generated_brillig.len(), 2);
        assert_eq!(
            context.generated_brillig[ptr1.as_usize()].byte_code,
            vec![Opcode::Call { location: 5 }]
        );
        assert_eq!(context.generated_brillig[ptr2.as_usize()].byte_code, vec![Opcode::Return]);
    }

    /// Test that Brillig stdlib calls are resolved correctly and not duplicated.
    #[test]
    fn brillig_stdlib_all_variants_resolved_once() {
        let mut context =
            SharedContext::<FieldElement>::new(BrilligStdLib::default(), Default::default());
        let func_id = Id::test_new(0);
        let opcode_location = OpcodeLocation::Acir(0);

        let variants =
            [BrilligStdlibFunc::Inverse, BrilligStdlibFunc::Quotient, BrilligStdlibFunc::ToLeBytes];

        for &func in &variants {
            // Generate twice for each to check deduplication
            context.generate_brillig_calls_to_resolve(&func, func_id, opcode_location);
            context.generate_brillig_calls_to_resolve(&func, func_id, opcode_location);
        }

        // There should be exactly 3 final GeneratedBrillig entries
        assert_eq!(context.generated_brillig.len(), variants.len());

        // Each variant should have a valid function pointer
        for &func in &variants {
            assert!(context.brillig_stdlib_func_pointer.contains_key(&func));
        }

        // Calls to resolve should be 2 per variant
        let calls = context.remove_call_to_resolve(func_id).unwrap();
        assert_eq!(calls.len(), variants.len() * 2);

        // Check that each call matches the expected stdlib function pointer
        for (i, (_, func_pointer)) in calls.iter().enumerate() {
            let variant_index = i / 2; // 2 calls per variant
            let expected_func = variants[variant_index];
            assert_eq!(context.brillig_stdlib_func_pointer[&expected_func], *func_pointer);
        }
    }

    /// Test that fetching a generated Brillig function with an invalid index panics.
    #[test]
    #[should_panic(expected = "index out of bounds: the len is 0 but the index is 0")]
    fn panic_on_invalid_generated_brillig_index() {
        let mut context =
            SharedContext::<FieldElement>::new(BrilligStdLib::default(), Default::default());
        let func_id = Id::test_new(0);
        let args = vec![];
        // Manually insert a pointer without inserting the corresponding Brillig bytecode
        context
            .brillig_generated_func_pointers
            .insert((func_id, args.clone()), BrilligFunctionId(0));
        // This should panic because the vector of Brillig artifacts is empty
        let pointer = context.generated_brillig_pointer(func_id, args).unwrap();
        let _ = &context.generated_brillig(pointer.as_usize());
    }
}
