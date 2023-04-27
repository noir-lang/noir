use iter_extended::vecmap;

use crate::ssa_refactor::ir::function::FunctionId as IrFunctionId;
use crate::ssa_refactor::ir::types::Type;
use crate::ssa_refactor::ir::value::ValueId as IrValueId;

use super::context::FunctionContext;

#[derive(Debug, Clone)]
pub(super) enum Tree<T> {
    Branch(Vec<Tree<T>>),
    Leaf(T),
}

#[derive(Debug, Copy, Clone)]
pub(super) enum Value {
    Normal(IrValueId),
    Function(IrFunctionId),

    /// A mutable variable that must be loaded as the given type before being used
    Mutable(IrValueId, Type),
}

impl Value {
    /// Evaluate a value, returning an IrValue from it.
    /// This has no effect on Value::Normal, but any variables will
    /// need to be loaded from memory
    pub(super) fn eval(self, ctx: &mut FunctionContext) -> IrValueId {
        match self {
            Value::Normal(value) => value,
            Value::Mutable(address, typ) => {
                let offset = ctx.builder.field_constant(0u128);
                ctx.builder.insert_load(address, offset, typ)
            }
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

    pub(super) fn map_mut(&mut self, mut f: impl FnMut(&T) -> Tree<T>) {
        self.map_mut_helper(&mut f);
    }

    fn map_mut_helper(&mut self, f: &mut impl FnMut(&T) -> Tree<T>) {
        match self {
            Tree::Branch(trees) => trees.iter_mut().for_each(|tree| tree.map_mut_helper(f)),
            Tree::Leaf(value) => *self = f(value),
        }
    }

    pub(super) fn map<U>(self, mut f: impl FnMut(T) -> Tree<U>) -> Tree<U> {
        self.map_helper(&mut f)
    }

    fn map_helper<U>(self, f: &mut impl FnMut(T) -> Tree<U>) -> Tree<U> {
        match self {
            Tree::Branch(trees) => Tree::Branch(vecmap(trees, |tree| tree.map_helper(f))),
            Tree::Leaf(value) => f(value),
        }
    }

    /// Unwraps this Tree into the value of the leaf node. Panics if
    /// this Tree is a Branch
    pub(super) fn into_leaf(self) -> T {
        match self {
            Tree::Branch(_) => panic!("into_leaf called on a Tree::Branch"),
            Tree::Leaf(value) => value,
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

impl Tree<Value> {
    /// Flattens and evaluates this Tree<Value> into a list of ir values
    /// for return statements, branching instructions, or function parameters.
    pub(super) fn into_value_list(self, ctx: &mut FunctionContext) -> Vec<IrValueId> {
        vecmap(self.flatten(), |value| value.eval(ctx))
    }
}
