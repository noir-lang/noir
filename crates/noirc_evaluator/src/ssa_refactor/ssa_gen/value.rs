use crate::ssa_refactor::ir::function::FunctionId as IrFunctionId;
use crate::ssa_refactor::ir::value::ValueId as IrValueId;

pub(super) enum Nested<T> {
    Tuple(Vec<Nested<T>>),
    Single(T),
}

#[derive(Debug, Clone)]
pub(super) enum SingleValue {
    Normal(IrValueId),
    Function(IrFunctionId),

    /// Lazily inserting unit values helps prevent cluttering the IR with too many
    /// unit literals.
    Unit,
}

pub(super) type Value = Nested<SingleValue>;

impl<T> Nested<T> {
    pub(super) fn flatten(self) -> Vec<T> {
        match self {
            Nested::Tuple(values) => values.into_iter().flat_map(Nested::flatten).collect(),
            Nested::Single(value) => vec![value],
        }
    }
}

impl From<IrValueId> for Nested<SingleValue> {
    fn from(id: IrValueId) -> Self {
        Self::Single(SingleValue::Normal(id))
    }
}

impl From<IrValueId> for SingleValue {
    fn from(id: IrValueId) -> Self {
        SingleValue::Normal(id)
    }
}
