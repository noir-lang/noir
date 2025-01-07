pub(crate) mod brillig_gen;
pub(crate) mod brillig_ir;

use acvm::FieldElement;
use brillig_gen::brillig_block_variables::{allocate_value, allocate_value_with_type};
use brillig_ir::{artifact::LabelType, brillig_variable::BrilligVariable, registers::GlobalSpace, BrilligContext};

use self::{
    brillig_gen::convert_ssa_function,
    brillig_ir::{
        artifact::{BrilligArtifact, Label},
        procedures::compile_procedure,
    },
};
use crate::ssa::{
    ir::{function::{Function, FunctionId}, map::DenseMap, types::Type, value::{Value, ValueId}},
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;
use std::{borrow::Cow, collections::BTreeSet};

pub use self::brillig_ir::procedures::ProcedureId;

/// Context structure for the brillig pass.
/// It stores brillig-related data required for brillig generation.
#[derive(Default)]
pub struct Brillig {
    /// Maps SSA function labels to their brillig artifact
    ssa_function_to_brillig: HashMap<FunctionId, BrilligArtifact<FieldElement>>,
    globals: BrilligArtifact<FieldElement>,
}

impl Brillig {
    /// Compiles a function into brillig and store the compilation artifacts
    pub(crate) fn compile(&mut self, func: &Function, enable_debug_trace: bool, globals: &HashMap<ValueId, BrilligVariable>) {
        let obj = convert_ssa_function(func, enable_debug_trace, globals);
        self.ssa_function_to_brillig.insert(func.id(), obj);
    }

    /// Finds a brillig artifact by its label
    pub(crate) fn find_by_label(
        &self,
        function_label: Label,
    ) -> Option<Cow<BrilligArtifact<FieldElement>>> {
        match function_label.label_type {
            LabelType::Function(function_id, _) => {
                self.ssa_function_to_brillig.get(&function_id).map(Cow::Borrowed)
            }
            // Procedures are compiled as needed
            LabelType::Procedure(procedure_id) => Some(Cow::Owned(compile_procedure(procedure_id))),
            LabelType::GlobalInit => Some(Cow::Borrowed(&self.globals)),
            _ => unreachable!("ICE: Expected a function or procedure label"),
        }
    }

    pub(crate) fn create_brillig_globals(brillig_context: &mut BrilligContext<FieldElement, GlobalSpace>, globals: &DenseMap<Value>) -> HashMap<ValueId, BrilligVariable> {
        let mut brillig_globals = HashMap::default();
        for (id, value) in globals.iter() {
            match value {
                Value::NumericConstant { constant, typ } => {                    
                    let new_variable = allocate_value_with_type(brillig_context, Type::Numeric(*typ));
                    dbg!(new_variable.clone());
                    brillig_context.const_instruction(new_variable.extract_single_addr(), *constant);

                    brillig_globals.insert(id, new_variable);
                }
                _ => {
                    panic!("got something other than numeric constant")
                }
            }
        }
        brillig_globals
    }
}

impl std::ops::Index<FunctionId> for Brillig {
    type Output = BrilligArtifact<FieldElement>;
    fn index(&self, id: FunctionId) -> &Self::Output {
        &self.ssa_function_to_brillig[&id]
    }
}

impl Ssa {
    /// Compile Brillig functions and ACIR functions reachable from them
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn to_brillig(&self, enable_debug_trace: bool) -> Brillig {
        // Collect all the function ids that are reachable from brillig
        // That means all the functions marked as brillig and ACIR functions called by them
        let brillig_reachable_function_ids = self
            .functions
            .iter()
            .filter_map(|(id, func)| func.runtime().is_brillig().then_some(*id))
            .collect::<BTreeSet<_>>();

        let mut brillig = Brillig::default();

        let mut brillig_context = BrilligContext::new_for_global_init(enable_debug_trace);
        brillig_context.enter_context(Label::globals_init());
        let brillig_globals = Brillig::create_brillig_globals(&mut brillig_context, &self.global_values);
        brillig_context.return_instruction();

        let artifact = brillig_context.artifact();
        brillig.globals = artifact;

        for brillig_function_id in brillig_reachable_function_ids {
            let func = &self.functions[&brillig_function_id];
            brillig.compile(func, enable_debug_trace, &brillig_globals);
        }

        brillig
    }
}
