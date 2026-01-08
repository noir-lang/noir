use crate::{
    black_box::BlackBoxOp,
    lengths::{SemanticLength, SemiFlattenedLength},
};
use acir_field::AcirField;
use serde::{Deserialize, Serialize};

/// Represents a program location (instruction index) used as a jump target.
pub type Label = usize;

/// Represents an address in the VM's memory.
/// Supports both direct and relative addressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum MemoryAddress {
    /// Specifies an exact index in the VM's memory.
    Direct(usize),
    /// Specifies an index relative to the stack pointer.
    ///
    /// It is resolved as the current stack pointer plus the offset stored here.
    ///
    /// The stack pointer is stored in memory slot 0, so this address is resolved
    /// by reading that slot and adding the offset to get the final memory address.
    Relative(usize),
}

impl MemoryAddress {
    /// Create a `Direct` address.
    pub fn direct(address: usize) -> Self {
        MemoryAddress::Direct(address)
    }

    /// Create a `Relative` address.
    pub fn relative(offset: usize) -> Self {
        MemoryAddress::Relative(offset)
    }

    /// Return the index in a `Direct` address.
    ///
    /// Panics if it's `Relative`.
    pub fn unwrap_direct(self) -> usize {
        match self {
            MemoryAddress::Direct(address) => address,
            MemoryAddress::Relative(_) => panic!("Expected direct memory address"),
        }
    }

    /// Return the index in a `Relative` address.
    ///
    /// Panics if it's `Direct`.
    pub fn unwrap_relative(self) -> usize {
        match self {
            MemoryAddress::Direct(_) => panic!("Expected relative memory address"),
            MemoryAddress::Relative(offset) => offset,
        }
    }

    /// Return the index in the address.
    pub fn to_usize(self) -> usize {
        match self {
            MemoryAddress::Direct(address) => address,
            MemoryAddress::Relative(offset) => offset,
        }
    }

    pub fn is_relative(&self) -> bool {
        match self {
            MemoryAddress::Relative(_) => true,
            MemoryAddress::Direct(_) => false,
        }
    }

    pub fn is_direct(&self) -> bool {
        !self.is_relative()
    }

    /// Offset a `Direct` address by `amount`.
    ///
    /// Panics if called on a `Relative` address.
    pub fn offset(&self, amount: usize) -> Self {
        // We disallow offsetting relatively addresses as this is not expected to be meaningful.
        let address = self.unwrap_direct();
        MemoryAddress::Direct(address + amount)
    }
}

impl std::fmt::Display for MemoryAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryAddress::Direct(address) => write!(f, "@{address}"),
            MemoryAddress::Relative(offset) => write!(f, "sp[{offset}]"),
        }
    }
}

/// Describes the memory layout for an array/vector element
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum HeapValueType {
    /// A single field element is enough to represent the value with a given bit size.
    Simple(BitSize),
    /// The value read should be interpreted as a pointer to a [HeapArray], which
    /// consists of a pointer to a slice of memory of size elements, and a
    /// reference count, to avoid cloning arrays that are not shared.
    Array { value_types: Vec<HeapValueType>, size: SemanticLength },
    /// The value read should be interpreted as a pointer to a [HeapVector], which
    /// consists of a pointer to a slice of memory, a number of elements in that
    /// vector, and a reference count.
    Vector { value_types: Vec<HeapValueType> },
}

impl HeapValueType {
    /// Check that all types are `Simple`.
    pub fn all_simple(types: &[HeapValueType]) -> bool {
        types.iter().all(|typ| matches!(typ, HeapValueType::Simple(_)))
    }

    /// Create a `Simple` type to represent a `Field`.
    pub fn field() -> HeapValueType {
        HeapValueType::Simple(BitSize::Field)
    }

