struct FunctionSelector {
    // 1st 4-bytes of abi-encoding of function.
    inner: u32,
}

impl FunctionSelector {
    fn to_field(self) -> Field {
        self.inner as Field
    }

    pub fn from_u32(value: u32) -> Self {
        Self {
            inner : value,
        }
    }

    pub fn from_field(value : Field) -> Self {
        Self {
            inner : value as u32,
        }
    }

    pub fn zero() -> Self {
        FunctionSelector { inner: 0 }
    }

    pub fn eq(self, function_selector: FunctionSelector) -> bool {
        function_selector.inner == self.inner
    }

    pub fn serialize(self: Self) -> [Field; 1] {
        [self.inner as Field]
    }

    pub fn deserialize(fields: [Field; 1]) -> Self {
        Self {
            inner: fields[0] as u32
        }
    }
}
