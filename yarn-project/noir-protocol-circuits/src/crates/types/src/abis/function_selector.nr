struct FunctionSelector {
    // 1st 4-bytes of abi-encoding of function.
    inner: u32,
}

impl FunctionSelector {
    fn to_field(self) -> Field {
        self.inner as Field
    }

    pub fn from_u32(value: u32) -> FunctionSelector {
        FunctionSelector {
            inner : value,
        }
    }

    pub fn zero() -> Self {
        FunctionSelector { inner: 0 }
    }

    pub fn eq(self, function_selector: FunctionSelector) -> bool {
        function_selector.inner == self.inner
    }
}
