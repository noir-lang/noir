use acir::{
    brillig::{BitSize, IntegerBitSize, MemoryAddress},
    AcirField,
};
use num_traits::{One, Zero};

pub const MEMORY_ADDRESSING_BIT_SIZE: IntegerBitSize = IntegerBitSize::U32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryValue<F> {
    Field(F),
    Integer(u128, IntegerBitSize),
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryTypeError {
    #[error("Bit size for value {value_bit_size} does not match the expected bit size {expected_bit_size}")]
    MismatchedBitSize { value_bit_size: u32, expected_bit_size: u32 },
}

impl<F> MemoryValue<F> {
    /// Builds a field-typed memory value.
    pub fn new_field(value: F) -> Self {
        MemoryValue::Field(value)
    }

    /// Builds an integer-typed memory value.
    pub fn new_integer(value: u128, bit_size: IntegerBitSize) -> Self {
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
    pub fn extract_integer(&self) -> Option<(u128, IntegerBitSize)> {
        match self {
            MemoryValue::Integer(value, bit_size) => Some((*value, *bit_size)),
            _ => None,
        }
    }

    pub fn bit_size(&self) -> BitSize {
        match self {
            MemoryValue::Field(_) => BitSize::Field,
            MemoryValue::Integer(_, bit_size) => BitSize::Integer(*bit_size),
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            MemoryValue::Integer(_, bit_size) if *bit_size == MEMORY_ADDRESSING_BIT_SIZE => {
                self.extract_integer().unwrap().0.try_into().unwrap()
            }
            _ => panic!("value is not typed as brillig usize"),
        }
    }
}

impl<F: AcirField> MemoryValue<F> {
    /// Builds a memory value from a field element.
    pub fn new_from_field(value: F, bit_size: BitSize) -> Self {
        if let BitSize::Integer(bit_size) = bit_size {
            MemoryValue::new_integer(value.to_u128(), bit_size)
        } else {
            MemoryValue::new_field(value)
        }
    }

    /// Builds a memory value from a field element, checking that the value is within the bit size.
    pub fn new_checked(value: F, bit_size: BitSize) -> Option<Self> {
        if let BitSize::Integer(bit_size) = bit_size {
            if value.num_bits() > bit_size.into() {
                return None;
            }
        }

        Some(MemoryValue::new_from_field(value, bit_size))
    }

    /// Converts the memory value to a field element, independent of its type.
    pub fn to_field(&self) -> F {
        match self {
            MemoryValue::Field(value) => *value,
            MemoryValue::Integer(value, _) => F::from(*value),
        }
    }

    pub fn expect_field(&self) -> Result<&F, MemoryTypeError> {
        match self {
            MemoryValue::Integer(_, bit_size) => Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: (*bit_size).into(),
                expected_bit_size: F::max_num_bits(),
            }),
            MemoryValue::Field(field) => Ok(field),
        }
    }

    pub fn expect_integer_with_bit_size(
        &self,
        expected_bit_size: IntegerBitSize,
    ) -> Result<u128, MemoryTypeError> {
        match self {
            MemoryValue::Integer(value, bit_size) => {
                if *bit_size != expected_bit_size {
                    return Err(MemoryTypeError::MismatchedBitSize {
                        value_bit_size: (*bit_size).into(),
                        expected_bit_size: expected_bit_size.into(),
                    });
                }
                Ok(*value)
            }
            MemoryValue::Field(_) => Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: F::max_num_bits(),
                expected_bit_size: expected_bit_size.into(),
            }),
        }
    }
}

impl<F: std::fmt::Display> std::fmt::Display for MemoryValue<F> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self {
            MemoryValue::Field(value) => write!(f, "{}: field", value),
            MemoryValue::Integer(value, bit_size) => {
                write!(f, "{}: {}", value, bit_size)
            }
        }
    }
}

impl<F: AcirField> Default for MemoryValue<F> {
    fn default() -> Self {
        MemoryValue::new_field(F::zero())
    }
}

impl<F: AcirField> From<bool> for MemoryValue<F> {
    fn from(value: bool) -> Self {
        let value = if value { 1 } else { 0 };
        MemoryValue::new_integer(value, IntegerBitSize::U1)
    }
}

