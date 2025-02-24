use iter_extended::vecmap;

use crate::ssa::ir::types::Type;
use crate::ssa::ir::value::ValueId as IrValueId;

use super::context::FunctionContext;

/// A general Tree structure which is used in the SSA generation pass
/// to represent both values and types which may be tuples.
///
/// Since the underlying SSA intermediate representation (IR) does not
/// support tuples directly, they're instead represented as Tree::Branch
/// nodes. For example, a single ssa value may be a Tree::Leaf(Value),
/// while a tuple would be a Tree::Branch(values).
#[derive(Debug, Clone)]
pub(super) enum Tree<T> {
    Branch(Vec<Tree<T>>),
    Leaf(T),
}

/// A single value in ssa form. This wrapper enum is needed mostly to enable
/// us to automatically create a Instruction::Load whenever a mutable variable
/// is referenced.
///
/// Note that these values wrap the ValueIds
/// used internally by functions in the ssa ir and should thus be isolated
/// to a given function. If used outside their function of origin, the IDs
/// would be invalid.
#[derive(Debug, Clone)]
pub(super) enum Value {
    Normal(IrValueId),

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
            Value::Mutable(address, typ) => ctx.builder.insert_load(address, typ),
        }
    }

    /// Evaluates the value, returning a reference to the mutable variable found within
    /// if possible. Compared to .eval, this method will not load from self if it is Value::Mutable.
    pub(super) fn eval_reference(self) -> IrValueId {
        match self {
            Value::Normal(value) => value,
            Value::Mutable(address, _) => address,
        }
    }
}

/// A tree of values.
///
/// Compared to Value alone, the addition of being able to represent structs/tuples as
/// a Tree::Branch means this type can hold any kind of value a frontend expression may return.
/// This is why it is used as the return type for every codegen_* function in ssa_gen/mod.rs.
pub(super) type Values = Tree<Value>;

impl<T> Tree<T> {
    /// Returns an empty tree node represented by a Branch with no branches
    pub(super) fn empty() -> Self {
        Tree::Branch(vec![])
    }

    /// Flattens the tree into a vector of each leaf value
    pub(super) fn flatten(self) -> Vec<T> {
        match self {
            Tree::Branch(values) => values.into_iter().flat_map(Tree::flatten).collect(),
            Tree::Leaf(value) => vec![value],
        }
    }

    /// Returns the total amount of leaves in this tree
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

    /// Calls the given function on each leaf node, mapping this tree into a new one.
    ///
    /// Because the given function returns a Tree<U> rather than a U, it is possible
    /// to use this function to turn Leaf nodes into either other Leaf nodes or even Branch nodes.
    pub(super) fn map<U>(self, mut f: impl FnMut(T) -> Tree<U>) -> Tree<U> {
        self.map_helper(&mut f)
    }

    fn map_helper<U>(self, f: &mut impl FnMut(T) -> Tree<U>) -> Tree<U> {
        match self {
            Tree::Branch(trees) => Tree::Branch(vecmap(trees, |tree| tree.map_helper(f))),
            Tree::Leaf(value) => f(value),
        }
    }

    /// Map two trees alongside each other.
    /// This asserts each tree has the same internal structure.
    pub(super) fn map_both<U, R>(
        &self,
        other: Tree<U>,
        mut f: impl FnMut(T, U) -> Tree<R>,
    ) -> Tree<R>
    where
        T: std::fmt::Debug + Clone,
        U: std::fmt::Debug,
    {
        self.map_both_helper(other, &mut f)
    }

    fn map_both_helper<U, R>(&self, other: Tree<U>, f: &mut impl FnMut(T, U) -> Tree<R>) -> Tree<R>
    where
        T: std::fmt::Debug + Clone,
        U: std::fmt::Debug,
    {
        match (self, other) {
            (Tree::Branch(self_trees), Tree::Branch(other_trees)) => {
                assert_eq!(self_trees.len(), other_trees.len());
                let trees = self_trees.iter().zip(other_trees);
                Tree::Branch(vecmap(trees, |(l, r)| l.map_both_helper(r, f)))
            }
            (Tree::Leaf(self_value), Tree::Leaf(other_value)) => f(self_value.clone(), other_value),
            other => panic!("Found unexpected tree combination during SSA: {other:?}"),
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
