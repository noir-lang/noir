//! Implementation of the VM's memory.
use acir::{
    AcirField,
    brillig::{BitSize, IntegerBitSize, MemoryAddress},
};

/// The bit size used for addressing memory within the Brillig VM.
///
/// All memory pointers are interpreted as `u32` values, meaning the VM can directly address up to 2^32 memory slots.
pub const MEMORY_ADDRESSING_BIT_SIZE: IntegerBitSize = IntegerBitSize::U32;

/// The current stack pointer is always in slot 0.
///
/// It gets manipulated by opcodes laid down for calls by codegen.
pub const STACK_POINTER_ADDRESS: MemoryAddress = MemoryAddress::Direct(0);

/// The _free memory pointer_ is always in slot 1.
///
/// We added it here to be able to implement a workaround for wrapping around
/// the free memory, ie. to detect "out of memory" events, but the AVM is not,
/// and does not want to be aware of the _free memory pointer_, so we cannot,
/// in general, build much functionality in the VM around it.
pub const FREE_MEMORY_POINTER_ADDRESS: MemoryAddress = MemoryAddress::Direct(1);

/// Offset constants for arrays and vectors:
/// * Arrays are `[ref-count, ...items]`
/// * Vectors are `[ref-count, size, capacity, ...items]`
pub mod offsets {
    /// Number of prefix fields in an array: RC.
    pub const ARRAY_META_COUNT: usize = 1;
    pub const ARRAY_ITEMS: usize = 1;

    /// Number of prefix fields in a vector: RC, size, capacity.
    pub const VECTOR_META_COUNT: usize = 3;
    pub const VECTOR_SIZE: usize = 1;
    pub const VECTOR_CAPACITY: usize = 2;
    pub const VECTOR_ITEMS: usize = 3;
}

/// Wrapper for array addresses, with convenience methods for various offsets.
///
/// The array consists of a ref-count followed by a number of items according
/// the size indicated by the type.
pub(crate) struct ArrayAddress(MemoryAddress);

impl ArrayAddress {
    /// The start of the items, after the meta-data.
    pub(crate) fn items_start(&self) -> MemoryAddress {
        self.0.offset(offsets::ARRAY_ITEMS)
    }
}

impl From<MemoryAddress> for ArrayAddress {
    fn from(value: MemoryAddress) -> Self {
        Self(value)
    }
}

/// Wrapper for vector addresses, with convenience methods for various offsets.
///
/// A vector is prefixed by 3 meta-data fields: the ref-count, the size, and the capacity,
/// which are followed by a number of items indicated by its capacity, with the items
/// its size being placeholders to accommodate future growth.
///
/// The semantic length of the vector is maintained at a separate address.
pub(crate) struct VectorAddress(MemoryAddress);

impl VectorAddress {
    /// Size of the vector.
    pub(crate) fn size_addr(&self) -> MemoryAddress {
        self.0.offset(offsets::VECTOR_SIZE)
    }
    /// The start of the items, after the meta-data.
    pub(crate) fn items_start(&self) -> MemoryAddress {
        self.0.offset(offsets::VECTOR_ITEMS)
    }
}

impl From<MemoryAddress> for VectorAddress {
    fn from(value: MemoryAddress) -> Self {
        Self(value)
    }
}

/// A single typed value in the Brillig VM's memory.
///
/// Memory in the VM is strongly typed and can represent either a native field element
/// or an integer of a specific bit width. This enum encapsulates all supported
/// in-memory types and allows conversion between representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryValue<F> {
    Field(F),
    U1(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
}

/// Represents errors that can occur when interpreting or converting typed memory values.
#[derive(Debug, thiserror::Error)]
pub enum MemoryTypeError {
    /// The value's bit size does not match the expected bit size for the operation.
    #[error(
        "Bit size for value {value_bit_size} does not match the expected bit size {expected_bit_size}"
    )]
    MismatchedBitSize { value_bit_size: u32, expected_bit_size: u32 },
    /// The memory value is not an integer and cannot be interpreted as one.
    /// For example, this can be triggered when attempting to convert a field element to an integer such as in [MemoryValue::to_u128].
    #[error("Value is not an integer")]
    NotAnInteger,
}

