use acir::{brillig::MemoryAddress, AcirField};
use num_bigint::BigUint;
use num_traits::{One, Zero};

pub const MEMORY_ADDRESSING_BIT_SIZE: u32 = 64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryValue<F> {
    Field(F),
    Integer(BigUint, u32),
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryTypeError {
    #[error("Bit size for value {value_bit_size} does not match the expected bit size {expected_bit_size}")]
    MismatchedBitSize { value_bit_size: u32, expected_bit_size: u32 },
}

impl<F: AcirField> MemoryValue<F> {
    /// Builds a memory value from a field element.
    pub fn new_from_field(value: F, bit_size: u32) -> Self {
        if bit_size == F::max_num_bits() {
            MemoryValue::new_field(value)
        } else {
            MemoryValue::new_integer(BigUint::from_bytes_be(&value.to_be_bytes()), bit_size)
        }
    }

    /// Builds a memory value from an integer
    pub fn new_from_integer(value: BigUint, bit_size: u32) -> Self {
        if bit_size == F::max_num_bits() {
            MemoryValue::new_field(F::from_be_bytes_reduce(&value.to_bytes_be()))
        } else {
            MemoryValue::new_integer(value, bit_size)
        }
    }

    /// Builds a memory value from a field element, checking that the value is within the bit size.
    pub fn new_checked(value: F, bit_size: u32) -> Option<Self> {
        if bit_size < F::max_num_bits() && value.num_bits() > bit_size {
            return None;
        }

        Some(MemoryValue::new_from_field(value, bit_size))
    }

    /// Builds a field-typed memory value.
    pub fn new_field(value: F) -> Self {
        MemoryValue::Field(value)
    }

    /// Builds an integer-typed memory value.
    pub fn new_integer(value: BigUint, bit_size: u32) -> Self {
        assert!(
            bit_size != F::max_num_bits(),
            "Tried to build a field memory value via new_integer"
        );
        MemoryValue::Integer(value, bit_size)
    }

    /// Extracts the field element from the memory value, if it is typed as field element.
    pub fn extract_field(&self) -> Option<&F> {
        match self {
            MemoryValue::Field(value) => Some(value),
            _ => None,
        }
    }

    /// Extracts the integer from the memory value, if it is typed as integer.
    pub fn extract_integer(&self) -> Option<(&BigUint, u32)> {
        match self {
            MemoryValue::Integer(value, bit_size) => Some((value, *bit_size)),
            _ => None,
        }
    }

    /// Converts the memory value to a field element, independent of its type.
    pub fn to_field(&self) -> F {
        match self {
            MemoryValue::Field(value) => *value,
            MemoryValue::Integer(value, _) => F::from_be_bytes_reduce(&value.to_bytes_be()),
        }
    }

    /// Converts the memory value to an integer, independent of its type.
    pub fn to_integer(self) -> BigUint {
        match self {
            MemoryValue::Field(value) => BigUint::from_bytes_be(&value.to_be_bytes()),
            MemoryValue::Integer(value, _) => value,
        }
    }

    pub fn bit_size(&self) -> u32 {
        match self {
            MemoryValue::Field(_) => F::max_num_bits(),
            MemoryValue::Integer(_, bit_size) => *bit_size,
        }
    }

    pub fn to_usize(&self) -> usize {
        assert!(
            self.bit_size() == MEMORY_ADDRESSING_BIT_SIZE,
            "value is not typed as brillig usize"
        );
        self.extract_integer().unwrap().0.try_into().unwrap()
    }

    pub fn expect_field(&self) -> Result<&F, MemoryTypeError> {
        match self {
            MemoryValue::Integer(_, bit_size) => Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: *bit_size,
                expected_bit_size: F::max_num_bits(),
            }),
            MemoryValue::Field(field) => Ok(field),
        }
    }

    pub fn expect_integer_with_bit_size(
        &self,
        expected_bit_size: u32,
    ) -> Result<&BigUint, MemoryTypeError> {
        match self {
            MemoryValue::Integer(value, bit_size) => {
                if *bit_size != expected_bit_size {
                    return Err(MemoryTypeError::MismatchedBitSize {
                        value_bit_size: *bit_size,
                        expected_bit_size,
                    });
                }
                Ok(value)
            }
            MemoryValue::Field(_) => Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: F::max_num_bits(),
                expected_bit_size,
            }),
        }
    }
}

impl<F: std::fmt::Display> std::fmt::Display for MemoryValue<F> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self {
            MemoryValue::Field(value) => write!(f, "{}: field", value),
            MemoryValue::Integer(value, bit_size) => {
                let typ = match bit_size {
                    0 => "null".to_string(),
                    1 => "bool".to_string(),
                    _ => format!("u{}", bit_size),
                };
                write!(f, "{}: {}", value, typ)
            }
        }
    }
}