    /// Returns the total number of field elements required to represent this type in memory.
    ///
    /// Returns `None` for `Vector`, as their size is not statically known.
    pub fn flattened_size(&self) -> Option<usize> {
        match self {
            HeapValueType::Simple(_) => Some(1),
            HeapValueType::Array { value_types, size } => {
                // TODO(lengths): use FlattenedLength here
                let element_size =
                    value_types.iter().map(|t| t.flattened_size()).sum::<Option<usize>>();

                // Multiply element size by number of elements.
                element_size.map(|element_size| element_size * size.0)
            }
            HeapValueType::Vector { .. } => {
                // Vectors are dynamic, so we cannot determine their size statically.
                None
            }
        }
    }
}

impl std::fmt::Display for HeapValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let write_types =
            |f: &mut std::fmt::Formatter<'_>, value_types: &[HeapValueType]| -> std::fmt::Result {
                if value_types.len() == 1 {
                    write!(f, "{}", value_types[0])?;
                } else {
                    write!(f, "(")?;
                    for (index, value_type) in value_types.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{value_type}")?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            };

        match self {
            HeapValueType::Simple(bit_size) => {
                write!(f, "{bit_size}")
            }
            HeapValueType::Array { value_types, size } => {
                write!(f, "[")?;
                write_types(f, value_types)?;
                write!(f, "; {size}")?;
                write!(f, "]")
            }
            HeapValueType::Vector { value_types } => {
                write!(f, "@[")?;
                write_types(f, value_types)?;
                write!(f, "]")
            }
        }
    }
}

/// A fixed-sized array starting from a Brillig memory location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct HeapArray {
    /// Pointer to a memory address which hold the address to the start of the items in the array.
    ///
    /// That is to say, the address retrieved from the pointer doesn't need any more offsetting.
    pub pointer: MemoryAddress,
    /// Statically known size of the array.
    pub size: SemiFlattenedLength,
}

impl Default for HeapArray {
    fn default() -> Self {
        Self { pointer: MemoryAddress::direct(0), size: SemiFlattenedLength(0) }
    }
}

impl std::fmt::Display for HeapArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}; {}]", self.pointer, self.size)
    }
}

/// A memory-sized vector passed starting from a Brillig memory location and with a memory-held size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub struct HeapVector {
    /// Pointer to a memory address which hold the address to the start of the items in the vector.
    ///
    /// That is to say, the address retrieved from the pointer doesn't need any more offsetting.
    pub pointer: MemoryAddress,
    /// Address to a memory slot holding the semantic length of the vector.
    pub size: MemoryAddress,
}

impl std::fmt::Display for HeapVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@[{}; {}]", self.pointer, self.size)
    }
}

/// Represents the bit size of unsigned integer types in Brillig.
///
/// These correspond to the standard unsigned integer types, with U1 representing a boolean.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum IntegerBitSize {
    U1,
    U8,
    U16,
    U32,
    U64,
    U128,
}

impl From<IntegerBitSize> for u32 {
    fn from(bit_size: IntegerBitSize) -> u32 {
        match bit_size {
            IntegerBitSize::U1 => 1,
            IntegerBitSize::U8 => 8,
            IntegerBitSize::U16 => 16,
            IntegerBitSize::U32 => 32,
            IntegerBitSize::U64 => 64,
            IntegerBitSize::U128 => 128,
        }
    }
}

impl TryFrom<u32> for IntegerBitSize {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(IntegerBitSize::U1),
            8 => Ok(IntegerBitSize::U8),
            16 => Ok(IntegerBitSize::U16),
            32 => Ok(IntegerBitSize::U32),
            64 => Ok(IntegerBitSize::U64),
            128 => Ok(IntegerBitSize::U128),
            _ => Err("Invalid bit size"),
        }
    }
}

impl std::fmt::Display for IntegerBitSize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IntegerBitSize::U1 => write!(f, "bool"),
            IntegerBitSize::U8 => write!(f, "u8"),
            IntegerBitSize::U16 => write!(f, "u16"),
            IntegerBitSize::U32 => write!(f, "u32"),
            IntegerBitSize::U64 => write!(f, "u64"),
            IntegerBitSize::U128 => write!(f, "u128"),
        }
    }
}

