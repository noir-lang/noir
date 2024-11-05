use crate::ssa::Ssa;

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn inline_const_brillig_calls(self) -> Self {
        self
    }
}