impl<F: std::fmt::Display> MemoryValue<F> {
    /// Builds a field-typed memory value.
    pub fn new_field(value: F) -> Self {
        MemoryValue::Field(value)
    }

    /// Builds an integer-typed memory value.
    pub fn new_integer(value: u128, bit_size: IntegerBitSize) -> Self {
        match bit_size {
            IntegerBitSize::U1 => MemoryValue::U1(value != 0),
            IntegerBitSize::U8 => MemoryValue::U8(value as u8),
            IntegerBitSize::U16 => MemoryValue::U16(value as u16),
            IntegerBitSize::U32 => MemoryValue::U32(value as u32),
            IntegerBitSize::U64 => MemoryValue::U64(value as u64),
            IntegerBitSize::U128 => MemoryValue::U128(value),
        }
    }

    pub fn bit_size(&self) -> BitSize {
        match self {
            MemoryValue::Field(_) => BitSize::Field,
            MemoryValue::U1(_) => BitSize::Integer(IntegerBitSize::U1),
            MemoryValue::U8(_) => BitSize::Integer(IntegerBitSize::U8),
            MemoryValue::U16(_) => BitSize::Integer(IntegerBitSize::U16),
            MemoryValue::U32(_) => BitSize::Integer(IntegerBitSize::U32),
            MemoryValue::U64(_) => BitSize::Integer(IntegerBitSize::U64),
            MemoryValue::U128(_) => BitSize::Integer(IntegerBitSize::U128),
        }
    }

    /// Expects a `U32` value and converts it into `usize`, otherwise panics.
    ///
    /// Primarily a convenience method for using values in memory operations as pointers, sizes and offsets.
    pub fn to_usize(&self) -> usize {
        match self {
            MemoryValue::U32(value) => (*value).try_into().unwrap(),
            other => panic!("value is not typed as Brillig usize: {other}"),
        }
    }
}

impl<F: AcirField> MemoryValue<F> {
    /// Builds a memory value from a field element, either field or integer type.
    ///
    /// If the bit size indicates an integer type, the value is downcast to fit into the specified size.
    pub fn new_from_field(value: F, bit_size: BitSize) -> Self {
        if let BitSize::Integer(bit_size) = bit_size {
            MemoryValue::new_integer(value.to_u128(), bit_size)
        } else {
            MemoryValue::new_field(value)
        }
    }

