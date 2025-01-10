use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;

use super::{BrilligVariable, ValueId};
use crate::brillig::{brillig_ir::BrilligContext, GlobalSpace};

pub(crate) struct BrilligGlobals<'global> {
    pub(crate) brillig_context: &'global mut BrilligContext<FieldElement, GlobalSpace>,

    brillig_globals: HashMap<ValueId, BrilligVariable>,
}
