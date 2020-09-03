use crate::polynomial::Arithmetic;
use crate::Function;

#[derive(Clone, Debug)]
pub enum Gate {
    Arithmetic(Arithmetic),
}
