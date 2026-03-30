//! Implementation of the VM's memory.
//!
//! # Memory Addressing Limits
//!
//! The VM uses u32 addresses, theoretically allowing up to 2^32 memory slots. However,
//! practical limits apply:
//!
//! - **Rust allocator limit**: All allocations are capped at `isize::MAX` bytes. On 32-bit
//!   systems, this limits addressable memory to approximately `i32::MAX / sizeof(MemoryValue)`
//!   elements (~44 million with typical element sizes).
//!
//! - **RAM limit**: On 64-bit systems, the allocator limit is not a concern, but allocating
//!   the full u32 address space would require ~200 GB of RAM.
use acir::{
    AcirField,
    brillig::{BitSize, IntegerBitSize, MemoryAddress},
};

use crate::assert_usize;

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
    pub const ARRAY_META_COUNT: u32 = 1;
    pub const ARRAY_ITEMS: u32 = 1;

    /// Number of prefix fields in a vector: RC, size, capacity.
    pub const VECTOR_META_COUNT: u32 = 3;
    pub const VECTOR_SIZE: u32 = 1;
    pub const VECTOR_CAPACITY: u32 = 2;
    pub const VECTOR_ITEMS: u32 = 3;
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
            IntegerBitSize::U1 => MemoryValue::U1(match value {
                0 => false,
                1 => true,
                _ => panic!("{value} is out of 1 bit range"),
            }),
            IntegerBitSize::U8 => {
                MemoryValue::U8(value.try_into().expect("{value} is out of 8 bits range"))
            }
            IntegerBitSize::U16 => {
                MemoryValue::U16(value.try_into().expect("{value} is out of 16 bits range"))
            }
            IntegerBitSize::U32 => {
                MemoryValue::U32(value.try_into().expect("{value} is out of 32 bits range"))
            }
            IntegerBitSize::U64 => {
                MemoryValue::U64(value.try_into().expect("{value} is out of 64 bits range"))
            }
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
    pub fn to_u32(&self) -> u32 {
        match self {
            MemoryValue::U32(value) => *value,
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
        if let BitSize::Integer(bit_size) = bit_size
            && value.num_bits() > bit_size.into()
        {
            return None;
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
/// Compact internal representation of a memory slot — 16 bytes.
///
/// Large values (Field, U128) are stored in a side table and referenced by a `u32` index.
///
/// `sizeof(Slot) == 16` because the largest inline variant is `u64` (8 bytes,
/// 8-byte alignment) and the discriminant is padded to match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Slot {
    /// Uninitialized — reads as `MemoryValue::Field(F::zero())`.
    #[default]
    Uninit,
    U1(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    /// Index into [`Memory::large`] holding `F::from(u128_value)`.
    U128(u32),
    /// Index into [`Memory::large`] holding the field element.
    Field(u32),
}

/// The VM's memory.
///
/// Uses a single contiguous `Vec<Slot>` (16 bytes/slot).
/// Field and U128 values overflow into a side table.
///
/// One indexed load per read from a single contiguous array, preserving cache behavior.
///
/// # Capacity Limits
///
/// The maximum number of addressable slots is `i32::MAX` to ensure
/// deterministic behavior across 32-bit and 64-bit systems.
#[derive(Debug, Clone)]
pub struct Memory<F> {
    /// One [`Slot`] per memory address — 16 bytes each.
    slots: Vec<Slot>,
    /// Side table for Field and U128 values (32 bytes each for BN254).
    /// U128 is promoted to `F` via `F::from(u128)`.
    large: Vec<F>,
    /// Free list of reusable indices in [`Self::large`].
    large_free: Vec<u32>,
}

impl<F> Default for Memory<F> {
    fn default() -> Self {
        Self { slots: Vec::new(), large: Vec::new(), large_free: Vec::new() }
    }
}

impl<F: PartialEq> PartialEq for Memory<F> {
    fn eq(&self, other: &Self) -> bool {
        if self.slots.len() != other.slots.len() {
            return false;
        }
        for i in 0..self.slots.len() {
            match (self.slots[i], other.slots[i]) {
                (Slot::Uninit, Slot::Uninit) => {}
                (Slot::U1(a), Slot::U1(b)) => {
                    if a != b {
                        return false;
                    }
                }
                (Slot::U8(a), Slot::U8(b)) => {
                    if a != b {
                        return false;
                    }
                }
                (Slot::U16(a), Slot::U16(b)) => {
                    if a != b {
                        return false;
                    }
                }
                (Slot::U32(a), Slot::U32(b)) => {
                    if a != b {
                        return false;
                    }
                }
                (Slot::U64(a), Slot::U64(b)) => {
                    if a != b {
                        return false;
                    }
                }
                (Slot::U128(a), Slot::U128(b)) => {
                    if self.large[a as usize] != other.large[b as usize] {
                        return false;
                    }
                }
                (Slot::Field(a), Slot::Field(b)) => {
                    if self.large[a as usize] != other.large[b as usize] {
                        return false;
                    }
                }
                _ => return false, // different tags
            }
        }
        true
    }
}

impl<F: Eq> Eq for Memory<F> {}

impl<F: AcirField> Memory<F> {
    /// Read the value from slot 0.
    ///
    /// Panics if it's not a `U32`.
    fn get_stack_pointer(&self) -> u32 {
        self.read(STACK_POINTER_ADDRESS).to_u32()
    }

    /// Resolve an address to either:
    /// * itself, if it's a direct address, or
    /// * the current stack pointer plus the offset, if it's relative.
    ///
    /// Returns a memory slot index.
    fn resolve(&self, address: MemoryAddress) -> u32 {
        match address {
            MemoryAddress::Direct(address) => address,
            MemoryAddress::Relative(offset) => {
                self.get_stack_pointer().checked_add(offset).expect("stack pointer offset overflow")
            }
        }
    }

    /// Decode a slot into the public `MemoryValue<F>` representation.
    fn decode_slot(&self, index: usize) -> MemoryValue<F> {
        match self.slots[index] {
            Slot::Uninit => MemoryValue::Field(F::zero()),
            Slot::U1(v) => MemoryValue::U1(v),
            Slot::U8(v) => MemoryValue::U8(v),
            Slot::U16(v) => MemoryValue::U16(v),
            Slot::U32(v) => MemoryValue::U32(v),
            Slot::U64(v) => MemoryValue::U64(v),
            Slot::U128(idx) => MemoryValue::U128(self.large[idx as usize].to_u128()),
            Slot::Field(idx) => MemoryValue::Field(self.large[idx as usize]),
        }
    }

    /// Allocate (or reuse) an entry in the `large` vec and return its index.
    fn alloc_large(&mut self, value: F) -> u32 {
        if let Some(idx) = self.large_free.pop() {
            self.large[idx as usize] = value;
            idx
        } else {
            let idx = self.large.len() as u32;
            self.large.push(value);
            idx
        }
    }

    /// If `slot` holds a large value, push its index onto the free list.
    fn free_large_if_needed(&mut self, slot: Slot) {
        match slot {
            Slot::U128(idx) | Slot::Field(idx) => self.large_free.push(idx),
            _ => {}
        }
    }

    /// Encode a `MemoryValue<F>` and store it at the given slot index.
    fn encode_and_store(&mut self, index: usize, value: MemoryValue<F>) {
        let old = self.slots[index];
        self.slots[index] = match value {
            MemoryValue::U1(v) => {
                self.free_large_if_needed(old);
                Slot::U1(v)
            }
            MemoryValue::U8(v) => {
                self.free_large_if_needed(old);
                Slot::U8(v)
            }
            MemoryValue::U16(v) => {
                self.free_large_if_needed(old);
                Slot::U16(v)
            }
            MemoryValue::U32(v) => {
                self.free_large_if_needed(old);
                Slot::U32(v)
            }
            MemoryValue::U64(v) => {
                self.free_large_if_needed(old);
                Slot::U64(v)
            }
            MemoryValue::U128(v) => {
                if let Slot::U128(idx) = old {
                    // In-place update of existing large entry
                    self.large[idx as usize] = F::from(v);
                    return;
                }
                self.free_large_if_needed(old);
                Slot::U128(self.alloc_large(F::from(v)))
            }
            MemoryValue::Field(f) => {
                if let Slot::Field(idx) = old {
                    self.large[idx as usize] = f;
                    return;
                }
                self.free_large_if_needed(old);
                Slot::Field(self.alloc_large(f))
            }
        };
    }

    /// Reads the numeric value at the address.
    ///
    /// If the address is beyond the size of memory, a default value is returned.
    pub fn read(&self, address: MemoryAddress) -> MemoryValue<F> {
        let resolved_addr = assert_usize(self.resolve(address));
        if resolved_addr >= self.slots.len() {
            return MemoryValue::default();
        }
        self.decode_slot(resolved_addr)
    }

    /// Reads the value at the address and returns it as a direct memory address,
    /// without dereferencing the pointer itself to a numeric value.
    pub fn read_ref(&self, ptr: MemoryAddress) -> MemoryAddress {
        MemoryAddress::direct(self.read(ptr).to_u32())
    }

    /// Sets `ptr` to point at `address`.
    pub fn write_ref(&mut self, ptr: MemoryAddress, address: MemoryAddress) {
        self.write(ptr, MemoryValue::from(address.to_u32()));
    }

    /// Read a contiguous range of memory starting at `address`, up to `len` slots.
    ///
    /// Panics if the end index is beyond the size of the memory.
    pub fn read_slice(&self, address: MemoryAddress, len: usize) -> Vec<MemoryValue<F>> {
        if len == 0 {
            return Vec::new();
        }
        let start = assert_usize(self.resolve(address));
        let end = start + len;
        // Bounds check — panics with standard slice message if out of range
        let _ = &self.slots[start..end];
        (start..end).map(|i| self.decode_slot(i)).collect()
    }

    /// Sets the value at `address` to `value`
    pub fn write(&mut self, address: MemoryAddress, value: MemoryValue<F>) {
        let resolved_addr = assert_usize(self.resolve(address));
        self.resize_to_fit(resolved_addr + 1);
        self.encode_and_store(resolved_addr, value);
    }

    /// Maximum number of memory slots that can be allocated.
    ///
    /// This limit is set to `i32::MAX` to ensure deterministic behavior across all architectures.
    /// On 32-bit systems, Rust's allocator limits allocations to `isize::MAX` bytes, which would
    /// restrict us to fewer elements anyway. By using `i32::MAX`, we ensure the same behavior
    /// on both 32-bit and 64-bit systems.
    ///
    /// See: <https://github.com/rust-lang/rust/pull/95295> and <https://doc.rust-lang.org/1.81.0/src/core/alloc/layout.rs.html>
    const MAX_MEMORY_SIZE: usize = i32::MAX as usize;

    /// Increase the size of memory to fit `size` slots, or the current length, whichever is bigger.
    ///
    /// # Panics
    ///
    /// Panics if `size` exceeds [`Self::MAX_MEMORY_SIZE`].
    fn resize_to_fit(&mut self, size: usize) {
        assert!(
            size <= Self::MAX_MEMORY_SIZE,
            "Memory address space exceeded: requested {size} slots, maximum is {} (i32::MAX)",
            Self::MAX_MEMORY_SIZE
        );
        let new_size = std::cmp::max(self.slots.len(), size);
        self.slots.resize(new_size, Slot::Uninit);
    }

    /// Sets the values starting at `address`.
    pub fn write_slice(&mut self, address: MemoryAddress, values: &[MemoryValue<F>]) {
        let start = assert_usize(self.resolve(address));
        let end = start + values.len();
        self.resize_to_fit(end);
        for (i, value) in values.iter().enumerate() {
            self.encode_and_store(start + i, *value);
        }
    }

    /// Returns all values of the memory as an owned vector.
    pub fn values(&self) -> Vec<MemoryValue<F>> {
        (0..self.slots.len()).map(|i| self.decode_slot(i)).collect()
    }

    /// Returns the number of allocated memory slots.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Returns `true` if no memory slots have been allocated.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acir::FieldElement;
    use test_case::test_case;

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
        assert_eq!(memory.values()[assert_usize(resolved_addr)].to_u128().unwrap(), 42);
    }

    #[test]
    fn memory_growth() {
        let mut memory = Memory::<FieldElement>::default();
        let addr = MemoryAddress::direct(10);

        memory.write(addr, MemoryValue::U32(123));

        let mut expected = vec![MemoryValue::default(); 10];
        expected.push(MemoryValue::U32(123));

        assert_eq!(memory.values(), expected);
    }

    #[test]
    fn resize_to_fit_grows_memory() {
        let mut memory = Memory::<FieldElement>::default();
        memory.resize_to_fit(15);

        assert_eq!(memory.len(), 15);
        assert!(memory.values().iter().all(|v| *v == MemoryValue::default()));
    }

    #[test]
    fn write_and_read_slice() {
        let mut memory = Memory::<FieldElement>::default();
        // [1, 2, 3, 4, 5]
        let values: Vec<_> = (1..=5).map(MemoryValue::U32).collect();

        // Write at an address > 0 to show resizing
        memory.write_slice(MemoryAddress::direct(2), &values);
        assert_eq!(
            memory
                .read_slice(MemoryAddress::direct(2), 3)
                .iter()
                .map(|v| v.to_u128().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            memory
                .read_slice(MemoryAddress::direct(5), 2)
                .iter()
                .map(|v| v.to_u128().unwrap())
                .collect::<Vec<_>>(),
            vec![4, 5]
        );
        let zero_field = FieldElement::zero();
        assert_eq!(
            memory
                .read_slice(MemoryAddress::direct(0), 2)
                .iter()
                .map(|v| v.to_field())
                .collect::<Vec<_>>(),
            vec![zero_field, zero_field]
        );
        assert_eq!(
            memory
                .read_slice(MemoryAddress::direct(2), 5)
                .iter()
                .map(|v| v.to_u128().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn read_ref_returns_expected_address_and_reads_slice() {
        let mut memory = Memory::<FieldElement>::default();

        // Imagine we have a heap array starting at address 10
        let heap_start = MemoryAddress::direct(10);
        // [1, 2, 3]
        let values: Vec<_> = (1..=3).map(MemoryValue::U32).collect();
        memory.write_slice(heap_start, &values);

        let array_pointer = MemoryAddress::direct(1);
        // Store a pointer to that array at address 1 (after the stack pointer)
        memory.write(array_pointer, MemoryValue::U32(10));

        // `read_ref` should read that pointer and returns MemoryAddress::direct(10)
        let array_start = memory.read_ref(array_pointer);
        assert_eq!(array_start, MemoryAddress::direct(10));

        // Use that reference to read the 3 element array
        let got_slice = memory.read_slice(array_start, 3);
        assert_eq!(got_slice, values);
    }

    #[test]
    fn zero_length_slice() {
        let memory = Memory::<FieldElement>::default();
        assert!(memory.read_slice(MemoryAddress::direct(20), 0).is_empty());
    }

    #[test]
    fn read_from_non_existent_memory() {
        let memory = Memory::<FieldElement>::default();
        let result = memory.read(MemoryAddress::direct(20));
        // `Memory::read` returns zero at out of bounds indices
        assert!(result.to_field().is_zero());
    }

    #[test]
    #[should_panic(expected = "range end index 30 out of range for slice of length 0")]
    fn read_vector_from_non_existent_memory() {
        let memory = Memory::<FieldElement>::default();
        let _ = memory.read_slice(MemoryAddress::direct(20), 10);
    }

    #[test]
    #[should_panic(expected = "Memory address space exceeded")]
    fn resize_to_fit_panics_when_exceeding_max_memory_size() {
        let mut memory = Memory::<FieldElement>::default();
        // Attempting to resize beyond i32::MAX should panic
        memory.resize_to_fit(Memory::<FieldElement>::MAX_MEMORY_SIZE + 1);
    }

    #[test_case(IntegerBitSize::U1, 2)]
    #[test_case(IntegerBitSize::U8, 256)]
    #[test_case(IntegerBitSize::U16, u128::from(u16::MAX) + 1)]
    #[test_case(IntegerBitSize::U32, u128::from(u32::MAX) + 1)]
    #[test_case(IntegerBitSize::U64, u128::from(u64::MAX) + 1)]
    #[should_panic(expected = "range")]
    fn memory_value_new_integer_out_of_range(bit_size: IntegerBitSize, value: u128) {
        let _ = MemoryValue::<FieldElement>::new_integer(value, bit_size);
    }

    #[test]
    #[should_panic = "stack pointer offset overflow"]
    fn memory_resolve_overflow() {
        let mut memory = Memory::<FieldElement>::default();
        memory.write(STACK_POINTER_ADDRESS, MemoryValue::from(u32::MAX - 10));
        let addr = MemoryAddress::relative(20);
        let _wrap = memory.resolve(addr);
    }

    #[test]
    fn roundtrip_all_types() {
        let mut memory = Memory::<FieldElement>::default();
        let test_values: Vec<MemoryValue<FieldElement>> = vec![
            MemoryValue::U1(true),
            MemoryValue::U1(false),
            MemoryValue::U8(0),
            MemoryValue::U8(255),
            MemoryValue::U16(12345),
            MemoryValue::U32(0xDEADBEEF),
            MemoryValue::U64(0xCAFEBABE_12345678),
            MemoryValue::U128(u128::MAX),
            MemoryValue::U128(0),
            MemoryValue::Field(FieldElement::from(42u128)),
            MemoryValue::Field(FieldElement::zero()),
        ];

        for (i, value) in test_values.iter().enumerate() {
            let addr = MemoryAddress::direct(i as u32);
            memory.write(addr, *value);
            assert_eq!(memory.read(addr), *value, "roundtrip failed for slot {i}: {value}");
        }
    }

    #[test]
    fn type_change_overwrites() {
        let mut memory = Memory::<FieldElement>::default();
        let addr = MemoryAddress::direct(0);

        // Write Field, then overwrite with U32, then with U128
        memory.write(addr, MemoryValue::Field(FieldElement::from(99u128)));
        assert_eq!(memory.read(addr), MemoryValue::Field(FieldElement::from(99u128)));

        memory.write(addr, MemoryValue::U32(42));
        assert_eq!(memory.read(addr), MemoryValue::U32(42));

        memory.write(addr, MemoryValue::U128(777));
        assert_eq!(memory.read(addr), MemoryValue::U128(777));

        // Overwrite back to a small type
        memory.write(addr, MemoryValue::U8(1));
        assert_eq!(memory.read(addr), MemoryValue::U8(1));
    }

    #[test]
    fn in_place_update_same_type() {
        let mut memory = Memory::<FieldElement>::default();
        let addr = MemoryAddress::direct(0);

        memory.write(addr, MemoryValue::U32(1));
        // With inline storage, overwriting same type just updates inline[0]
        memory.write(addr, MemoryValue::U32(2));
        assert_eq!(memory.read(addr), MemoryValue::U32(2));
    }

    #[test]
    fn large_free_list_reuse() {
        let mut memory = Memory::<FieldElement>::default();

        // Write a field value
        memory.write(MemoryAddress::direct(0), MemoryValue::Field(FieldElement::from(1u128)));
        assert_eq!(memory.large.len(), 1);

        // Overwrite with integer — large entry should be freed
        memory.write(MemoryAddress::direct(0), MemoryValue::U32(42));
        assert_eq!(memory.large_free.len(), 1);

        // Write another field value — should reuse the freed entry
        memory.write(MemoryAddress::direct(1), MemoryValue::Field(FieldElement::from(2u128)));
        assert_eq!(memory.large.len(), 1); // no growth
        assert_eq!(memory.large_free.len(), 0); // reused
    }
}