impl<F: AcirField> From<u8> for MemoryValue<F> {
    fn from(value: u8) -> Self {
        MemoryValue::new_integer(value.into(), IntegerBitSize::U8)
    }
}

impl<F: AcirField> From<usize> for MemoryValue<F> {
    fn from(value: usize) -> Self {
        MemoryValue::new_integer(value as u128, MEMORY_ADDRESSING_BIT_SIZE)
    }
}

impl<F: AcirField> From<u32> for MemoryValue<F> {
    fn from(value: u32) -> Self {
        MemoryValue::new_integer(value.into(), IntegerBitSize::U32)
    }
}

impl<F: AcirField> From<u64> for MemoryValue<F> {
    fn from(value: u64) -> Self {
        MemoryValue::new_integer(value.into(), IntegerBitSize::U64)
    }
}

impl<F: AcirField> From<u128> for MemoryValue<F> {
    fn from(value: u128) -> Self {
        MemoryValue::new_integer(value, IntegerBitSize::U128)
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for bool {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        let as_integer = memory_value.expect_integer_with_bit_size(IntegerBitSize::U1)?;

        if as_integer.is_zero() {
            Ok(false)
        } else if as_integer.is_one() {
            Ok(true)
        } else {
            unreachable!("value typed as bool is greater than one")
        }
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u8 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(IntegerBitSize::U8).map(|value| value as u8)
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u32 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(IntegerBitSize::U32).map(|value| value as u32)
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u64 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(IntegerBitSize::U64).map(|value| value as u64)
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u128 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_integer_with_bit_size(IntegerBitSize::U128)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory<F> {
    // Memory is a vector of values.
    // We grow the memory when values past the end are set, extending with 0s.
    inner: Vec<MemoryValue<F>>,
}

impl<F: AcirField> Memory<F> {
    fn get_stack_pointer(&self) -> usize {
        self.read(MemoryAddress::Direct(0)).to_usize()
    }

    fn resolve(&self, address: MemoryAddress) -> usize {
        match address {
            MemoryAddress::Direct(address) => address,
            MemoryAddress::Relative(offset) => self.get_stack_pointer() + offset,
        }
    }

    /// Gets the value at address
    pub fn read(&self, address: MemoryAddress) -> MemoryValue<F> {
        let resolved_addr = self.resolve(address);
        self.inner.get(resolved_addr).copied().unwrap_or_default()
    }

    pub fn read_ref(&self, ptr: MemoryAddress) -> MemoryAddress {
        MemoryAddress::direct(self.read(ptr).to_usize())
    }

    pub fn read_slice(&self, addr: MemoryAddress, len: usize) -> &[MemoryValue<F>] {
        // Allows to read a slice of uninitialized memory if the length is zero.
        // Ideally we'd be able to read uninitialized memory in general (as read does)
        // but that's not possible if we want to return a slice instead of owned data.
        if len == 0 {
            return &[];
        }
        let resolved_addr = self.resolve(addr);
        &self.inner[resolved_addr..(resolved_addr + len)]
    }

    /// Sets the value at `address` to `value`
    pub fn write(&mut self, address: MemoryAddress, value: MemoryValue<F>) {
        let resolved_ptr = self.resolve(address);
        self.resize_to_fit(resolved_ptr + 1);
        self.inner[resolved_ptr] = value;
    }

    fn resize_to_fit(&mut self, size: usize) {
        // Calculate new memory size
        let new_size = std::cmp::max(self.inner.len(), size);
        // Expand memory to new size with default values if needed
        self.inner.resize(new_size, MemoryValue::default());
    }

    /// Sets the values after `address` to `values`
    pub fn write_slice(&mut self, address: MemoryAddress, values: &[MemoryValue<F>]) {
        let resolved_address = self.resolve(address);
        self.resize_to_fit(resolved_address + values.len());
        self.inner[resolved_address..(resolved_address + values.len())].copy_from_slice(values);
    }

    /// Returns the values of the memory
    pub fn values(&self) -> &[MemoryValue<F>] {
        &self.inner
    }
}