    /// Builds a memory value from a field element, checking that the value is within the bit size,
    /// otherwise returns `None`.
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
            MemoryValue::U1(value) => F::from(*value),
            MemoryValue::U8(value) => F::from(u128::from(*value)),
            MemoryValue::U16(value) => F::from(u128::from(*value)),
            MemoryValue::U32(value) => F::from(u128::from(*value)),
            MemoryValue::U64(value) => F::from(u128::from(*value)),
            MemoryValue::U128(value) => F::from(*value),
        }
    }

    /// Converts the memory value to U128, if the value is an integer.
    pub fn to_u128(&self) -> Result<u128, MemoryTypeError> {
        match self {
            MemoryValue::Field(..) => Err(MemoryTypeError::NotAnInteger),
            MemoryValue::U1(value) => Ok(u128::from(*value)),
            MemoryValue::U8(value) => Ok(u128::from(*value)),
            MemoryValue::U16(value) => Ok(u128::from(*value)),
            MemoryValue::U32(value) => Ok(u128::from(*value)),
            MemoryValue::U64(value) => Ok(u128::from(*value)),
            MemoryValue::U128(value) => Ok(*value),
        }
    }

    /// Extracts the field element from the memory value, if it is typed as field element.
    pub fn expect_field(self) -> Result<F, MemoryTypeError> {
        if let MemoryValue::Field(field) = self {
            Ok(field)
        } else {
            Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: self.bit_size().to_u32::<F>(),
                expected_bit_size: F::max_num_bits(),
            })
        }
    }
    pub(crate) fn expect_u1(self) -> Result<bool, MemoryTypeError> {
        if let MemoryValue::U1(value) = self {
            Ok(value)
        } else {
            Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: self.bit_size().to_u32::<F>(),
                expected_bit_size: 1,
            })
        }
    }

    pub(crate) fn expect_u8(self) -> Result<u8, MemoryTypeError> {
        if let MemoryValue::U8(value) = self {
            Ok(value)
        } else {
            Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: self.bit_size().to_u32::<F>(),
                expected_bit_size: 8,
            })
        }
    }

    pub(crate) fn expect_u16(self) -> Result<u16, MemoryTypeError> {
        if let MemoryValue::U16(value) = self {
            Ok(value)
        } else {
            Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: self.bit_size().to_u32::<F>(),
                expected_bit_size: 16,
            })
        }
    }

    pub(crate) fn expect_u32(self) -> Result<u32, MemoryTypeError> {
        if let MemoryValue::U32(value) = self {
            Ok(value)
        } else {
            Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: self.bit_size().to_u32::<F>(),
                expected_bit_size: 32,
            })
        }
    }

    pub(crate) fn expect_u64(self) -> Result<u64, MemoryTypeError> {
        if let MemoryValue::U64(value) = self {
            Ok(value)
        } else {
            Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: self.bit_size().to_u32::<F>(),
                expected_bit_size: 64,
            })
        }
    }

    pub(crate) fn expect_u128(self) -> Result<u128, MemoryTypeError> {
        if let MemoryValue::U128(value) = self {
            Ok(value)
        } else {
            Err(MemoryTypeError::MismatchedBitSize {
                value_bit_size: self.bit_size().to_u32::<F>(),
                expected_bit_size: 128,
            })
        }
    }
}

impl<F: std::fmt::Display> std::fmt::Display for MemoryValue<F> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self {
            MemoryValue::Field(value) => write!(f, "{value}: field"),
            MemoryValue::U1(value) => write!(f, "{value}: u1"),
            MemoryValue::U8(value) => write!(f, "{value}: u8"),
            MemoryValue::U16(value) => write!(f, "{value}: u16"),
            MemoryValue::U32(value) => write!(f, "{value}: u32"),
            MemoryValue::U64(value) => write!(f, "{value}: u64"),
            MemoryValue::U128(value) => write!(f, "{value}: u128"),
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
        MemoryValue::U1(value)
    }
}

impl<F: AcirField> From<u8> for MemoryValue<F> {
    fn from(value: u8) -> Self {
        MemoryValue::U8(value)
    }
}

impl<F: AcirField> From<usize> for MemoryValue<F> {
    fn from(value: usize) -> Self {
        MemoryValue::U32(value as u32)
    }
}

impl<F: AcirField> From<u32> for MemoryValue<F> {
    fn from(value: u32) -> Self {
        MemoryValue::U32(value)
    }
}

impl<F: AcirField> From<u64> for MemoryValue<F> {
    fn from(value: u64) -> Self {
        MemoryValue::U64(value)
    }
}

impl<F: AcirField> From<u128> for MemoryValue<F> {
    fn from(value: u128) -> Self {
        MemoryValue::U128(value)
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for bool {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_u1()
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u8 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_u8()
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u32 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_u32()
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u64 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_u64()
    }
}

impl<F: AcirField> TryFrom<MemoryValue<F>> for u128 {
    type Error = MemoryTypeError;

    fn try_from(memory_value: MemoryValue<F>) -> Result<Self, Self::Error> {
        memory_value.expect_u128()
    }
}
/// The VM's memory.
/// Memory is internally represented as a vector of values.
/// We grow the memory when values past the end are set, extending with 0s.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Memory<F> {
    // Internal memory representation
    inner: Vec<MemoryValue<F>>,
}

