use std::fmt::Display;

use crate::{Ident, NoirFunction};

#[derive(Clone, Debug)]
pub struct NoirContract {
    pub name: Ident,
    pub functions: Vec<NoirFunction>,
}

impl Display for NoirContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "contract {} {{", self.name)?;

        for function in self.functions.iter() {
            for line in function.to_string().lines() {
                writeln!(f, "    {}", line)?;
            }
        }

        write!(f, "}}")
    }
}
