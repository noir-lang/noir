use crate::ssa_refactor::ir::function::FunctionId as IrFunctionId;
use crate::ssa_refactor::ir::value::ValueId as IrValueId;

pub(super) enum Tree<T> {
    Branch(Vec<Tree<T>>),
    Leaf(T),
}

#[derive(Debug, Clone)]
pub(super) enum Value {
    Normal(IrValueId),
    Function(IrFunctionId),

    /// Lazily inserting unit values helps prevent cluttering the IR with too many
    /// unit literals.
    Unit,
}

pub(super) type Values = Tree<Value>;

impl<T> Tree<T> {
    pub(super) fn flatten(self) -> Vec<T> {
        match self {
            Tree::Branch(values) => values.into_iter().flat_map(Tree::flatten).collect(),
            Tree::Leaf(value) => vec![value],
        }
    }
}

impl From<IrValueId> for Values {
    fn from(id: IrValueId) -> Self {
        Self::Leaf(Value::Normal(id))
    }
}

impl From<IrValueId> for Value {
    fn from(id: IrValueId) -> Self {
        Value::Normal(id)
    }
}
