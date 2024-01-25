use crate::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Memory {
    // Memory is a vector of values.
    // We grow the memory when values past the end are set, extending with 0s.
    inner: Vec<Value>,
}

impl From<Vec<Value>> for Memory {
    fn from(values: Vec<Value>) -> Self {
        Memory { inner: values }
    }
}

impl Memory {
    /// Gets the value at pointer
    pub fn read(&self, ptr: usize) -> Value {
        self.inner[ptr]
    }

    pub fn read_slice(&self, ptr: usize, len: usize) -> &[Value] {
        &self.inner[ptr..ptr + len]
    }

    /// Sets the value at pointer `ptr` to `value`
    pub fn write(&mut self, ptr: usize, value: Value) {
        self.write_slice(ptr, &[value]);
    }

    /// Sets the values after pointer `ptr` to `values`
    pub fn write_slice(&mut self, ptr: usize, values: &[Value]) {
        // Calculate new memory size
        let new_size = std::cmp::max(self.inner.len(), ptr + values.len());
        // Expand memory to new size with default values if needed
        self.inner.resize(new_size, Value::from(0_usize));

        self.inner[ptr..ptr + values.len()].copy_from_slice(values);
    }

    /// Returns the values of the memory
    pub fn values(&self) -> &[Value] {
        &self.inner
    }
}