/// Represents the bit size of values in Brillig.
///
/// Values can either be field elements (whose size depends on the field being used)
/// or fixed-size unsigned integers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum BitSize {
    Field,
    Integer(IntegerBitSize),
}

impl BitSize {
    /// Convert the bit size to a u32 value.
    ///
    /// For field elements, returns the maximum number of bits in the field.
    /// For integers, returns the bit size of the integer type.
    pub fn to_u32<F: AcirField>(self) -> u32 {
        match self {
            BitSize::Field => F::max_num_bits(),
            BitSize::Integer(bit_size) => bit_size.into(),
        }
    }

    /// Try to create a BitSize from a u32 value.
    ///
    /// If the value matches the field's maximum bit count, returns `BitSize::Field`.
    /// Otherwise, attempts to interpret it as an integer bit size.
    pub fn try_from_u32<F: AcirField>(value: u32) -> Result<Self, &'static str> {
        if value == F::max_num_bits() {
            Ok(BitSize::Field)
        } else {
            Ok(BitSize::Integer(IntegerBitSize::try_from(value)?))
        }
    }
}

impl std::fmt::Display for BitSize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BitSize::Field => write!(f, "field"),
            BitSize::Integer(bit_size) => write!(f, "{bit_size}"),
        }
    }
}

/// Lays out various ways an external foreign call's input and output data may be interpreted inside Brillig.
/// This data can either be an individual value or memory.
///
/// While we are usually agnostic to how memory is passed within Brillig,
/// this needs to be encoded somehow when dealing with an external system.
/// For simplicity, the extra type information is given right in the `ForeignCall` instructions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum ValueOrArray {
    /// A single value to be passed to or from an external call.
    /// It is an 'immediate' value - used without dereferencing.
    /// For a foreign call input, the value is read directly from memory.
    /// For a foreign call output, the value is written directly to memory.
    MemoryAddress(MemoryAddress),
    /// An array to be passed to or from an external call.
    /// In the case of a foreign call input, the array is read from this Brillig memory location + `size` more cells.
    /// In the case of a foreign call output, the array is written to this Brillig memory location with the `size` being here just as a sanity check for the write size.
    HeapArray(HeapArray),
    /// A vector to be passed to or from an external call.
    /// In the case of a foreign call input, the vector is read from this Brillig memory location + as many cells as the second address indicates.
    /// In the case of a foreign call output, the vector is written to this Brillig memory location as 'size' cells, with size being stored in the second address.
    HeapVector(HeapVector),
}

