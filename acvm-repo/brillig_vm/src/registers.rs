use acir::brillig::{RegisterIndex, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registers {
    // Registers are a vector of values.
    // We grow the register as registers past the end are set, extending with 0s.
    pub inner: Vec<Value>,
}

/// Aims to match a reasonable max register count for a SNARK prover.
/// As well, catches obvious erroneous use of registers.
/// This can be revisited if it proves not enough.
const MAX_REGISTERS: usize = 2_usize.pow(16);

/// Registers will store field element values during the
/// duration of the execution of the bytecode.
impl Registers {
    /// Create a Registers object initialized with definite values
    pub fn load(values: Vec<Value>) -> Registers {
        let inner = values.into_iter().collect();
        Self { inner }
    }

    /// Gets the values at register with address `index`
    pub fn get(&self, register_index: RegisterIndex) -> Value {
        let index = register_index.to_usize();
        assert!(index < MAX_REGISTERS, "Reading register past maximum!");
        let value = self.inner.get(index);
        match value {
            Some(value) => *value,
            None => 0u128.into(),
        }
    }

    /// Sets the value at register with address `index` to `value`
    pub fn set(&mut self, RegisterIndex(index): RegisterIndex, value: Value) {
        assert!(index < MAX_REGISTERS, "Writing register past maximum!");
        // if size isn't at least index + 1, resize
        let new_register_size = std::cmp::max(index + 1, self.inner.len());
        self.inner.resize(new_register_size, 0u128.into());
        self.inner[index] = value;
    }
}