impl<F: AcirField> Default for MemoryValue<F> {
    fn default() -> Self {
        MemoryValue::new_integer(BigUint::zero(), 0)
    }
}

impl<F: AcirField> From<usize> for MemoryValue<F> {
    fn from(value: usize) -> Self {
        MemoryValue::new_integer(value.into(), MEMORY_ADDRESSING_BIT_SIZE)
    }
}

impl<F: AcirField> From<u64> for MemoryValue<F> {
    fn from(value: u64) -> Self {
        MemoryValue::new_integer(value.into(), 64)
    }
}

impl<F: AcirField> From<u32> for MemoryValue<F> {
    fn from(value: u32) -> Self {
        MemoryValue::new_integer(value.into(), 32)
    }
}

impl<F: AcirField> From<u8> for MemoryValue<F> {
    fn from(value: u8) -> Self {
        MemoryValue::new_integer(value.into(), 8)
    }
}

impl<F: AcirField> From<bool> for MemoryValue<F> {
    fn from(value: bool) -> Self {
        let value = if value { BigUint::one() } else { BigUint::zero() };
        MemoryValue::new_integer(value, 1)
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u64 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(64).map(|value| value.try_into().unwrap())
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u32 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(32).map(|value| value.try_into().unwrap())
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u8 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(8).map(|value| value.try_into().unwrap())
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for bool {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        let as_integer = memory_value.expect_integer_with_bit_size(1)?;

        if as_integer.is_zero() {
            Ok(false)
        } else if as_integer.is_one() {
            Ok(true)
        } else {
            unreachable!("value typed as bool is greater than one")
        }
    }
}

impl<F: AcirField> TryFrom<&MemoryValue<F>> for u64 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: &MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(64).map(|value| {
            value.try_into().expect("memory_value has been asserted to contain a 64 bit integer")
        })
    }
}

impl<F: AcirField> TryFrom<&MemoryValue<F>> for u32 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: &MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(32).map(|value| {
            value.try_into().expect("memory_value has been asserted to contain a 32 bit integer")
        })
    }
}

impl<F: AcirField> TryFrom<&MemoryValue<F>> for u8 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: &MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(8).map(|value| {
            value.try_into().expect("memory_value has been asserted to contain an 8 bit integer")
        })
    }
}

impl<F: AcirField> TryFrom<&MemoryValue<F>> for bool {
    type Error = MemoryTypeError;

    fn try_from(memory_value: &MemoryValue<F>) -> Result<Self, Self::Error> {
        let as_integer = memory_value.expect_integer_with_bit_size(1)?;

        if as_integer.is_zero() {
            Ok(false)
        } else if as_integer.is_one() {
            Ok(true)
        } else {
            unreachable!("value typed as bool is greater than one")
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory<F> {
    // Memory is a vector of values.
    // We grow the memory when values past the end are set, extending with 0s.
    inner: Vec<MemoryValue<F>>,
}

impl<F: AcirField> Memory<F> {
    /// Gets the value at pointer
    pub fn read(&self, ptr: MemoryAddress) -> MemoryValue<F> {
        self.inner.get(ptr.to_usize()).cloned().unwrap_or_default()
    }

    pub fn read_ref(&self, ptr: MemoryAddress) -> MemoryAddress {
        MemoryAddress(self.read(ptr).to_usize())
    }

    pub fn read_slice(&self, addr: MemoryAddress, len: usize) -> &[MemoryValue<F>] {
        // Allows to read a slice of uninitialized memory if the length is zero.
        // Ideally we'd be able to read uninitialized memory in general (as read does)
        // but that's not possible if we want to return a slice instead of owned data.
        if len == 0 {
            return &[];
        }
        &self.inner[addr.to_usize()..(addr.to_usize() + len)]
    }

    /// Sets the value at pointer `ptr` to `value`
    pub fn write(&mut self, ptr: MemoryAddress, value: MemoryValue<F>) {
        self.resize_to_fit(ptr.to_usize() + 1);
        self.inner[ptr.to_usize()] = value;
    }

    fn resize_to_fit(&mut self, size: usize) {
        // Calculate new memory size
        let new_size = std::cmp::max(self.inner.len(), size);
        // Expand memory to new size with default values if needed
        self.inner.resize(new_size, MemoryValue::default());
    }

    /// Sets the values after pointer `ptr` to `values`
    pub fn write_slice(&mut self, ptr: MemoryAddress, values: &[MemoryValue<F>]) {
        self.resize_to_fit(ptr.to_usize() + values.len());
        self.inner[ptr.to_usize()..(ptr.to_usize() + values.len())].clone_from_slice(values);
    }

    /// Returns the values of the memory
    pub fn values(&self) -> &[MemoryValue<F>] {
        &self.inner
    }
}