impl std::fmt::Display for ValueOrArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueOrArray::MemoryAddress(memory_address) => {
                write!(f, "{memory_address}")
            }
            ValueOrArray::HeapArray(heap_array) => {
                write!(f, "{heap_array}")
            }
            ValueOrArray::HeapVector(heap_vector) => {
                write!(f, "{heap_vector}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum BrilligOpcode<F> {
    /// Takes the fields in addresses `lhs` and `rhs`,
    /// performs the specified binary operation,
    /// and stores the value in the `destination` address.
    BinaryFieldOp {
        destination: MemoryAddress,
        op: BinaryFieldOp,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
    },
    /// Takes the `bit_size` size integers in addresses `lhs` and `rhs`,
    /// performs the specified binary operation,
    /// and stores the value in the `destination` address.
    BinaryIntOp {
        destination: MemoryAddress,
        op: BinaryIntOp,
        bit_size: IntegerBitSize,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
    },
    /// Takes the value from the `source` address, inverts it,
    /// and stores the value in the `destination` address.
    Not { destination: MemoryAddress, source: MemoryAddress, bit_size: IntegerBitSize },
    /// Takes the value from the `source` address,
    /// casts it into the type indicated by `bit_size`,
    /// and stores the value in the `destination` address.
    Cast { destination: MemoryAddress, source: MemoryAddress, bit_size: BitSize },
    /// Sets the program counter to the value of `location`
    /// if the value at `condition` is non-zero.
    JumpIf { condition: MemoryAddress, location: Label },
    /// Sets the program counter to the value of `location`.
    Jump { location: Label },
    /// Copies calldata after the `offset_address` with length indicated by `size_address`
    /// to the specified `destination_address`.
    CalldataCopy {
        destination_address: MemoryAddress,
        size_address: MemoryAddress,
        offset_address: MemoryAddress,
    },
    /// Pushes the current program counter to the call stack as to set a return location.
    /// Sets the program counter to the value of `location`.
    ///
    /// We don't support dynamic jumps or calls;
    /// see <https://github.com/ethereum/aleth/issues/3404> for reasoning.
    Call { location: Label },
    /// Stores a constant `value` with a `bit_size` in the `destination` address.
    Const { destination: MemoryAddress, bit_size: BitSize, value: F },
    /// Reads the address from `destination_pointer`, then stores a constant `value` with a `bit_size` at that address.
    IndirectConst { destination_pointer: MemoryAddress, bit_size: BitSize, value: F },
    /// Pops the top element from the call stack, which represents the return location,
    /// and sets the program counter to that value. This operation is used to return
    /// from a function call.
    Return,
    /// Used to get data from an outside source.
    ///
    /// Also referred to as an Oracle, intended for things like state tree reads;
    /// it shouldn't be confused with e.g. blockchain price oracles.
    ForeignCall {
        /// Interpreted by caller context, ie. this will have different meanings depending on
        /// who the caller is.
        function: String,
        /// Destination addresses (may be single values or memory pointers).
        ///
        /// Output vectors are passed as a [ValueOrArray::MemoryAddress]. Since their size is not known up front,
        /// we cannot allocate space for them on the heap. Instead, the VM is expected to write their data after
        /// the current free memory pointer, and store the heap address into the destination.
        destinations: Vec<ValueOrArray>,
        /// Destination value types.
        destination_value_types: Vec<HeapValueType>,
        /// Input addresses (may be single values or memory pointers).
        inputs: Vec<ValueOrArray>,
        /// Input value types (for heap allocated structures indicates how to
        /// retrieve the elements).
        input_value_types: Vec<HeapValueType>,
    },
    /// Moves the content in the `source` address to the `destination` address.
    Mov { destination: MemoryAddress, source: MemoryAddress },
    /// If the value at `condition` is non-zero, moves the content in the `source_a`
    /// address to the `destination` address, otherwise moves the content from the
    /// `source_b` address instead.
    ///
    /// `destination = condition > 0 ? source_a : source_b`
    ConditionalMov {
        destination: MemoryAddress,
        source_a: MemoryAddress,
        source_b: MemoryAddress,
        condition: MemoryAddress,
    },
    /// Reads the `source_pointer` to obtain a memory address, then retrieves the data
    /// stored at that address and writes it to the `destination` address.
    Load { destination: MemoryAddress, source_pointer: MemoryAddress },
    /// Reads the `destination_pointer` to obtain a memory address, then stores the value
    /// from the `source` address at that location.
    Store { destination_pointer: MemoryAddress, source: MemoryAddress },
    /// Native functions in the VM.
    /// These are equivalent to the black box functions in ACIR.
    BlackBox(BlackBoxOp),
    /// Used to denote execution failure, halting the VM and returning data specified by a dynamically-sized vector.
    Trap { revert_data: HeapVector },
    /// Halts execution and returns data specified by a dynamically-sized vector.
    Stop { return_data: HeapVector },
}

impl<F: std::fmt::Display> std::fmt::Display for BrilligOpcode<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrilligOpcode::BinaryFieldOp { destination, op, lhs, rhs } => {
                write!(f, "{destination} = field {op} {lhs}, {rhs}")
            }
            BrilligOpcode::BinaryIntOp { destination, op, bit_size, lhs, rhs } => {
                write!(f, "{destination} = {bit_size} {op} {lhs}, {rhs}")
            }
            BrilligOpcode::Not { destination, source, bit_size } => {
                write!(f, "{destination} = {bit_size} not {source}")
            }
            BrilligOpcode::Cast { destination, source, bit_size } => {
                write!(f, "{destination} = cast {source} to {bit_size}")
            }
            BrilligOpcode::JumpIf { condition, location } => {
                write!(f, "jump if {condition} to {location}")
            }
            BrilligOpcode::Jump { location } => {
                write!(f, "jump to {location}")
            }
            BrilligOpcode::CalldataCopy { destination_address, size_address, offset_address } => {
                write!(
                    f,
                    "{destination_address} = calldata copy [{offset_address}; {size_address}]"
                )
            }
            BrilligOpcode::Call { location } => {
                write!(f, "call {location}")
            }
            BrilligOpcode::Const { destination, bit_size, value } => {
                write!(f, "{destination} = const {bit_size} {value}")
            }
            BrilligOpcode::IndirectConst { destination_pointer, bit_size, value } => {
                write!(f, "{destination_pointer} = indirect const {bit_size} {value}")
            }
            BrilligOpcode::Return => {
                write!(f, "return")
            }
            BrilligOpcode::ForeignCall {
                function,
                destinations,
                destination_value_types,
                inputs,
                input_value_types,
            } => {
                assert_eq!(destinations.len(), destination_value_types.len());

                if !destinations.is_empty() {
                    for (index, (destination, destination_value_type)) in
                        destinations.iter().zip(destination_value_types).enumerate()
                    {
                        if index > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{destination}: {destination_value_type}")?;
                    }
                    write!(f, " = ")?;
                }

                write!(f, "foreign call {function}(")?;

                assert_eq!(inputs.len(), input_value_types.len());
                for (index, (input, input_value_type)) in
                    inputs.iter().zip(input_value_types).enumerate()
                {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{input}: {input_value_type}")?;
                }

                write!(f, ")")?;
                Ok(())
            }
            BrilligOpcode::Mov { destination, source } => {
                write!(f, "{destination} = {source}")
            }
            BrilligOpcode::ConditionalMov { destination, source_a, source_b, condition } => {
                write!(f, "{destination} = if {condition} then {source_a} else {source_b}")
            }
            BrilligOpcode::Load { destination, source_pointer } => {
                write!(f, "{destination} = load {source_pointer}")
            }
            BrilligOpcode::Store { destination_pointer, source } => {
                write!(f, "store {source} at {destination_pointer}")
            }
            BrilligOpcode::BlackBox(black_box_op) => {
                write!(f, "{black_box_op}")
            }
            BrilligOpcode::Trap { revert_data } => {
                write!(f, "trap {revert_data}")
            }
            BrilligOpcode::Stop { return_data } => {
                write!(f, "stop {return_data}")
            }
        }
    }
}

