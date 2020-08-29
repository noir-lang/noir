use crate::polynomial::Arithmetic;

#[derive(Clone, Debug)]
pub enum Gate {
    Arithmetic(Arithmetic),
}