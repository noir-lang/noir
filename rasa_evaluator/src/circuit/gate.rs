use crate::polynomial::Arithmetic;
use crate::Function;

#[derive(Clone, Debug)]
pub struct Directive {
    // We evaluate the input and then feed it to the directive
    // This directive is created in the evaluator, also possible to create this at compile time
    // XXX: This is a bit tricky because we could have the case where the input disappears due to optimisations
    input_values: Vec<Arithmetic>,
    func: Function,
}

#[derive(Clone, Debug)]
pub enum Gate {
    Arithmetic(Arithmetic),
    Directive(Directive),
}