/// Binary operations on field elements.
///
/// Most operations work with field arithmetic, but some operations like
/// `IntegerDiv` interpret the field elements as unsigned integers for the purpose
/// of the operation (useful when field elements are used to represent integer values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum BinaryFieldOp {
    Add,
    Sub,
    Mul,
    /// Field division (inverse multiplication in the field)
    Div,
    /// Unsigned integer division (treating field elements as unsigned integers)
    IntegerDiv,
    /// (==) Equal
    Equals,
    /// (<) Field less than
    LessThan,
    /// (<=) Field less or equal
    LessThanEquals,
}

impl std::fmt::Display for BinaryFieldOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFieldOp::Add => write!(f, "add"),
            BinaryFieldOp::Sub => write!(f, "sub"),
            BinaryFieldOp::Mul => write!(f, "mul"),
            BinaryFieldOp::Div => write!(f, "field_div"),
            BinaryFieldOp::IntegerDiv => write!(f, "int_div"),
            BinaryFieldOp::Equals => write!(f, "eq"),
            BinaryFieldOp::LessThan => write!(f, "lt"),
            BinaryFieldOp::LessThanEquals => write!(f, "lt_eq"),
        }
    }
}

/// Binary fixed-length integer expressions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(feature = "arb", derive(proptest_derive::Arbitrary))]
pub enum BinaryIntOp {
    Add,
    Sub,
    Mul,
    Div,
    /// (==) Equal
    Equals,
    /// (<) Integer less than
    LessThan,
    /// (<=) Integer less or equal
    LessThanEquals,
    /// (&) Bitwise AND
    And,
    /// (|) Bitwise OR
    Or,
    /// (^) Bitwise XOR
    Xor,
    /// (<<) Shift left
    Shl,
    /// (>>) Shift right
    Shr,
}

