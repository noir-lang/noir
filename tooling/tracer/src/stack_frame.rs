use acvm::FieldElement;
use noirc_printable_type::{PrintableType, PrintableValue};

/// A stack frame during execution of the program being traced.
#[derive(Clone, Debug)]
pub(crate) struct StackFrame {
    pub(crate) function_name: String,
    /// The param indexes are used to address the `variables` vector; they indicate which of the
    /// variables in the stack frame are parameters.
    pub(crate) function_param_indexes: Vec<usize>,
    /// A sorted vector of the frame variables. Not enforced, so beware of bugs.
    pub(crate) variables: Vec<Variable>,
}

// Implement PartialEq for StackFrame manually, because it only needs to use the function_name as an
// identifier of the StackFrame. Considered using the derivative crate, but decided against it to
// keep dependency complexity low.
impl PartialEq for StackFrame {
    fn eq(&self, other: &Self) -> bool {
        self.function_name == other.function_name
    }
}

/// A representation of a variable on the stack. This representation owns the data that makes up the
/// variable -- the name, type, and value of the variable -- unlike the representation used by the
/// debugger.
#[derive(Clone, Debug)]
pub(crate) struct Variable {
    pub(crate) name: String,
    pub(crate) value: PrintableValue<FieldElement>,
    pub(crate) typ: PrintableType,
}

// The following trait implementations are added so that a collection of `Variable`s can be sorted
// using `.sort()`.
impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for Variable {
    fn assert_receiver_is_total_eq(&self) {
        self.name.assert_receiver_is_total_eq()
    }
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Variable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Variable {
    /// Creates an instance from a tuple to references of the contents. Clones the variables
    /// pointed to by the references.
    ///
    /// It has this rather odd form and not a proper argument list, because this is the standard way
    /// variables are represented in the nargo debugger.
    pub(crate) fn from_tuple(
        args: &(&str, &PrintableValue<FieldElement>, &PrintableType),
    ) -> Variable {
        Variable { name: String::from(args.0), value: args.1.clone(), typ: args.2.clone() }
    }
}