impl<F: AcirField> Memory<F> {
    /// Read the value from slot 0.
    ///
    /// Panics if it's not a `U32`.
    fn get_stack_pointer(&self) -> usize {
        self.read(STACK_POINTER_ADDRESS).to_usize()
    }

    /// Resolve an address to either:
    /// * itself, if it's a direct address, or
    /// * the current stack pointer plus the offset, if it's relative.
    ///
    /// Returns a memory slot index.
    fn resolve(&self, address: MemoryAddress) -> usize {
        match address {
            MemoryAddress::Direct(address) => address,
            MemoryAddress::Relative(offset) => self.get_stack_pointer() + offset,
        }
    }

    /// Reads the numeric value at the address.
    ///
    /// If the address is beyond the size of memory, a default value is returned.
    pub fn read(&self, address: MemoryAddress) -> MemoryValue<F> {
        let resolved_addr = self.resolve(address);
        self.inner.get(resolved_addr).copied().unwrap_or_default()
    }

    /// Reads the value at the address and returns it as a direct memory address,
    /// without dereferencing the pointer itself to a numeric value.
    pub fn read_ref(&self, ptr: MemoryAddress) -> MemoryAddress {
        MemoryAddress::direct(self.read(ptr).to_usize())
    }

    /// Sets `ptr` to point at `address`.
    pub fn write_ref(&mut self, ptr: MemoryAddress, address: MemoryAddress) {
        self.write(ptr, MemoryValue::from(address.to_usize()));
    }

    /// Read a contiguous vector of memory starting at `address`, up to `len` slots.
    ///
    /// Panics if the end index is beyond the size of the memory.
    pub fn read_vector(&self, address: MemoryAddress, len: usize) -> &[MemoryValue<F>] {
        // Allows to read a vector of uninitialized memory if the length is zero.
        // Ideally we'd be able to read uninitialized memory in general (as read does)
        // but that's not possible if we want to return a vector instead of owned data.
        if len == 0 {
            return &[];
        }
        let resolved_addr = self.resolve(address);
        &self.inner[resolved_addr..(resolved_addr + len)]
    }

    /// Sets the value at `address` to `value`
    pub fn write(&mut self, address: MemoryAddress, value: MemoryValue<F>) {
        let resolved_addr = self.resolve(address);
        self.resize_to_fit(resolved_addr + 1);
        self.inner[resolved_addr] = value;
    }

    /// Increase the size of memory fit `size` elements, or the current length, whichever is bigger.
    fn resize_to_fit(&mut self, size: usize) {
        // Calculate new memory size
        let new_size = std::cmp::max(self.inner.len(), size);
        // Expand memory to new size with default values if needed
        self.inner.resize(new_size, MemoryValue::default());
    }

    /// Sets the values after `address` to `values`
    pub fn write_vector(&mut self, address: MemoryAddress, values: &[MemoryValue<F>]) {
        let resolved_addr = self.resolve(address);
        let end_addr = resolved_addr + values.len();
        self.resize_to_fit(end_addr);
        self.inner[resolved_addr..end_addr].copy_from_slice(values);
    }