impl std::fmt::Display for BinaryIntOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryIntOp::Add => write!(f, "add"),
            BinaryIntOp::Sub => write!(f, "sub"),
            BinaryIntOp::Mul => write!(f, "mul"),
            BinaryIntOp::Div => write!(f, "div"),
            BinaryIntOp::Equals => write!(f, "eq"),
            BinaryIntOp::LessThan => write!(f, "lt"),
            BinaryIntOp::LessThanEquals => write!(f, "lt_eq"),
            BinaryIntOp::And => write!(f, "and"),
            BinaryIntOp::Or => write!(f, "or"),
            BinaryIntOp::Xor => write!(f, "xor"),
            BinaryIntOp::Shl => write!(f, "shl"),
            BinaryIntOp::Shr => write!(f, "shr"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BitSize, IntegerBitSize};
    use acir_field::FieldElement;

    /// Test that IntegerBitSize round trips correctly through From/TryFrom u32
    #[test]
    fn test_integer_bitsize_roundtrip() {
        let integer_sizes = [
            IntegerBitSize::U1,
            IntegerBitSize::U8,
            IntegerBitSize::U16,
            IntegerBitSize::U32,
            IntegerBitSize::U64,
            IntegerBitSize::U128,
        ];

        for int_size in integer_sizes {
            // Convert to u32 using From trait
            let as_u32: u32 = int_size.into();
            // Convert back using TryFrom trait
            let roundtrip = IntegerBitSize::try_from(as_u32)
                .expect("Should successfully convert back from u32");
            assert_eq!(
                int_size, roundtrip,
                "IntegerBitSize::{int_size} should roundtrip through From<IntegerBitSize> for u32 and TryFrom<u32>"
            );
        }
    }

    #[test]
    fn test_integer_bitsize_values() {
        // Verify the actual u32 values returned by From trait
        assert_eq!(u32::from(IntegerBitSize::U1), 1);
        assert_eq!(u32::from(IntegerBitSize::U8), 8);
        assert_eq!(u32::from(IntegerBitSize::U16), 16);
        assert_eq!(u32::from(IntegerBitSize::U32), 32);
        assert_eq!(u32::from(IntegerBitSize::U64), 64);
        assert_eq!(u32::from(IntegerBitSize::U128), 128);
    }

    #[test]
    fn test_integer_bitsize_try_from_invalid() {
        // Test that invalid bit sizes return an error
        assert!(IntegerBitSize::try_from(0).is_err());
        assert!(IntegerBitSize::try_from(2).is_err());
        assert!(IntegerBitSize::try_from(7).is_err());
        assert!(IntegerBitSize::try_from(15).is_err());
        assert!(IntegerBitSize::try_from(31).is_err());
        assert!(IntegerBitSize::try_from(63).is_err());
        assert!(IntegerBitSize::try_from(127).is_err());
        assert!(IntegerBitSize::try_from(129).is_err());
        assert!(IntegerBitSize::try_from(256).is_err());
    }

    /// Test that BitSize roundtrips correctly through to_u32/try_from_u32
    #[test]
    fn test_bitsize_roundtrip() {
        // Test all integer bit sizes
        let integer_sizes = [
            IntegerBitSize::U1,
            IntegerBitSize::U8,
            IntegerBitSize::U16,
            IntegerBitSize::U32,
            IntegerBitSize::U64,
            IntegerBitSize::U128,
        ];

        for int_size in integer_sizes {
            let bit_size = BitSize::Integer(int_size);
            let as_u32 = bit_size.to_u32::<FieldElement>();
            let roundtrip = BitSize::try_from_u32::<FieldElement>(as_u32)
                .expect("Should successfully convert back from u32");
            assert_eq!(
                bit_size, roundtrip,
                "BitSize::Integer({int_size}) should roundtrip through to_u32/try_from_u32"
            );
        }

        // Test Field type
        let field_bit_size = BitSize::Field;
        let as_u32 = field_bit_size.to_u32::<FieldElement>();
        let roundtrip = BitSize::try_from_u32::<FieldElement>(as_u32)
            .expect("Should successfully convert Field back from u32");
        assert_eq!(
            field_bit_size, roundtrip,
            "BitSize::Field should roundtrip through to_u32/try_from_u32"
        );
    }

    #[test]
    fn test_bitsize_to_u32_values_integers() {
        // Verify the actual u32 values returned for integer types
        assert_eq!(BitSize::Integer(IntegerBitSize::U1).to_u32::<FieldElement>(), 1);
        assert_eq!(BitSize::Integer(IntegerBitSize::U8).to_u32::<FieldElement>(), 8);
        assert_eq!(BitSize::Integer(IntegerBitSize::U16).to_u32::<FieldElement>(), 16);
        assert_eq!(BitSize::Integer(IntegerBitSize::U32).to_u32::<FieldElement>(), 32);
        assert_eq!(BitSize::Integer(IntegerBitSize::U64).to_u32::<FieldElement>(), 64);
        assert_eq!(BitSize::Integer(IntegerBitSize::U128).to_u32::<FieldElement>(), 128);
    }

    #[test]
    #[cfg(feature = "bn254")]
    fn test_bitsize_to_u32_field_bn254() {
        // Field type returns 254 bits for bn254
        assert_eq!(BitSize::Field.to_u32::<FieldElement>(), 254);
    }

    #[test]
    #[cfg(feature = "bls12_381")]
    fn test_bitsize_to_u32_field_bls12_381() {
        // Field type returns 255 bits for bls12_381
        assert_eq!(BitSize::Field.to_u32::<FieldElement>(), 255);
    }

    #[test]
    fn test_bitsize_try_from_u32_invalid() {
        // Test that invalid bit sizes return an error
        assert!(BitSize::try_from_u32::<FieldElement>(2).is_err());
        assert!(BitSize::try_from_u32::<FieldElement>(7).is_err());
        assert!(BitSize::try_from_u32::<FieldElement>(0).is_err());
        assert!(BitSize::try_from_u32::<FieldElement>(256).is_err());
    }
}

#[cfg(feature = "arb")]
mod prop_tests {
    use proptest::arbitrary::Arbitrary;
    use proptest::prelude::*;

    use crate::lengths::SemanticLength;

    use super::{BitSize, HeapValueType};

    // Need to define recursive strategy for `HeapValueType`
    impl Arbitrary for HeapValueType {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let leaf = any::<BitSize>().prop_map(HeapValueType::Simple);
            leaf.prop_recursive(2, 3, 2, |inner| {
                prop_oneof![
                    (prop::collection::vec(inner.clone(), 1..3), any::<usize>()).prop_map(
                        |(value_types, size)| {
                            HeapValueType::Array { value_types, size: SemanticLength(size) }
                        }
                    ),
                    (prop::collection::vec(inner.clone(), 1..3))
                        .prop_map(|value_types| { HeapValueType::Vector { value_types } }),
                ]
            })
            .boxed()
        }
    }
}
