struct FunctionSelector {
    // 1st 4-bytes of abi-encoding of function.
    inner : u32,
}

impl FunctionSelector {
    fn to_field(self) -> Field {
        self.inner as Field
    }

    pub fn from_u32(value : u32) -> FunctionSelector {
        FunctionSelector {
            inner : value,
        }
    } 
}
