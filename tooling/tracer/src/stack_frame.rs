/// A stack frame during execution of the program being traced.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StackFrame {
    pub(crate) function_name: String,
}
