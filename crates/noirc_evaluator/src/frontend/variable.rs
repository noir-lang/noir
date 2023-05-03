/// A variable in the SSA IR.
/// By definition, a variable can only be defined once.
///
/// As in Cranelift, we also allow variable use before definition.
/// This will produce side-effects which will need to be handled
/// before sealing a block.
pub struct Variable(u32);

impl From<u32> for Variable {
    fn from(value: u32) -> Self {
        Variable(value)
    }
}
impl From<u16> for Variable {
    fn from(value: u16) -> Self {
        Variable(value as u32)
    }
}
impl From<u8> for Variable {
    fn from(value: u8) -> Self {
        Variable(value as u32)
    }
}
