use crate::ssa_refactor::ir::function::FunctionId as IrFunctionId;
use crate::ssa_refactor::ir::types::Type;
use crate::ssa_refactor::ir::value::ValueId as IrValueId;

#[derive(Debug)]
pub(super) enum Tree<T> {
    Branch(Vec<Tree<T>>),
    Leaf(T),
}

#[derive(Debug, Clone)]
pub(super) enum Value {
    Normal(IrValueId),
    Function(IrFunctionId),
}

impl Value {
    /// Evaluate a value, returning an IrValue from it.
    /// This has no effect on Value::Normal, but any variables will be updated with their latest
    /// use.
    pub(super) fn eval(self) -> IrValueId {
        match self {
            Value::Normal(value) => value,
            Value::Function(_) => panic!("Tried to evaluate a function value"),
        }
    }
}

pub(super) type Values = Tree<Value>;

impl<T> Tree<T> {
    pub(super) fn flatten(self) -> Vec<T> {
        match self {
            Tree::Branch(values) => values.into_iter().flat_map(Tree::flatten).collect(),
            Tree::Leaf(value) => vec![value],
        }
    }

    pub(super) fn count_leaves(&self) -> usize {
        match self {
            Tree::Branch(trees) => trees.iter().map(|tree| tree.count_leaves()).sum(),
            Tree::Leaf(_) => 1,
        }
    }

    /// Iterates over each Leaf node, calling f on each value within.
    pub(super) fn for_each(self, mut f: impl FnMut(T)) {
        self.for_each_helper(&mut f);
    }

    fn for_each_helper(self, f: &mut impl FnMut(T)) {
        match self {
            Tree::Branch(trees) => trees.into_iter().for_each(|tree| tree.for_each_helper(f)),
            Tree::Leaf(value) => f(value),
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

// Specialize this impl just to give a better name for this function
impl Tree<Type> {
    /// Returns the size of the type in terms of the number of FieldElements it contains.
    /// Non-field types like functions and references are also counted as 1 FieldElement.
    pub(super) fn size_of_type(&self) -> usize {
        self.count_leaves()
    }
}
