use acir::circuit::Circuit;

/// Currently a "noop" transformer.
pub(crate) struct R1CSTransformer {
    acir: Circuit,
}

impl R1CSTransformer {
    pub(crate) fn new(acir: Circuit) -> Self {
        Self { acir }
    }
    // TODO: We could possibly make sure that all polynomials are at most degree-2
    pub(crate) fn transform(self) -> Circuit {
        self.acir
    }
}
