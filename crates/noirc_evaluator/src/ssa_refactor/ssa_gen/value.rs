use crate::ssa_refactor::ir::function::FunctionId as IrFunctionId;
use crate::ssa_refactor::ir::value::ValueId;

#[derive(Debug, Clone)]
pub(super) enum Value {
    Normal(ValueId),
    Function(IrFunctionId),
    Tuple(Vec<Value>),

    /// Lazily inserting unit values helps prevent cluttering the IR with too many
    /// unit literals.
    Unit,
}