    /// Returns the values of the memory
    pub fn values(&self) -> &[MemoryValue<F>] {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir::FieldElement;

    #[test]
    fn direct_write_and_read() {
        let mut memory = Memory::<FieldElement>::default();
        let addr = MemoryAddress::direct(5);

        memory.write(addr, MemoryValue::U32(42));
        assert_eq!(memory.read(addr).to_u128().unwrap(), 42);
    }

    #[test]
    fn relative_write_and_read() {
        let mut memory = Memory::<FieldElement>::default();
        // Stack pointer = 10
        memory.write(MemoryAddress::direct(0), MemoryValue::U32(10));

        let addr = MemoryAddress::Relative(5);
        memory.write(addr, MemoryValue::U32(42));
        assert_eq!(memory.read(addr).to_u128().unwrap(), 42);

        let resolved_addr = memory.resolve(addr);
        // Stack pointer + offset
        // 10 + 5 = 15
        assert_eq!(resolved_addr, 15);
        assert_eq!(memory.values()[resolved_addr].to_u128().unwrap(), 42);
    }

    #[test]
    fn memory_growth() {
        let mut memory = Memory::<FieldElement>::default();
        let addr = MemoryAddress::direct(10);

        memory.write(addr, MemoryValue::U32(123));

        let mut expected = vec![MemoryValue::default(); 10];
        expected.push(MemoryValue::U32(123));

        assert_eq!(memory.values(), &expected);
    }

    #[test]
    fn resize_to_fit_grows_memory() {
        let mut memory = Memory::<FieldElement>::default();
        memory.resize_to_fit(15);

        assert_eq!(memory.values().len(), 15);
        assert!(memory.values().iter().all(|v| *v == MemoryValue::default()));
    }

    #[test]
    fn write_and_read_vector() {
        let mut memory = Memory::<FieldElement>::default();
        // [1, 2, 3, 4, 5]
        let values: Vec<_> = (1..=5).map(MemoryValue::U32).collect();

        // Write at an address > 0 to show resizing
        memory.write_vector(MemoryAddress::direct(2), &values);
        assert_eq!(
            memory
                .read_vector(MemoryAddress::direct(2), 3)
                .iter()
                .map(|v| v.to_u128().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            memory
                .read_vector(MemoryAddress::direct(5), 2)
                .iter()
                .map(|v| v.to_u128().unwrap())
                .collect::<Vec<_>>(),
            vec![4, 5]
        );
        let zero_field = FieldElement::zero();
        assert_eq!(
            memory
                .read_vector(MemoryAddress::direct(0), 2)
                .iter()
                .map(|v| v.to_field())
                .collect::<Vec<_>>(),
            vec![zero_field, zero_field]
        );
        assert_eq!(
            memory
                .read_vector(MemoryAddress::direct(2), 5)
                .iter()
                .map(|v| v.to_u128().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn read_ref_returns_expected_address_and_reads_vector() {
        let mut memory = Memory::<FieldElement>::default();

        // Imagine we have a heap array starting at address 10
        let heap_start = MemoryAddress::direct(10);
        // [1, 2, 3]
        let values: Vec<_> = (1..=3).map(MemoryValue::U32).collect();
        memory.write_vector(heap_start, &values);

        let array_pointer = MemoryAddress::direct(1);
        // Store a pointer to that array at address 1 (after the stack pointer)
        memory.write(array_pointer, MemoryValue::U32(10));

        // `read_ref` should read that pointer and returns MemoryAddress::direct(10)
        let array_start = memory.read_ref(array_pointer);
        assert_eq!(array_start, MemoryAddress::direct(10));

        // Use that reference to read the 3 element array
        let got_vector = memory.read_vector(array_start, 3);
        assert_eq!(got_vector, values);
    }

    #[test]
    fn zero_length_vector() {
        let memory = Memory::<FieldElement>::default();
        assert_eq!(memory.read_vector(MemoryAddress::direct(20), 0), &[]);
    }

    #[test]
    fn read_from_non_existent_memory() {
        let memory = Memory::<FieldElement>::default();
        let result = memory.read(MemoryAddress::direct(20));
        // `Memory::read` returns zero at out of bounds indices
        assert!(result.to_field().is_zero());
    }

    #[test]
    #[should_panic(expected = "range end index 30 out of range for vector of length 0")]
    fn read_vector_from_non_existent_memory() {
        let memory = Memory::<FieldElement>::default();
        let _ = memory.read_vector(MemoryAddress::direct(20), 10);
    }
}
