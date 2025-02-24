#pragma once

#include "serde.hpp"
#include "bincode.hpp"

namespace Program {

    struct BinaryFieldOp {

        struct Add {
            friend bool operator==(const Add&, const Add&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Add bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Sub {
            friend bool operator==(const Sub&, const Sub&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sub bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Mul {
            friend bool operator==(const Mul&, const Mul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Mul bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Div {
            friend bool operator==(const Div&, const Div&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Div bincodeDeserialize(std::vector<uint8_t>);
        };

        struct IntegerDiv {
            friend bool operator==(const IntegerDiv&, const IntegerDiv&);
            std::vector<uint8_t> bincodeSerialize() const;
            static IntegerDiv bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Equals {
            friend bool operator==(const Equals&, const Equals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Equals bincodeDeserialize(std::vector<uint8_t>);
        };

        struct LessThan {
            friend bool operator==(const LessThan&, const LessThan&);
            std::vector<uint8_t> bincodeSerialize() const;
            static LessThan bincodeDeserialize(std::vector<uint8_t>);
        };

        struct LessThanEquals {
            friend bool operator==(const LessThanEquals&, const LessThanEquals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static LessThanEquals bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Add, Sub, Mul, Div, IntegerDiv, Equals, LessThan, LessThanEquals> value;

        friend bool operator==(const BinaryFieldOp&, const BinaryFieldOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BinaryFieldOp bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BinaryIntOp {

        struct Add {
            friend bool operator==(const Add&, const Add&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Add bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Sub {
            friend bool operator==(const Sub&, const Sub&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sub bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Mul {
            friend bool operator==(const Mul&, const Mul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Mul bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Div {
            friend bool operator==(const Div&, const Div&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Div bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Equals {
            friend bool operator==(const Equals&, const Equals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Equals bincodeDeserialize(std::vector<uint8_t>);
        };

        struct LessThan {
            friend bool operator==(const LessThan&, const LessThan&);
            std::vector<uint8_t> bincodeSerialize() const;
            static LessThan bincodeDeserialize(std::vector<uint8_t>);
        };

        struct LessThanEquals {
            friend bool operator==(const LessThanEquals&, const LessThanEquals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static LessThanEquals bincodeDeserialize(std::vector<uint8_t>);
        };

        struct And {
            friend bool operator==(const And&, const And&);
            std::vector<uint8_t> bincodeSerialize() const;
            static And bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Or {
            friend bool operator==(const Or&, const Or&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Or bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Xor {
            friend bool operator==(const Xor&, const Xor&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Xor bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Shl {
            friend bool operator==(const Shl&, const Shl&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Shl bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Shr {
            friend bool operator==(const Shr&, const Shr&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Shr bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Add, Sub, Mul, Div, Equals, LessThan, LessThanEquals, And, Or, Xor, Shl, Shr> value;

        friend bool operator==(const BinaryIntOp&, const BinaryIntOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BinaryIntOp bincodeDeserialize(std::vector<uint8_t>);
    };

    struct IntegerBitSize {

        struct U1 {
            friend bool operator==(const U1&, const U1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct U8 {
            friend bool operator==(const U8&, const U8&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U8 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct U16 {
            friend bool operator==(const U16&, const U16&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U16 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct U32 {
            friend bool operator==(const U32&, const U32&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U32 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct U64 {
            friend bool operator==(const U64&, const U64&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U64 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct U128 {
            friend bool operator==(const U128&, const U128&);
            std::vector<uint8_t> bincodeSerialize() const;
            static U128 bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<U1, U8, U16, U32, U64, U128> value;

        friend bool operator==(const IntegerBitSize&, const IntegerBitSize&);
        std::vector<uint8_t> bincodeSerialize() const;
        static IntegerBitSize bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BitSize {

        struct Field {
            friend bool operator==(const Field&, const Field&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Field bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Integer {
            Program::IntegerBitSize value;

            friend bool operator==(const Integer&, const Integer&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Integer bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Field, Integer> value;

        friend bool operator==(const BitSize&, const BitSize&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BitSize bincodeDeserialize(std::vector<uint8_t>);
    };

    struct MemoryAddress {

        struct Direct {
            uint64_t value;

            friend bool operator==(const Direct&, const Direct&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Direct bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Relative {
            uint64_t value;

            friend bool operator==(const Relative&, const Relative&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Relative bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Direct, Relative> value;

        friend bool operator==(const MemoryAddress&, const MemoryAddress&);
        std::vector<uint8_t> bincodeSerialize() const;
        static MemoryAddress bincodeDeserialize(std::vector<uint8_t>);
    };

    struct HeapArray {
        Program::MemoryAddress pointer;
        uint64_t size;

        friend bool operator==(const HeapArray&, const HeapArray&);
        std::vector<uint8_t> bincodeSerialize() const;
        static HeapArray bincodeDeserialize(std::vector<uint8_t>);
    };

    struct HeapVector {
        Program::MemoryAddress pointer;
        Program::MemoryAddress size;

        friend bool operator==(const HeapVector&, const HeapVector&);
        std::vector<uint8_t> bincodeSerialize() const;
        static HeapVector bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BlackBoxOp {

        struct AES128Encrypt {
            Program::HeapVector inputs;
            Program::HeapArray iv;
            Program::HeapArray key;
            Program::HeapVector outputs;

            friend bool operator==(const AES128Encrypt&, const AES128Encrypt&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AES128Encrypt bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Blake2s {
            Program::HeapVector message;
            Program::HeapArray output;

            friend bool operator==(const Blake2s&, const Blake2s&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake2s bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Blake3 {
            Program::HeapVector message;
            Program::HeapArray output;

            friend bool operator==(const Blake3&, const Blake3&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake3 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Keccakf1600 {
            Program::HeapArray input;
            Program::HeapArray output;

            friend bool operator==(const Keccakf1600&, const Keccakf1600&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccakf1600 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EcdsaSecp256k1 {
            Program::HeapVector hashed_msg;
            Program::HeapArray public_key_x;
            Program::HeapArray public_key_y;
            Program::HeapArray signature;
            Program::MemoryAddress result;

            friend bool operator==(const EcdsaSecp256k1&, const EcdsaSecp256k1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256k1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EcdsaSecp256r1 {
            Program::HeapVector hashed_msg;
            Program::HeapArray public_key_x;
            Program::HeapArray public_key_y;
            Program::HeapArray signature;
            Program::MemoryAddress result;

            friend bool operator==(const EcdsaSecp256r1&, const EcdsaSecp256r1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256r1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct MultiScalarMul {
            Program::HeapVector points;
            Program::HeapVector scalars;
            Program::HeapArray outputs;

            friend bool operator==(const MultiScalarMul&, const MultiScalarMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MultiScalarMul bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EmbeddedCurveAdd {
            Program::MemoryAddress input1_x;
            Program::MemoryAddress input1_y;
            Program::MemoryAddress input1_infinite;
            Program::MemoryAddress input2_x;
            Program::MemoryAddress input2_y;
            Program::MemoryAddress input2_infinite;
            Program::HeapArray result;

            friend bool operator==(const EmbeddedCurveAdd&, const EmbeddedCurveAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EmbeddedCurveAdd bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntAdd {
            Program::MemoryAddress lhs;
            Program::MemoryAddress rhs;
            Program::MemoryAddress output;

            friend bool operator==(const BigIntAdd&, const BigIntAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntAdd bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntSub {
            Program::MemoryAddress lhs;
            Program::MemoryAddress rhs;
            Program::MemoryAddress output;

            friend bool operator==(const BigIntSub&, const BigIntSub&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntSub bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntMul {
            Program::MemoryAddress lhs;
            Program::MemoryAddress rhs;
            Program::MemoryAddress output;

            friend bool operator==(const BigIntMul&, const BigIntMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntMul bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntDiv {
            Program::MemoryAddress lhs;
            Program::MemoryAddress rhs;
            Program::MemoryAddress output;

            friend bool operator==(const BigIntDiv&, const BigIntDiv&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntDiv bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntFromLeBytes {
            Program::HeapVector inputs;
            Program::HeapVector modulus;
            Program::MemoryAddress output;

            friend bool operator==(const BigIntFromLeBytes&, const BigIntFromLeBytes&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntFromLeBytes bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntToLeBytes {
            Program::MemoryAddress input;
            Program::HeapVector output;

            friend bool operator==(const BigIntToLeBytes&, const BigIntToLeBytes&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntToLeBytes bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Poseidon2Permutation {
            Program::HeapVector message;
            Program::HeapArray output;
            Program::MemoryAddress len;

            friend bool operator==(const Poseidon2Permutation&, const Poseidon2Permutation&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Poseidon2Permutation bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Sha256Compression {
            Program::HeapArray input;
            Program::HeapArray hash_values;
            Program::HeapArray output;

            friend bool operator==(const Sha256Compression&, const Sha256Compression&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sha256Compression bincodeDeserialize(std::vector<uint8_t>);
        };

        struct ToRadix {
            Program::MemoryAddress input;
            Program::MemoryAddress radix;
            Program::MemoryAddress output_pointer;
            Program::MemoryAddress num_limbs;
            Program::MemoryAddress output_bits;

            friend bool operator==(const ToRadix&, const ToRadix&);
            std::vector<uint8_t> bincodeSerialize() const;
            static ToRadix bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<AES128Encrypt, Blake2s, Blake3, Keccakf1600, EcdsaSecp256k1, EcdsaSecp256r1, MultiScalarMul, EmbeddedCurveAdd, BigIntAdd, BigIntSub, BigIntMul, BigIntDiv, BigIntFromLeBytes, BigIntToLeBytes, Poseidon2Permutation, Sha256Compression, ToRadix> value;

        friend bool operator==(const BlackBoxOp&, const BlackBoxOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BlackBoxOp bincodeDeserialize(std::vector<uint8_t>);
    };

    struct HeapValueType;

    struct HeapValueType {

        struct Simple {
            Program::BitSize value;

            friend bool operator==(const Simple&, const Simple&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Simple bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Array {
            std::vector<Program::HeapValueType> value_types;
            uint64_t size;

            friend bool operator==(const Array&, const Array&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Array bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Vector {
            std::vector<Program::HeapValueType> value_types;

            friend bool operator==(const Vector&, const Vector&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Vector bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Simple, Array, Vector> value;

        friend bool operator==(const HeapValueType&, const HeapValueType&);
        std::vector<uint8_t> bincodeSerialize() const;
        static HeapValueType bincodeDeserialize(std::vector<uint8_t>);
    };

    struct ValueOrArray {

        struct MemoryAddress {
            Program::MemoryAddress value;

            friend bool operator==(const MemoryAddress&, const MemoryAddress&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryAddress bincodeDeserialize(std::vector<uint8_t>);
        };

        struct HeapArray {
            Program::HeapArray value;

            friend bool operator==(const HeapArray&, const HeapArray&);
            std::vector<uint8_t> bincodeSerialize() const;
            static HeapArray bincodeDeserialize(std::vector<uint8_t>);
        };

        struct HeapVector {
            Program::HeapVector value;

            friend bool operator==(const HeapVector&, const HeapVector&);
            std::vector<uint8_t> bincodeSerialize() const;
            static HeapVector bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<MemoryAddress, HeapArray, HeapVector> value;

        friend bool operator==(const ValueOrArray&, const ValueOrArray&);
        std::vector<uint8_t> bincodeSerialize() const;
        static ValueOrArray bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BrilligOpcode {

        struct BinaryFieldOp {
            Program::MemoryAddress destination;
            Program::BinaryFieldOp op;
            Program::MemoryAddress lhs;
            Program::MemoryAddress rhs;

            friend bool operator==(const BinaryFieldOp&, const BinaryFieldOp&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BinaryFieldOp bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BinaryIntOp {
            Program::MemoryAddress destination;
            Program::BinaryIntOp op;
            Program::IntegerBitSize bit_size;
            Program::MemoryAddress lhs;
            Program::MemoryAddress rhs;

            friend bool operator==(const BinaryIntOp&, const BinaryIntOp&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BinaryIntOp bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Not {
            Program::MemoryAddress destination;
            Program::MemoryAddress source;
            Program::IntegerBitSize bit_size;

            friend bool operator==(const Not&, const Not&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Not bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Cast {
            Program::MemoryAddress destination;
            Program::MemoryAddress source;
            Program::BitSize bit_size;

            friend bool operator==(const Cast&, const Cast&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Cast bincodeDeserialize(std::vector<uint8_t>);
        };

        struct JumpIfNot {
            Program::MemoryAddress condition;
            uint64_t location;

            friend bool operator==(const JumpIfNot&, const JumpIfNot&);
            std::vector<uint8_t> bincodeSerialize() const;
            static JumpIfNot bincodeDeserialize(std::vector<uint8_t>);
        };

        struct JumpIf {
            Program::MemoryAddress condition;
            uint64_t location;

            friend bool operator==(const JumpIf&, const JumpIf&);
            std::vector<uint8_t> bincodeSerialize() const;
            static JumpIf bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Jump {
            uint64_t location;

            friend bool operator==(const Jump&, const Jump&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Jump bincodeDeserialize(std::vector<uint8_t>);
        };

        struct CalldataCopy {
            Program::MemoryAddress destination_address;
            Program::MemoryAddress size_address;
            Program::MemoryAddress offset_address;

            friend bool operator==(const CalldataCopy&, const CalldataCopy&);
            std::vector<uint8_t> bincodeSerialize() const;
            static CalldataCopy bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Call {
            uint64_t location;

            friend bool operator==(const Call&, const Call&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Call bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Const {
            Program::MemoryAddress destination;
            Program::BitSize bit_size;
            std::string value;

            friend bool operator==(const Const&, const Const&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Const bincodeDeserialize(std::vector<uint8_t>);
        };

        struct IndirectConst {
            Program::MemoryAddress destination_pointer;
            Program::BitSize bit_size;
            std::string value;

            friend bool operator==(const IndirectConst&, const IndirectConst&);
            std::vector<uint8_t> bincodeSerialize() const;
            static IndirectConst bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Return {
            friend bool operator==(const Return&, const Return&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Return bincodeDeserialize(std::vector<uint8_t>);
        };

        struct ForeignCall {
            std::string function;
            std::vector<Program::ValueOrArray> destinations;
            std::vector<Program::HeapValueType> destination_value_types;
            std::vector<Program::ValueOrArray> inputs;
            std::vector<Program::HeapValueType> input_value_types;

            friend bool operator==(const ForeignCall&, const ForeignCall&);
            std::vector<uint8_t> bincodeSerialize() const;
            static ForeignCall bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Mov {
            Program::MemoryAddress destination;
            Program::MemoryAddress source;

            friend bool operator==(const Mov&, const Mov&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Mov bincodeDeserialize(std::vector<uint8_t>);
        };

        struct ConditionalMov {
            Program::MemoryAddress destination;
            Program::MemoryAddress source_a;
            Program::MemoryAddress source_b;
            Program::MemoryAddress condition;

            friend bool operator==(const ConditionalMov&, const ConditionalMov&);
            std::vector<uint8_t> bincodeSerialize() const;
            static ConditionalMov bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Load {
            Program::MemoryAddress destination;
            Program::MemoryAddress source_pointer;

            friend bool operator==(const Load&, const Load&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Load bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Store {
            Program::MemoryAddress destination_pointer;
            Program::MemoryAddress source;

            friend bool operator==(const Store&, const Store&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Store bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BlackBox {
            Program::BlackBoxOp value;

            friend bool operator==(const BlackBox&, const BlackBox&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BlackBox bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Trap {
            Program::HeapVector revert_data;

            friend bool operator==(const Trap&, const Trap&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Trap bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Stop {
            Program::HeapVector return_data;

            friend bool operator==(const Stop&, const Stop&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Stop bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<BinaryFieldOp, BinaryIntOp, Not, Cast, JumpIfNot, JumpIf, Jump, CalldataCopy, Call, Const, IndirectConst, Return, ForeignCall, Mov, ConditionalMov, Load, Store, BlackBox, Trap, Stop> value;

        friend bool operator==(const BrilligOpcode&, const BrilligOpcode&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligOpcode bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Witness {
        uint32_t value;

        friend bool operator==(const Witness&, const Witness&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Witness bincodeDeserialize(std::vector<uint8_t>);
    };

    struct ConstantOrWitnessEnum {

        struct Constant {
            std::string value;

            friend bool operator==(const Constant&, const Constant&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Constant bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Witness {
            Program::Witness value;

            friend bool operator==(const Witness&, const Witness&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Witness bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Constant, Witness> value;

        friend bool operator==(const ConstantOrWitnessEnum&, const ConstantOrWitnessEnum&);
        std::vector<uint8_t> bincodeSerialize() const;
        static ConstantOrWitnessEnum bincodeDeserialize(std::vector<uint8_t>);
    };

    struct FunctionInput {
        Program::ConstantOrWitnessEnum input;
        uint32_t num_bits;

        friend bool operator==(const FunctionInput&, const FunctionInput&);
        std::vector<uint8_t> bincodeSerialize() const;
        static FunctionInput bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BlackBoxFuncCall {

        struct AES128Encrypt {
            std::vector<Program::FunctionInput> inputs;
            std::array<Program::FunctionInput, 16> iv;
            std::array<Program::FunctionInput, 16> key;
            std::vector<Program::Witness> outputs;

            friend bool operator==(const AES128Encrypt&, const AES128Encrypt&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AES128Encrypt bincodeDeserialize(std::vector<uint8_t>);
        };

        struct AND {
            Program::FunctionInput lhs;
            Program::FunctionInput rhs;
            Program::Witness output;

            friend bool operator==(const AND&, const AND&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AND bincodeDeserialize(std::vector<uint8_t>);
        };

        struct XOR {
            Program::FunctionInput lhs;
            Program::FunctionInput rhs;
            Program::Witness output;

            friend bool operator==(const XOR&, const XOR&);
            std::vector<uint8_t> bincodeSerialize() const;
            static XOR bincodeDeserialize(std::vector<uint8_t>);
        };

        struct RANGE {
            Program::FunctionInput input;

            friend bool operator==(const RANGE&, const RANGE&);
            std::vector<uint8_t> bincodeSerialize() const;
            static RANGE bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Blake2s {
            std::vector<Program::FunctionInput> inputs;
            std::array<Program::Witness, 32> outputs;

            friend bool operator==(const Blake2s&, const Blake2s&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake2s bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Blake3 {
            std::vector<Program::FunctionInput> inputs;
            std::array<Program::Witness, 32> outputs;

            friend bool operator==(const Blake3&, const Blake3&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake3 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EcdsaSecp256k1 {
            std::array<Program::FunctionInput, 32> public_key_x;
            std::array<Program::FunctionInput, 32> public_key_y;
            std::array<Program::FunctionInput, 64> signature;
            std::array<Program::FunctionInput, 32> hashed_message;
            Program::Witness output;

            friend bool operator==(const EcdsaSecp256k1&, const EcdsaSecp256k1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256k1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EcdsaSecp256r1 {
            std::array<Program::FunctionInput, 32> public_key_x;
            std::array<Program::FunctionInput, 32> public_key_y;
            std::array<Program::FunctionInput, 64> signature;
            std::array<Program::FunctionInput, 32> hashed_message;
            Program::Witness output;

            friend bool operator==(const EcdsaSecp256r1&, const EcdsaSecp256r1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256r1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct MultiScalarMul {
            std::vector<Program::FunctionInput> points;
            std::vector<Program::FunctionInput> scalars;
            std::array<Program::Witness, 3> outputs;

            friend bool operator==(const MultiScalarMul&, const MultiScalarMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MultiScalarMul bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EmbeddedCurveAdd {
            std::array<Program::FunctionInput, 3> input1;
            std::array<Program::FunctionInput, 3> input2;
            std::array<Program::Witness, 3> outputs;

            friend bool operator==(const EmbeddedCurveAdd&, const EmbeddedCurveAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EmbeddedCurveAdd bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Keccakf1600 {
            std::array<Program::FunctionInput, 25> inputs;
            std::array<Program::Witness, 25> outputs;

            friend bool operator==(const Keccakf1600&, const Keccakf1600&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccakf1600 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct RecursiveAggregation {
            std::vector<Program::FunctionInput> verification_key;
            std::vector<Program::FunctionInput> proof;
            std::vector<Program::FunctionInput> public_inputs;
            Program::FunctionInput key_hash;
            uint32_t proof_type;

            friend bool operator==(const RecursiveAggregation&, const RecursiveAggregation&);
            std::vector<uint8_t> bincodeSerialize() const;
            static RecursiveAggregation bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntAdd {
            uint32_t lhs;
            uint32_t rhs;
            uint32_t output;

            friend bool operator==(const BigIntAdd&, const BigIntAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntAdd bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntSub {
            uint32_t lhs;
            uint32_t rhs;
            uint32_t output;

            friend bool operator==(const BigIntSub&, const BigIntSub&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntSub bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntMul {
            uint32_t lhs;
            uint32_t rhs;
            uint32_t output;

            friend bool operator==(const BigIntMul&, const BigIntMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntMul bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntDiv {
            uint32_t lhs;
            uint32_t rhs;
            uint32_t output;

            friend bool operator==(const BigIntDiv&, const BigIntDiv&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntDiv bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntFromLeBytes {
            std::vector<Program::FunctionInput> inputs;
            std::vector<uint8_t> modulus;
            uint32_t output;

            friend bool operator==(const BigIntFromLeBytes&, const BigIntFromLeBytes&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntFromLeBytes bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BigIntToLeBytes {
            uint32_t input;
            std::vector<Program::Witness> outputs;

            friend bool operator==(const BigIntToLeBytes&, const BigIntToLeBytes&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BigIntToLeBytes bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Poseidon2Permutation {
            std::vector<Program::FunctionInput> inputs;
            std::vector<Program::Witness> outputs;
            uint32_t len;

            friend bool operator==(const Poseidon2Permutation&, const Poseidon2Permutation&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Poseidon2Permutation bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Sha256Compression {
            std::array<Program::FunctionInput, 16> inputs;
            std::array<Program::FunctionInput, 8> hash_values;
            std::array<Program::Witness, 8> outputs;

            friend bool operator==(const Sha256Compression&, const Sha256Compression&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sha256Compression bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<AES128Encrypt, AND, XOR, RANGE, Blake2s, Blake3, EcdsaSecp256k1, EcdsaSecp256r1, MultiScalarMul, EmbeddedCurveAdd, Keccakf1600, RecursiveAggregation, BigIntAdd, BigIntSub, BigIntMul, BigIntDiv, BigIntFromLeBytes, BigIntToLeBytes, Poseidon2Permutation, Sha256Compression> value;

        friend bool operator==(const BlackBoxFuncCall&, const BlackBoxFuncCall&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BlackBoxFuncCall bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BlockId {
        uint32_t value;

        friend bool operator==(const BlockId&, const BlockId&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BlockId bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BlockType {

        struct Memory {
            friend bool operator==(const Memory&, const Memory&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Memory bincodeDeserialize(std::vector<uint8_t>);
        };

        struct CallData {
            uint32_t value;

            friend bool operator==(const CallData&, const CallData&);
            std::vector<uint8_t> bincodeSerialize() const;
            static CallData bincodeDeserialize(std::vector<uint8_t>);
        };

        struct ReturnData {
            friend bool operator==(const ReturnData&, const ReturnData&);
            std::vector<uint8_t> bincodeSerialize() const;
            static ReturnData bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Memory, CallData, ReturnData> value;

        friend bool operator==(const BlockType&, const BlockType&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BlockType bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Expression {
        std::vector<std::tuple<std::string, Program::Witness, Program::Witness>> mul_terms;
        std::vector<std::tuple<std::string, Program::Witness>> linear_combinations;
        std::string q_c;

        friend bool operator==(const Expression&, const Expression&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Expression bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BrilligInputs {

        struct Single {
            Program::Expression value;

            friend bool operator==(const Single&, const Single&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Single bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Array {
            std::vector<Program::Expression> value;

            friend bool operator==(const Array&, const Array&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Array bincodeDeserialize(std::vector<uint8_t>);
        };

        struct MemoryArray {
            Program::BlockId value;

            friend bool operator==(const MemoryArray&, const MemoryArray&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryArray bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Single, Array, MemoryArray> value;

        friend bool operator==(const BrilligInputs&, const BrilligInputs&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligInputs bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BrilligOutputs {

        struct Simple {
            Program::Witness value;

            friend bool operator==(const Simple&, const Simple&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Simple bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Array {
            std::vector<Program::Witness> value;

            friend bool operator==(const Array&, const Array&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Array bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Simple, Array> value;

        friend bool operator==(const BrilligOutputs&, const BrilligOutputs&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligOutputs bincodeDeserialize(std::vector<uint8_t>);
    };

    struct MemOp {
        Program::Expression operation;
        Program::Expression index;
        Program::Expression value;

        friend bool operator==(const MemOp&, const MemOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static MemOp bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Opcode {

        struct AssertZero {
            Program::Expression value;

            friend bool operator==(const AssertZero&, const AssertZero&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AssertZero bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BlackBoxFuncCall {
            Program::BlackBoxFuncCall value;

            friend bool operator==(const BlackBoxFuncCall&, const BlackBoxFuncCall&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BlackBoxFuncCall bincodeDeserialize(std::vector<uint8_t>);
        };

        struct MemoryOp {
            Program::BlockId block_id;
            Program::MemOp op;
            std::optional<Program::Expression> predicate;

            friend bool operator==(const MemoryOp&, const MemoryOp&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryOp bincodeDeserialize(std::vector<uint8_t>);
        };

        struct MemoryInit {
            Program::BlockId block_id;
            std::vector<Program::Witness> init;
            Program::BlockType block_type;

            friend bool operator==(const MemoryInit&, const MemoryInit&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryInit bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BrilligCall {
            uint32_t id;
            std::vector<Program::BrilligInputs> inputs;
            std::vector<Program::BrilligOutputs> outputs;
            std::optional<Program::Expression> predicate;

            friend bool operator==(const BrilligCall&, const BrilligCall&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BrilligCall bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Call {
            uint32_t id;
            std::vector<Program::Witness> inputs;
            std::vector<Program::Witness> outputs;
            std::optional<Program::Expression> predicate;

            friend bool operator==(const Call&, const Call&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Call bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<AssertZero, BlackBoxFuncCall, MemoryOp, MemoryInit, BrilligCall, Call> value;

        friend bool operator==(const Opcode&, const Opcode&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Opcode bincodeDeserialize(std::vector<uint8_t>);
    };

    struct ExpressionOrMemory {

        struct Expression {
            Program::Expression value;

            friend bool operator==(const Expression&, const Expression&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Expression bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Memory {
            Program::BlockId value;

            friend bool operator==(const Memory&, const Memory&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Memory bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Expression, Memory> value;

        friend bool operator==(const ExpressionOrMemory&, const ExpressionOrMemory&);
        std::vector<uint8_t> bincodeSerialize() const;
        static ExpressionOrMemory bincodeDeserialize(std::vector<uint8_t>);
    };

    struct AssertionPayload {
        uint64_t error_selector;
        std::vector<Program::ExpressionOrMemory> payload;

        friend bool operator==(const AssertionPayload&, const AssertionPayload&);
        std::vector<uint8_t> bincodeSerialize() const;
        static AssertionPayload bincodeDeserialize(std::vector<uint8_t>);
    };

    struct ExpressionWidth {

        struct Unbounded {
            friend bool operator==(const Unbounded&, const Unbounded&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Unbounded bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Bounded {
            uint64_t width;

            friend bool operator==(const Bounded&, const Bounded&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Bounded bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Unbounded, Bounded> value;

        friend bool operator==(const ExpressionWidth&, const ExpressionWidth&);
        std::vector<uint8_t> bincodeSerialize() const;
        static ExpressionWidth bincodeDeserialize(std::vector<uint8_t>);
    };

    struct OpcodeLocation {

        struct Acir {
            uint64_t value;

            friend bool operator==(const Acir&, const Acir&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Acir bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Brillig {
            uint64_t acir_index;
            uint64_t brillig_index;

            friend bool operator==(const Brillig&, const Brillig&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Brillig bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Acir, Brillig> value;

        friend bool operator==(const OpcodeLocation&, const OpcodeLocation&);
        std::vector<uint8_t> bincodeSerialize() const;
        static OpcodeLocation bincodeDeserialize(std::vector<uint8_t>);
    };

    struct PublicInputs {
        std::vector<Program::Witness> value;

        friend bool operator==(const PublicInputs&, const PublicInputs&);
        std::vector<uint8_t> bincodeSerialize() const;
        static PublicInputs bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Circuit {
        uint32_t current_witness_index;
        std::vector<Program::Opcode> opcodes;
        Program::ExpressionWidth expression_width;
        std::vector<Program::Witness> private_parameters;
        Program::PublicInputs public_parameters;
        Program::PublicInputs return_values;
        std::vector<std::tuple<Program::OpcodeLocation, Program::AssertionPayload>> assert_messages;

        friend bool operator==(const Circuit&, const Circuit&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Circuit bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BrilligBytecode {
        std::vector<Program::BrilligOpcode> bytecode;

        friend bool operator==(const BrilligBytecode&, const BrilligBytecode&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligBytecode bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Program {
        std::vector<Program::Circuit> functions;
        std::vector<Program::BrilligBytecode> unconstrained_functions;

        friend bool operator==(const Program&, const Program&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Program bincodeDeserialize(std::vector<uint8_t>);
    };

} // end of namespace Program


namespace Program {

    inline bool operator==(const AssertionPayload &lhs, const AssertionPayload &rhs) {
        if (!(lhs.error_selector == rhs.error_selector)) { return false; }
        if (!(lhs.payload == rhs.payload)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> AssertionPayload::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<AssertionPayload>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline AssertionPayload AssertionPayload::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<AssertionPayload>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::AssertionPayload>::serialize(const Program::AssertionPayload &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.error_selector)>::serialize(obj.error_selector, serializer);
    serde::Serializable<decltype(obj.payload)>::serialize(obj.payload, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::AssertionPayload serde::Deserializable<Program::AssertionPayload>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::AssertionPayload obj;
    obj.error_selector = serde::Deserializable<decltype(obj.error_selector)>::deserialize(deserializer);
    obj.payload = serde::Deserializable<decltype(obj.payload)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp &lhs, const BinaryFieldOp &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp BinaryFieldOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp>::serialize(const Program::BinaryFieldOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp serde::Deserializable<Program::BinaryFieldOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BinaryFieldOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp::Add &lhs, const BinaryFieldOp::Add &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::Add::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Add>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Add BinaryFieldOp::Add::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Add>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp::Add>::serialize(const Program::BinaryFieldOp::Add &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp::Add serde::Deserializable<Program::BinaryFieldOp::Add>::deserialize(Deserializer &deserializer) {
    Program::BinaryFieldOp::Add obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp::Sub &lhs, const BinaryFieldOp::Sub &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::Sub::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Sub>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Sub BinaryFieldOp::Sub::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Sub>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp::Sub>::serialize(const Program::BinaryFieldOp::Sub &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp::Sub serde::Deserializable<Program::BinaryFieldOp::Sub>::deserialize(Deserializer &deserializer) {
    Program::BinaryFieldOp::Sub obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp::Mul &lhs, const BinaryFieldOp::Mul &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::Mul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Mul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Mul BinaryFieldOp::Mul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Mul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp::Mul>::serialize(const Program::BinaryFieldOp::Mul &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp::Mul serde::Deserializable<Program::BinaryFieldOp::Mul>::deserialize(Deserializer &deserializer) {
    Program::BinaryFieldOp::Mul obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp::Div &lhs, const BinaryFieldOp::Div &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::Div::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Div>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Div BinaryFieldOp::Div::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Div>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp::Div>::serialize(const Program::BinaryFieldOp::Div &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp::Div serde::Deserializable<Program::BinaryFieldOp::Div>::deserialize(Deserializer &deserializer) {
    Program::BinaryFieldOp::Div obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp::IntegerDiv &lhs, const BinaryFieldOp::IntegerDiv &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::IntegerDiv::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::IntegerDiv>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::IntegerDiv BinaryFieldOp::IntegerDiv::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::IntegerDiv>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp::IntegerDiv>::serialize(const Program::BinaryFieldOp::IntegerDiv &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp::IntegerDiv serde::Deserializable<Program::BinaryFieldOp::IntegerDiv>::deserialize(Deserializer &deserializer) {
    Program::BinaryFieldOp::IntegerDiv obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp::Equals &lhs, const BinaryFieldOp::Equals &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::Equals::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::Equals>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::Equals BinaryFieldOp::Equals::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::Equals>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp::Equals>::serialize(const Program::BinaryFieldOp::Equals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp::Equals serde::Deserializable<Program::BinaryFieldOp::Equals>::deserialize(Deserializer &deserializer) {
    Program::BinaryFieldOp::Equals obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp::LessThan &lhs, const BinaryFieldOp::LessThan &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::LessThan::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::LessThan>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::LessThan BinaryFieldOp::LessThan::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::LessThan>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp::LessThan>::serialize(const Program::BinaryFieldOp::LessThan &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp::LessThan serde::Deserializable<Program::BinaryFieldOp::LessThan>::deserialize(Deserializer &deserializer) {
    Program::BinaryFieldOp::LessThan obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryFieldOp::LessThanEquals &lhs, const BinaryFieldOp::LessThanEquals &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryFieldOp::LessThanEquals::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryFieldOp::LessThanEquals>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryFieldOp::LessThanEquals BinaryFieldOp::LessThanEquals::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryFieldOp::LessThanEquals>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryFieldOp::LessThanEquals>::serialize(const Program::BinaryFieldOp::LessThanEquals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryFieldOp::LessThanEquals serde::Deserializable<Program::BinaryFieldOp::LessThanEquals>::deserialize(Deserializer &deserializer) {
    Program::BinaryFieldOp::LessThanEquals obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp &lhs, const BinaryIntOp &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp BinaryIntOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp>::serialize(const Program::BinaryIntOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BinaryIntOp serde::Deserializable<Program::BinaryIntOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BinaryIntOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Add &lhs, const BinaryIntOp::Add &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Add::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Add>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Add BinaryIntOp::Add::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Add>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Add>::serialize(const Program::BinaryIntOp::Add &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Add serde::Deserializable<Program::BinaryIntOp::Add>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Add obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Sub &lhs, const BinaryIntOp::Sub &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Sub::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Sub>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Sub BinaryIntOp::Sub::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Sub>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Sub>::serialize(const Program::BinaryIntOp::Sub &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Sub serde::Deserializable<Program::BinaryIntOp::Sub>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Sub obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Mul &lhs, const BinaryIntOp::Mul &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Mul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Mul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Mul BinaryIntOp::Mul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Mul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Mul>::serialize(const Program::BinaryIntOp::Mul &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Mul serde::Deserializable<Program::BinaryIntOp::Mul>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Mul obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Div &lhs, const BinaryIntOp::Div &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Div::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Div>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Div BinaryIntOp::Div::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Div>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Div>::serialize(const Program::BinaryIntOp::Div &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Div serde::Deserializable<Program::BinaryIntOp::Div>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Div obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Equals &lhs, const BinaryIntOp::Equals &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Equals::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Equals>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Equals BinaryIntOp::Equals::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Equals>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Equals>::serialize(const Program::BinaryIntOp::Equals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Equals serde::Deserializable<Program::BinaryIntOp::Equals>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Equals obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::LessThan &lhs, const BinaryIntOp::LessThan &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::LessThan::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::LessThan>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::LessThan BinaryIntOp::LessThan::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::LessThan>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::LessThan>::serialize(const Program::BinaryIntOp::LessThan &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::LessThan serde::Deserializable<Program::BinaryIntOp::LessThan>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::LessThan obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::LessThanEquals &lhs, const BinaryIntOp::LessThanEquals &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::LessThanEquals::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::LessThanEquals>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::LessThanEquals BinaryIntOp::LessThanEquals::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::LessThanEquals>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::LessThanEquals>::serialize(const Program::BinaryIntOp::LessThanEquals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::LessThanEquals serde::Deserializable<Program::BinaryIntOp::LessThanEquals>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::LessThanEquals obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::And &lhs, const BinaryIntOp::And &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::And::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::And>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::And BinaryIntOp::And::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::And>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::And>::serialize(const Program::BinaryIntOp::And &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::And serde::Deserializable<Program::BinaryIntOp::And>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::And obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Or &lhs, const BinaryIntOp::Or &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Or::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Or>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Or BinaryIntOp::Or::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Or>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Or>::serialize(const Program::BinaryIntOp::Or &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Or serde::Deserializable<Program::BinaryIntOp::Or>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Or obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Xor &lhs, const BinaryIntOp::Xor &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Xor::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Xor>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Xor BinaryIntOp::Xor::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Xor>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Xor>::serialize(const Program::BinaryIntOp::Xor &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Xor serde::Deserializable<Program::BinaryIntOp::Xor>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Xor obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Shl &lhs, const BinaryIntOp::Shl &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Shl::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Shl>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Shl BinaryIntOp::Shl::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Shl>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Shl>::serialize(const Program::BinaryIntOp::Shl &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Shl serde::Deserializable<Program::BinaryIntOp::Shl>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Shl obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BinaryIntOp::Shr &lhs, const BinaryIntOp::Shr &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::Shr::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::Shr>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::Shr BinaryIntOp::Shr::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::Shr>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BinaryIntOp::Shr>::serialize(const Program::BinaryIntOp::Shr &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BinaryIntOp::Shr serde::Deserializable<Program::BinaryIntOp::Shr>::deserialize(Deserializer &deserializer) {
    Program::BinaryIntOp::Shr obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BitSize &lhs, const BitSize &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BitSize::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BitSize>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BitSize BitSize::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BitSize>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BitSize>::serialize(const Program::BitSize &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BitSize serde::Deserializable<Program::BitSize>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BitSize obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BitSize::Field &lhs, const BitSize::Field &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BitSize::Field::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BitSize::Field>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BitSize::Field BitSize::Field::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BitSize::Field>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BitSize::Field>::serialize(const Program::BitSize::Field &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BitSize::Field serde::Deserializable<Program::BitSize::Field>::deserialize(Deserializer &deserializer) {
    Program::BitSize::Field obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BitSize::Integer &lhs, const BitSize::Integer &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BitSize::Integer::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BitSize::Integer>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BitSize::Integer BitSize::Integer::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BitSize::Integer>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BitSize::Integer>::serialize(const Program::BitSize::Integer &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BitSize::Integer serde::Deserializable<Program::BitSize::Integer>::deserialize(Deserializer &deserializer) {
    Program::BitSize::Integer obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall &lhs, const BlackBoxFuncCall &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall BlackBoxFuncCall::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall>::serialize(const Program::BlackBoxFuncCall &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall serde::Deserializable<Program::BlackBoxFuncCall>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BlackBoxFuncCall obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::AES128Encrypt &lhs, const BlackBoxFuncCall::AES128Encrypt &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.iv == rhs.iv)) { return false; }
        if (!(lhs.key == rhs.key)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::AES128Encrypt::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::AES128Encrypt>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::AES128Encrypt BlackBoxFuncCall::AES128Encrypt::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::AES128Encrypt>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::AES128Encrypt>::serialize(const Program::BlackBoxFuncCall::AES128Encrypt &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.iv)>::serialize(obj.iv, serializer);
    serde::Serializable<decltype(obj.key)>::serialize(obj.key, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::AES128Encrypt serde::Deserializable<Program::BlackBoxFuncCall::AES128Encrypt>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::AES128Encrypt obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.iv = serde::Deserializable<decltype(obj.iv)>::deserialize(deserializer);
    obj.key = serde::Deserializable<decltype(obj.key)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::AND &lhs, const BlackBoxFuncCall::AND &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::AND::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::AND>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::AND BlackBoxFuncCall::AND::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::AND>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::AND>::serialize(const Program::BlackBoxFuncCall::AND &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::AND serde::Deserializable<Program::BlackBoxFuncCall::AND>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::AND obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::XOR &lhs, const BlackBoxFuncCall::XOR &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::XOR::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::XOR>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::XOR BlackBoxFuncCall::XOR::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::XOR>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::XOR>::serialize(const Program::BlackBoxFuncCall::XOR &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::XOR serde::Deserializable<Program::BlackBoxFuncCall::XOR>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::XOR obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::RANGE &lhs, const BlackBoxFuncCall::RANGE &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::RANGE::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::RANGE>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::RANGE BlackBoxFuncCall::RANGE::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::RANGE>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::RANGE>::serialize(const Program::BlackBoxFuncCall::RANGE &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::RANGE serde::Deserializable<Program::BlackBoxFuncCall::RANGE>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::RANGE obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::Blake2s &lhs, const BlackBoxFuncCall::Blake2s &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Blake2s::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Blake2s>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Blake2s BlackBoxFuncCall::Blake2s::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Blake2s>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::Blake2s>::serialize(const Program::BlackBoxFuncCall::Blake2s &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::Blake2s serde::Deserializable<Program::BlackBoxFuncCall::Blake2s>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::Blake2s obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::Blake3 &lhs, const BlackBoxFuncCall::Blake3 &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Blake3::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Blake3>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Blake3 BlackBoxFuncCall::Blake3::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Blake3>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::Blake3>::serialize(const Program::BlackBoxFuncCall::Blake3 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::Blake3 serde::Deserializable<Program::BlackBoxFuncCall::Blake3>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::Blake3 obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::EcdsaSecp256k1 &lhs, const BlackBoxFuncCall::EcdsaSecp256k1 &rhs) {
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.hashed_message == rhs.hashed_message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::EcdsaSecp256k1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::EcdsaSecp256k1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::EcdsaSecp256k1 BlackBoxFuncCall::EcdsaSecp256k1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::EcdsaSecp256k1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::EcdsaSecp256k1>::serialize(const Program::BlackBoxFuncCall::EcdsaSecp256k1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.hashed_message)>::serialize(obj.hashed_message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::EcdsaSecp256k1 serde::Deserializable<Program::BlackBoxFuncCall::EcdsaSecp256k1>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::EcdsaSecp256k1 obj;
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.hashed_message = serde::Deserializable<decltype(obj.hashed_message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::EcdsaSecp256r1 &lhs, const BlackBoxFuncCall::EcdsaSecp256r1 &rhs) {
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.hashed_message == rhs.hashed_message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::EcdsaSecp256r1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::EcdsaSecp256r1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::EcdsaSecp256r1 BlackBoxFuncCall::EcdsaSecp256r1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::EcdsaSecp256r1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::EcdsaSecp256r1>::serialize(const Program::BlackBoxFuncCall::EcdsaSecp256r1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.hashed_message)>::serialize(obj.hashed_message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::EcdsaSecp256r1 serde::Deserializable<Program::BlackBoxFuncCall::EcdsaSecp256r1>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::EcdsaSecp256r1 obj;
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.hashed_message = serde::Deserializable<decltype(obj.hashed_message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::MultiScalarMul &lhs, const BlackBoxFuncCall::MultiScalarMul &rhs) {
        if (!(lhs.points == rhs.points)) { return false; }
        if (!(lhs.scalars == rhs.scalars)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::MultiScalarMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::MultiScalarMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::MultiScalarMul BlackBoxFuncCall::MultiScalarMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::MultiScalarMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::MultiScalarMul>::serialize(const Program::BlackBoxFuncCall::MultiScalarMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.points)>::serialize(obj.points, serializer);
    serde::Serializable<decltype(obj.scalars)>::serialize(obj.scalars, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::MultiScalarMul serde::Deserializable<Program::BlackBoxFuncCall::MultiScalarMul>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::MultiScalarMul obj;
    obj.points = serde::Deserializable<decltype(obj.points)>::deserialize(deserializer);
    obj.scalars = serde::Deserializable<decltype(obj.scalars)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::EmbeddedCurveAdd &lhs, const BlackBoxFuncCall::EmbeddedCurveAdd &rhs) {
        if (!(lhs.input1 == rhs.input1)) { return false; }
        if (!(lhs.input2 == rhs.input2)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::EmbeddedCurveAdd::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::EmbeddedCurveAdd>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::EmbeddedCurveAdd BlackBoxFuncCall::EmbeddedCurveAdd::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::EmbeddedCurveAdd>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::EmbeddedCurveAdd>::serialize(const Program::BlackBoxFuncCall::EmbeddedCurveAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input1)>::serialize(obj.input1, serializer);
    serde::Serializable<decltype(obj.input2)>::serialize(obj.input2, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::EmbeddedCurveAdd serde::Deserializable<Program::BlackBoxFuncCall::EmbeddedCurveAdd>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::EmbeddedCurveAdd obj;
    obj.input1 = serde::Deserializable<decltype(obj.input1)>::deserialize(deserializer);
    obj.input2 = serde::Deserializable<decltype(obj.input2)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::Keccakf1600 &lhs, const BlackBoxFuncCall::Keccakf1600 &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Keccakf1600::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Keccakf1600>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Keccakf1600 BlackBoxFuncCall::Keccakf1600::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Keccakf1600>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::Keccakf1600>::serialize(const Program::BlackBoxFuncCall::Keccakf1600 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::Keccakf1600 serde::Deserializable<Program::BlackBoxFuncCall::Keccakf1600>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::Keccakf1600 obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::RecursiveAggregation &lhs, const BlackBoxFuncCall::RecursiveAggregation &rhs) {
        if (!(lhs.verification_key == rhs.verification_key)) { return false; }
        if (!(lhs.proof == rhs.proof)) { return false; }
        if (!(lhs.public_inputs == rhs.public_inputs)) { return false; }
        if (!(lhs.key_hash == rhs.key_hash)) { return false; }
        if (!(lhs.proof_type == rhs.proof_type)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::RecursiveAggregation::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::RecursiveAggregation>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::RecursiveAggregation BlackBoxFuncCall::RecursiveAggregation::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::RecursiveAggregation>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::RecursiveAggregation>::serialize(const Program::BlackBoxFuncCall::RecursiveAggregation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.verification_key)>::serialize(obj.verification_key, serializer);
    serde::Serializable<decltype(obj.proof)>::serialize(obj.proof, serializer);
    serde::Serializable<decltype(obj.public_inputs)>::serialize(obj.public_inputs, serializer);
    serde::Serializable<decltype(obj.key_hash)>::serialize(obj.key_hash, serializer);
    serde::Serializable<decltype(obj.proof_type)>::serialize(obj.proof_type, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::RecursiveAggregation serde::Deserializable<Program::BlackBoxFuncCall::RecursiveAggregation>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::RecursiveAggregation obj;
    obj.verification_key = serde::Deserializable<decltype(obj.verification_key)>::deserialize(deserializer);
    obj.proof = serde::Deserializable<decltype(obj.proof)>::deserialize(deserializer);
    obj.public_inputs = serde::Deserializable<decltype(obj.public_inputs)>::deserialize(deserializer);
    obj.key_hash = serde::Deserializable<decltype(obj.key_hash)>::deserialize(deserializer);
    obj.proof_type = serde::Deserializable<decltype(obj.proof_type)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::BigIntAdd &lhs, const BlackBoxFuncCall::BigIntAdd &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntAdd::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntAdd>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntAdd BlackBoxFuncCall::BigIntAdd::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntAdd>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::BigIntAdd>::serialize(const Program::BlackBoxFuncCall::BigIntAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::BigIntAdd serde::Deserializable<Program::BlackBoxFuncCall::BigIntAdd>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::BigIntAdd obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::BigIntSub &lhs, const BlackBoxFuncCall::BigIntSub &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntSub::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntSub>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntSub BlackBoxFuncCall::BigIntSub::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntSub>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::BigIntSub>::serialize(const Program::BlackBoxFuncCall::BigIntSub &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::BigIntSub serde::Deserializable<Program::BlackBoxFuncCall::BigIntSub>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::BigIntSub obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::BigIntMul &lhs, const BlackBoxFuncCall::BigIntMul &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntMul BlackBoxFuncCall::BigIntMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::BigIntMul>::serialize(const Program::BlackBoxFuncCall::BigIntMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::BigIntMul serde::Deserializable<Program::BlackBoxFuncCall::BigIntMul>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::BigIntMul obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::BigIntDiv &lhs, const BlackBoxFuncCall::BigIntDiv &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntDiv::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntDiv>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntDiv BlackBoxFuncCall::BigIntDiv::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntDiv>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::BigIntDiv>::serialize(const Program::BlackBoxFuncCall::BigIntDiv &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::BigIntDiv serde::Deserializable<Program::BlackBoxFuncCall::BigIntDiv>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::BigIntDiv obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::BigIntFromLeBytes &lhs, const BlackBoxFuncCall::BigIntFromLeBytes &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.modulus == rhs.modulus)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntFromLeBytes::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntFromLeBytes>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntFromLeBytes BlackBoxFuncCall::BigIntFromLeBytes::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntFromLeBytes>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::BigIntFromLeBytes>::serialize(const Program::BlackBoxFuncCall::BigIntFromLeBytes &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.modulus)>::serialize(obj.modulus, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::BigIntFromLeBytes serde::Deserializable<Program::BlackBoxFuncCall::BigIntFromLeBytes>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::BigIntFromLeBytes obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.modulus = serde::Deserializable<decltype(obj.modulus)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::BigIntToLeBytes &lhs, const BlackBoxFuncCall::BigIntToLeBytes &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::BigIntToLeBytes::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::BigIntToLeBytes>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::BigIntToLeBytes BlackBoxFuncCall::BigIntToLeBytes::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::BigIntToLeBytes>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::BigIntToLeBytes>::serialize(const Program::BlackBoxFuncCall::BigIntToLeBytes &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::BigIntToLeBytes serde::Deserializable<Program::BlackBoxFuncCall::BigIntToLeBytes>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::BigIntToLeBytes obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::Poseidon2Permutation &lhs, const BlackBoxFuncCall::Poseidon2Permutation &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        if (!(lhs.len == rhs.len)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Poseidon2Permutation::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Poseidon2Permutation>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Poseidon2Permutation BlackBoxFuncCall::Poseidon2Permutation::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Poseidon2Permutation>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::Poseidon2Permutation>::serialize(const Program::BlackBoxFuncCall::Poseidon2Permutation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
    serde::Serializable<decltype(obj.len)>::serialize(obj.len, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::Poseidon2Permutation serde::Deserializable<Program::BlackBoxFuncCall::Poseidon2Permutation>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::Poseidon2Permutation obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    obj.len = serde::Deserializable<decltype(obj.len)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxFuncCall::Sha256Compression &lhs, const BlackBoxFuncCall::Sha256Compression &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.hash_values == rhs.hash_values)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Sha256Compression::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Sha256Compression>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Sha256Compression BlackBoxFuncCall::Sha256Compression::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Sha256Compression>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxFuncCall::Sha256Compression>::serialize(const Program::BlackBoxFuncCall::Sha256Compression &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.hash_values)>::serialize(obj.hash_values, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxFuncCall::Sha256Compression serde::Deserializable<Program::BlackBoxFuncCall::Sha256Compression>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxFuncCall::Sha256Compression obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.hash_values = serde::Deserializable<decltype(obj.hash_values)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp &lhs, const BlackBoxOp &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp BlackBoxOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp>::serialize(const Program::BlackBoxOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BlackBoxOp serde::Deserializable<Program::BlackBoxOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BlackBoxOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::AES128Encrypt &lhs, const BlackBoxOp::AES128Encrypt &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.iv == rhs.iv)) { return false; }
        if (!(lhs.key == rhs.key)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::AES128Encrypt::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::AES128Encrypt>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::AES128Encrypt BlackBoxOp::AES128Encrypt::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::AES128Encrypt>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::AES128Encrypt>::serialize(const Program::BlackBoxOp::AES128Encrypt &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.iv)>::serialize(obj.iv, serializer);
    serde::Serializable<decltype(obj.key)>::serialize(obj.key, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::AES128Encrypt serde::Deserializable<Program::BlackBoxOp::AES128Encrypt>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::AES128Encrypt obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.iv = serde::Deserializable<decltype(obj.iv)>::deserialize(deserializer);
    obj.key = serde::Deserializable<decltype(obj.key)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::Blake2s &lhs, const BlackBoxOp::Blake2s &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Blake2s::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Blake2s>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Blake2s BlackBoxOp::Blake2s::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Blake2s>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::Blake2s>::serialize(const Program::BlackBoxOp::Blake2s &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::Blake2s serde::Deserializable<Program::BlackBoxOp::Blake2s>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::Blake2s obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::Blake3 &lhs, const BlackBoxOp::Blake3 &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Blake3::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Blake3>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Blake3 BlackBoxOp::Blake3::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Blake3>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::Blake3>::serialize(const Program::BlackBoxOp::Blake3 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::Blake3 serde::Deserializable<Program::BlackBoxOp::Blake3>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::Blake3 obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::Keccakf1600 &lhs, const BlackBoxOp::Keccakf1600 &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Keccakf1600::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Keccakf1600>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Keccakf1600 BlackBoxOp::Keccakf1600::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Keccakf1600>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::Keccakf1600>::serialize(const Program::BlackBoxOp::Keccakf1600 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::Keccakf1600 serde::Deserializable<Program::BlackBoxOp::Keccakf1600>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::Keccakf1600 obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::EcdsaSecp256k1 &lhs, const BlackBoxOp::EcdsaSecp256k1 &rhs) {
        if (!(lhs.hashed_msg == rhs.hashed_msg)) { return false; }
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::EcdsaSecp256k1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::EcdsaSecp256k1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::EcdsaSecp256k1 BlackBoxOp::EcdsaSecp256k1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::EcdsaSecp256k1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::EcdsaSecp256k1>::serialize(const Program::BlackBoxOp::EcdsaSecp256k1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.hashed_msg)>::serialize(obj.hashed_msg, serializer);
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::EcdsaSecp256k1 serde::Deserializable<Program::BlackBoxOp::EcdsaSecp256k1>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::EcdsaSecp256k1 obj;
    obj.hashed_msg = serde::Deserializable<decltype(obj.hashed_msg)>::deserialize(deserializer);
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::EcdsaSecp256r1 &lhs, const BlackBoxOp::EcdsaSecp256r1 &rhs) {
        if (!(lhs.hashed_msg == rhs.hashed_msg)) { return false; }
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::EcdsaSecp256r1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::EcdsaSecp256r1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::EcdsaSecp256r1 BlackBoxOp::EcdsaSecp256r1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::EcdsaSecp256r1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::EcdsaSecp256r1>::serialize(const Program::BlackBoxOp::EcdsaSecp256r1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.hashed_msg)>::serialize(obj.hashed_msg, serializer);
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::EcdsaSecp256r1 serde::Deserializable<Program::BlackBoxOp::EcdsaSecp256r1>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::EcdsaSecp256r1 obj;
    obj.hashed_msg = serde::Deserializable<decltype(obj.hashed_msg)>::deserialize(deserializer);
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::MultiScalarMul &lhs, const BlackBoxOp::MultiScalarMul &rhs) {
        if (!(lhs.points == rhs.points)) { return false; }
        if (!(lhs.scalars == rhs.scalars)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::MultiScalarMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::MultiScalarMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::MultiScalarMul BlackBoxOp::MultiScalarMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::MultiScalarMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::MultiScalarMul>::serialize(const Program::BlackBoxOp::MultiScalarMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.points)>::serialize(obj.points, serializer);
    serde::Serializable<decltype(obj.scalars)>::serialize(obj.scalars, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::MultiScalarMul serde::Deserializable<Program::BlackBoxOp::MultiScalarMul>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::MultiScalarMul obj;
    obj.points = serde::Deserializable<decltype(obj.points)>::deserialize(deserializer);
    obj.scalars = serde::Deserializable<decltype(obj.scalars)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::EmbeddedCurveAdd &lhs, const BlackBoxOp::EmbeddedCurveAdd &rhs) {
        if (!(lhs.input1_x == rhs.input1_x)) { return false; }
        if (!(lhs.input1_y == rhs.input1_y)) { return false; }
        if (!(lhs.input1_infinite == rhs.input1_infinite)) { return false; }
        if (!(lhs.input2_x == rhs.input2_x)) { return false; }
        if (!(lhs.input2_y == rhs.input2_y)) { return false; }
        if (!(lhs.input2_infinite == rhs.input2_infinite)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::EmbeddedCurveAdd::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::EmbeddedCurveAdd>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::EmbeddedCurveAdd BlackBoxOp::EmbeddedCurveAdd::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::EmbeddedCurveAdd>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::EmbeddedCurveAdd>::serialize(const Program::BlackBoxOp::EmbeddedCurveAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input1_x)>::serialize(obj.input1_x, serializer);
    serde::Serializable<decltype(obj.input1_y)>::serialize(obj.input1_y, serializer);
    serde::Serializable<decltype(obj.input1_infinite)>::serialize(obj.input1_infinite, serializer);
    serde::Serializable<decltype(obj.input2_x)>::serialize(obj.input2_x, serializer);
    serde::Serializable<decltype(obj.input2_y)>::serialize(obj.input2_y, serializer);
    serde::Serializable<decltype(obj.input2_infinite)>::serialize(obj.input2_infinite, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::EmbeddedCurveAdd serde::Deserializable<Program::BlackBoxOp::EmbeddedCurveAdd>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::EmbeddedCurveAdd obj;
    obj.input1_x = serde::Deserializable<decltype(obj.input1_x)>::deserialize(deserializer);
    obj.input1_y = serde::Deserializable<decltype(obj.input1_y)>::deserialize(deserializer);
    obj.input1_infinite = serde::Deserializable<decltype(obj.input1_infinite)>::deserialize(deserializer);
    obj.input2_x = serde::Deserializable<decltype(obj.input2_x)>::deserialize(deserializer);
    obj.input2_y = serde::Deserializable<decltype(obj.input2_y)>::deserialize(deserializer);
    obj.input2_infinite = serde::Deserializable<decltype(obj.input2_infinite)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::BigIntAdd &lhs, const BlackBoxOp::BigIntAdd &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntAdd::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntAdd>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntAdd BlackBoxOp::BigIntAdd::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntAdd>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::BigIntAdd>::serialize(const Program::BlackBoxOp::BigIntAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::BigIntAdd serde::Deserializable<Program::BlackBoxOp::BigIntAdd>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::BigIntAdd obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::BigIntSub &lhs, const BlackBoxOp::BigIntSub &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntSub::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntSub>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntSub BlackBoxOp::BigIntSub::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntSub>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::BigIntSub>::serialize(const Program::BlackBoxOp::BigIntSub &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::BigIntSub serde::Deserializable<Program::BlackBoxOp::BigIntSub>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::BigIntSub obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::BigIntMul &lhs, const BlackBoxOp::BigIntMul &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntMul BlackBoxOp::BigIntMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::BigIntMul>::serialize(const Program::BlackBoxOp::BigIntMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::BigIntMul serde::Deserializable<Program::BlackBoxOp::BigIntMul>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::BigIntMul obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::BigIntDiv &lhs, const BlackBoxOp::BigIntDiv &rhs) {
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntDiv::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntDiv>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntDiv BlackBoxOp::BigIntDiv::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntDiv>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::BigIntDiv>::serialize(const Program::BlackBoxOp::BigIntDiv &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::BigIntDiv serde::Deserializable<Program::BlackBoxOp::BigIntDiv>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::BigIntDiv obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::BigIntFromLeBytes &lhs, const BlackBoxOp::BigIntFromLeBytes &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.modulus == rhs.modulus)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntFromLeBytes::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntFromLeBytes>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntFromLeBytes BlackBoxOp::BigIntFromLeBytes::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntFromLeBytes>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::BigIntFromLeBytes>::serialize(const Program::BlackBoxOp::BigIntFromLeBytes &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.modulus)>::serialize(obj.modulus, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::BigIntFromLeBytes serde::Deserializable<Program::BlackBoxOp::BigIntFromLeBytes>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::BigIntFromLeBytes obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.modulus = serde::Deserializable<decltype(obj.modulus)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::BigIntToLeBytes &lhs, const BlackBoxOp::BigIntToLeBytes &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::BigIntToLeBytes::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::BigIntToLeBytes>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::BigIntToLeBytes BlackBoxOp::BigIntToLeBytes::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::BigIntToLeBytes>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::BigIntToLeBytes>::serialize(const Program::BlackBoxOp::BigIntToLeBytes &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::BigIntToLeBytes serde::Deserializable<Program::BlackBoxOp::BigIntToLeBytes>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::BigIntToLeBytes obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::Poseidon2Permutation &lhs, const BlackBoxOp::Poseidon2Permutation &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        if (!(lhs.len == rhs.len)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Poseidon2Permutation::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Poseidon2Permutation>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Poseidon2Permutation BlackBoxOp::Poseidon2Permutation::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Poseidon2Permutation>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::Poseidon2Permutation>::serialize(const Program::BlackBoxOp::Poseidon2Permutation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
    serde::Serializable<decltype(obj.len)>::serialize(obj.len, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::Poseidon2Permutation serde::Deserializable<Program::BlackBoxOp::Poseidon2Permutation>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::Poseidon2Permutation obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    obj.len = serde::Deserializable<decltype(obj.len)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::Sha256Compression &lhs, const BlackBoxOp::Sha256Compression &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.hash_values == rhs.hash_values)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Sha256Compression::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Sha256Compression>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Sha256Compression BlackBoxOp::Sha256Compression::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Sha256Compression>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::Sha256Compression>::serialize(const Program::BlackBoxOp::Sha256Compression &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.hash_values)>::serialize(obj.hash_values, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::Sha256Compression serde::Deserializable<Program::BlackBoxOp::Sha256Compression>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::Sha256Compression obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.hash_values = serde::Deserializable<decltype(obj.hash_values)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlackBoxOp::ToRadix &lhs, const BlackBoxOp::ToRadix &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.radix == rhs.radix)) { return false; }
        if (!(lhs.output_pointer == rhs.output_pointer)) { return false; }
        if (!(lhs.num_limbs == rhs.num_limbs)) { return false; }
        if (!(lhs.output_bits == rhs.output_bits)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::ToRadix::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::ToRadix>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::ToRadix BlackBoxOp::ToRadix::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::ToRadix>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlackBoxOp::ToRadix>::serialize(const Program::BlackBoxOp::ToRadix &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.radix)>::serialize(obj.radix, serializer);
    serde::Serializable<decltype(obj.output_pointer)>::serialize(obj.output_pointer, serializer);
    serde::Serializable<decltype(obj.num_limbs)>::serialize(obj.num_limbs, serializer);
    serde::Serializable<decltype(obj.output_bits)>::serialize(obj.output_bits, serializer);
}

template <>
template <typename Deserializer>
Program::BlackBoxOp::ToRadix serde::Deserializable<Program::BlackBoxOp::ToRadix>::deserialize(Deserializer &deserializer) {
    Program::BlackBoxOp::ToRadix obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.radix = serde::Deserializable<decltype(obj.radix)>::deserialize(deserializer);
    obj.output_pointer = serde::Deserializable<decltype(obj.output_pointer)>::deserialize(deserializer);
    obj.num_limbs = serde::Deserializable<decltype(obj.num_limbs)>::deserialize(deserializer);
    obj.output_bits = serde::Deserializable<decltype(obj.output_bits)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlockId &lhs, const BlockId &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlockId::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockId>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockId BlockId::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockId>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlockId>::serialize(const Program::BlockId &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BlockId serde::Deserializable<Program::BlockId>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BlockId obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BlockType &lhs, const BlockType &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlockType::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockType>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockType BlockType::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockType>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlockType>::serialize(const Program::BlockType &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BlockType serde::Deserializable<Program::BlockType>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BlockType obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BlockType::Memory &lhs, const BlockType::Memory &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BlockType::Memory::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockType::Memory>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockType::Memory BlockType::Memory::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockType::Memory>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlockType::Memory>::serialize(const Program::BlockType::Memory &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BlockType::Memory serde::Deserializable<Program::BlockType::Memory>::deserialize(Deserializer &deserializer) {
    Program::BlockType::Memory obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BlockType::CallData &lhs, const BlockType::CallData &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlockType::CallData::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockType::CallData>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockType::CallData BlockType::CallData::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockType::CallData>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlockType::CallData>::serialize(const Program::BlockType::CallData &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BlockType::CallData serde::Deserializable<Program::BlockType::CallData>::deserialize(Deserializer &deserializer) {
    Program::BlockType::CallData obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BlockType::ReturnData &lhs, const BlockType::ReturnData &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BlockType::ReturnData::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlockType::ReturnData>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlockType::ReturnData BlockType::ReturnData::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlockType::ReturnData>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BlockType::ReturnData>::serialize(const Program::BlockType::ReturnData &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BlockType::ReturnData serde::Deserializable<Program::BlockType::ReturnData>::deserialize(Deserializer &deserializer) {
    Program::BlockType::ReturnData obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligBytecode &lhs, const BrilligBytecode &rhs) {
        if (!(lhs.bytecode == rhs.bytecode)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligBytecode::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligBytecode>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligBytecode BrilligBytecode::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligBytecode>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligBytecode>::serialize(const Program::BrilligBytecode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.bytecode)>::serialize(obj.bytecode, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BrilligBytecode serde::Deserializable<Program::BrilligBytecode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BrilligBytecode obj;
    obj.bytecode = serde::Deserializable<decltype(obj.bytecode)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligInputs &lhs, const BrilligInputs &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligInputs::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligInputs>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligInputs BrilligInputs::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligInputs>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligInputs>::serialize(const Program::BrilligInputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BrilligInputs serde::Deserializable<Program::BrilligInputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BrilligInputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligInputs::Single &lhs, const BrilligInputs::Single &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligInputs::Single::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligInputs::Single>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligInputs::Single BrilligInputs::Single::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligInputs::Single>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligInputs::Single>::serialize(const Program::BrilligInputs::Single &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligInputs::Single serde::Deserializable<Program::BrilligInputs::Single>::deserialize(Deserializer &deserializer) {
    Program::BrilligInputs::Single obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligInputs::Array &lhs, const BrilligInputs::Array &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligInputs::Array::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligInputs::Array>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligInputs::Array BrilligInputs::Array::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligInputs::Array>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligInputs::Array>::serialize(const Program::BrilligInputs::Array &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligInputs::Array serde::Deserializable<Program::BrilligInputs::Array>::deserialize(Deserializer &deserializer) {
    Program::BrilligInputs::Array obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligInputs::MemoryArray &lhs, const BrilligInputs::MemoryArray &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligInputs::MemoryArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligInputs::MemoryArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligInputs::MemoryArray BrilligInputs::MemoryArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligInputs::MemoryArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligInputs::MemoryArray>::serialize(const Program::BrilligInputs::MemoryArray &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligInputs::MemoryArray serde::Deserializable<Program::BrilligInputs::MemoryArray>::deserialize(Deserializer &deserializer) {
    Program::BrilligInputs::MemoryArray obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode &lhs, const BrilligOpcode &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode BrilligOpcode::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode>::serialize(const Program::BrilligOpcode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BrilligOpcode serde::Deserializable<Program::BrilligOpcode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BrilligOpcode obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::BinaryFieldOp &lhs, const BrilligOpcode::BinaryFieldOp &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.op == rhs.op)) { return false; }
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::BinaryFieldOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::BinaryFieldOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::BinaryFieldOp BrilligOpcode::BinaryFieldOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::BinaryFieldOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::BinaryFieldOp>::serialize(const Program::BrilligOpcode::BinaryFieldOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::BinaryFieldOp serde::Deserializable<Program::BrilligOpcode::BinaryFieldOp>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::BinaryFieldOp obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::BinaryIntOp &lhs, const BrilligOpcode::BinaryIntOp &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.op == rhs.op)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        if (!(lhs.lhs == rhs.lhs)) { return false; }
        if (!(lhs.rhs == rhs.rhs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::BinaryIntOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::BinaryIntOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::BinaryIntOp BrilligOpcode::BinaryIntOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::BinaryIntOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::BinaryIntOp>::serialize(const Program::BrilligOpcode::BinaryIntOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::BinaryIntOp serde::Deserializable<Program::BrilligOpcode::BinaryIntOp>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::BinaryIntOp obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Not &lhs, const BrilligOpcode::Not &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source == rhs.source)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Not::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Not>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Not BrilligOpcode::Not::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Not>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Not>::serialize(const Program::BrilligOpcode::Not &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Not serde::Deserializable<Program::BrilligOpcode::Not>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Not obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Cast &lhs, const BrilligOpcode::Cast &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source == rhs.source)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Cast::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Cast>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Cast BrilligOpcode::Cast::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Cast>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Cast>::serialize(const Program::BrilligOpcode::Cast &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Cast serde::Deserializable<Program::BrilligOpcode::Cast>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Cast obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::JumpIfNot &lhs, const BrilligOpcode::JumpIfNot &rhs) {
        if (!(lhs.condition == rhs.condition)) { return false; }
        if (!(lhs.location == rhs.location)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::JumpIfNot::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::JumpIfNot>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::JumpIfNot BrilligOpcode::JumpIfNot::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::JumpIfNot>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::JumpIfNot>::serialize(const Program::BrilligOpcode::JumpIfNot &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.condition)>::serialize(obj.condition, serializer);
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::JumpIfNot serde::Deserializable<Program::BrilligOpcode::JumpIfNot>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::JumpIfNot obj;
    obj.condition = serde::Deserializable<decltype(obj.condition)>::deserialize(deserializer);
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::JumpIf &lhs, const BrilligOpcode::JumpIf &rhs) {
        if (!(lhs.condition == rhs.condition)) { return false; }
        if (!(lhs.location == rhs.location)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::JumpIf::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::JumpIf>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::JumpIf BrilligOpcode::JumpIf::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::JumpIf>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::JumpIf>::serialize(const Program::BrilligOpcode::JumpIf &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.condition)>::serialize(obj.condition, serializer);
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::JumpIf serde::Deserializable<Program::BrilligOpcode::JumpIf>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::JumpIf obj;
    obj.condition = serde::Deserializable<decltype(obj.condition)>::deserialize(deserializer);
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Jump &lhs, const BrilligOpcode::Jump &rhs) {
        if (!(lhs.location == rhs.location)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Jump::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Jump>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Jump BrilligOpcode::Jump::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Jump>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Jump>::serialize(const Program::BrilligOpcode::Jump &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Jump serde::Deserializable<Program::BrilligOpcode::Jump>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Jump obj;
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::CalldataCopy &lhs, const BrilligOpcode::CalldataCopy &rhs) {
        if (!(lhs.destination_address == rhs.destination_address)) { return false; }
        if (!(lhs.size_address == rhs.size_address)) { return false; }
        if (!(lhs.offset_address == rhs.offset_address)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::CalldataCopy::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::CalldataCopy>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::CalldataCopy BrilligOpcode::CalldataCopy::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::CalldataCopy>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::CalldataCopy>::serialize(const Program::BrilligOpcode::CalldataCopy &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination_address)>::serialize(obj.destination_address, serializer);
    serde::Serializable<decltype(obj.size_address)>::serialize(obj.size_address, serializer);
    serde::Serializable<decltype(obj.offset_address)>::serialize(obj.offset_address, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::CalldataCopy serde::Deserializable<Program::BrilligOpcode::CalldataCopy>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::CalldataCopy obj;
    obj.destination_address = serde::Deserializable<decltype(obj.destination_address)>::deserialize(deserializer);
    obj.size_address = serde::Deserializable<decltype(obj.size_address)>::deserialize(deserializer);
    obj.offset_address = serde::Deserializable<decltype(obj.offset_address)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Call &lhs, const BrilligOpcode::Call &rhs) {
        if (!(lhs.location == rhs.location)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Call::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Call>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Call BrilligOpcode::Call::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Call>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Call>::serialize(const Program::BrilligOpcode::Call &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Call serde::Deserializable<Program::BrilligOpcode::Call>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Call obj;
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Const &lhs, const BrilligOpcode::Const &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Const::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Const>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Const BrilligOpcode::Const::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Const>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Const>::serialize(const Program::BrilligOpcode::Const &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Const serde::Deserializable<Program::BrilligOpcode::Const>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Const obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::IndirectConst &lhs, const BrilligOpcode::IndirectConst &rhs) {
        if (!(lhs.destination_pointer == rhs.destination_pointer)) { return false; }
        if (!(lhs.bit_size == rhs.bit_size)) { return false; }
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::IndirectConst::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::IndirectConst>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::IndirectConst BrilligOpcode::IndirectConst::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::IndirectConst>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::IndirectConst>::serialize(const Program::BrilligOpcode::IndirectConst &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination_pointer)>::serialize(obj.destination_pointer, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::IndirectConst serde::Deserializable<Program::BrilligOpcode::IndirectConst>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::IndirectConst obj;
    obj.destination_pointer = serde::Deserializable<decltype(obj.destination_pointer)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Return &lhs, const BrilligOpcode::Return &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Return::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Return>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Return BrilligOpcode::Return::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Return>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Return>::serialize(const Program::BrilligOpcode::Return &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Return serde::Deserializable<Program::BrilligOpcode::Return>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Return obj;
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::ForeignCall &lhs, const BrilligOpcode::ForeignCall &rhs) {
        if (!(lhs.function == rhs.function)) { return false; }
        if (!(lhs.destinations == rhs.destinations)) { return false; }
        if (!(lhs.destination_value_types == rhs.destination_value_types)) { return false; }
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.input_value_types == rhs.input_value_types)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::ForeignCall::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::ForeignCall>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::ForeignCall BrilligOpcode::ForeignCall::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::ForeignCall>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::ForeignCall>::serialize(const Program::BrilligOpcode::ForeignCall &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.function)>::serialize(obj.function, serializer);
    serde::Serializable<decltype(obj.destinations)>::serialize(obj.destinations, serializer);
    serde::Serializable<decltype(obj.destination_value_types)>::serialize(obj.destination_value_types, serializer);
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.input_value_types)>::serialize(obj.input_value_types, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::ForeignCall serde::Deserializable<Program::BrilligOpcode::ForeignCall>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::ForeignCall obj;
    obj.function = serde::Deserializable<decltype(obj.function)>::deserialize(deserializer);
    obj.destinations = serde::Deserializable<decltype(obj.destinations)>::deserialize(deserializer);
    obj.destination_value_types = serde::Deserializable<decltype(obj.destination_value_types)>::deserialize(deserializer);
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.input_value_types = serde::Deserializable<decltype(obj.input_value_types)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Mov &lhs, const BrilligOpcode::Mov &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source == rhs.source)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Mov::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Mov>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Mov BrilligOpcode::Mov::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Mov>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Mov>::serialize(const Program::BrilligOpcode::Mov &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Mov serde::Deserializable<Program::BrilligOpcode::Mov>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Mov obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::ConditionalMov &lhs, const BrilligOpcode::ConditionalMov &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source_a == rhs.source_a)) { return false; }
        if (!(lhs.source_b == rhs.source_b)) { return false; }
        if (!(lhs.condition == rhs.condition)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::ConditionalMov::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::ConditionalMov>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::ConditionalMov BrilligOpcode::ConditionalMov::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::ConditionalMov>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::ConditionalMov>::serialize(const Program::BrilligOpcode::ConditionalMov &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source_a)>::serialize(obj.source_a, serializer);
    serde::Serializable<decltype(obj.source_b)>::serialize(obj.source_b, serializer);
    serde::Serializable<decltype(obj.condition)>::serialize(obj.condition, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::ConditionalMov serde::Deserializable<Program::BrilligOpcode::ConditionalMov>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::ConditionalMov obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source_a = serde::Deserializable<decltype(obj.source_a)>::deserialize(deserializer);
    obj.source_b = serde::Deserializable<decltype(obj.source_b)>::deserialize(deserializer);
    obj.condition = serde::Deserializable<decltype(obj.condition)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Load &lhs, const BrilligOpcode::Load &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
        if (!(lhs.source_pointer == rhs.source_pointer)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Load::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Load>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Load BrilligOpcode::Load::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Load>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Load>::serialize(const Program::BrilligOpcode::Load &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source_pointer)>::serialize(obj.source_pointer, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Load serde::Deserializable<Program::BrilligOpcode::Load>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Load obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source_pointer = serde::Deserializable<decltype(obj.source_pointer)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Store &lhs, const BrilligOpcode::Store &rhs) {
        if (!(lhs.destination_pointer == rhs.destination_pointer)) { return false; }
        if (!(lhs.source == rhs.source)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Store::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Store>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Store BrilligOpcode::Store::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Store>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Store>::serialize(const Program::BrilligOpcode::Store &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination_pointer)>::serialize(obj.destination_pointer, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Store serde::Deserializable<Program::BrilligOpcode::Store>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Store obj;
    obj.destination_pointer = serde::Deserializable<decltype(obj.destination_pointer)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::BlackBox &lhs, const BrilligOpcode::BlackBox &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::BlackBox::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::BlackBox>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::BlackBox BrilligOpcode::BlackBox::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::BlackBox>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::BlackBox>::serialize(const Program::BrilligOpcode::BlackBox &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::BlackBox serde::Deserializable<Program::BrilligOpcode::BlackBox>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::BlackBox obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Trap &lhs, const BrilligOpcode::Trap &rhs) {
        if (!(lhs.revert_data == rhs.revert_data)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Trap::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Trap>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Trap BrilligOpcode::Trap::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Trap>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Trap>::serialize(const Program::BrilligOpcode::Trap &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.revert_data)>::serialize(obj.revert_data, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Trap serde::Deserializable<Program::BrilligOpcode::Trap>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Trap obj;
    obj.revert_data = serde::Deserializable<decltype(obj.revert_data)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOpcode::Stop &lhs, const BrilligOpcode::Stop &rhs) {
        if (!(lhs.return_data == rhs.return_data)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOpcode::Stop::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOpcode::Stop>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOpcode::Stop BrilligOpcode::Stop::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOpcode::Stop>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOpcode::Stop>::serialize(const Program::BrilligOpcode::Stop &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.return_data)>::serialize(obj.return_data, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOpcode::Stop serde::Deserializable<Program::BrilligOpcode::Stop>::deserialize(Deserializer &deserializer) {
    Program::BrilligOpcode::Stop obj;
    obj.return_data = serde::Deserializable<decltype(obj.return_data)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOutputs &lhs, const BrilligOutputs &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOutputs::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOutputs>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOutputs BrilligOutputs::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOutputs>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOutputs>::serialize(const Program::BrilligOutputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::BrilligOutputs serde::Deserializable<Program::BrilligOutputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::BrilligOutputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOutputs::Simple &lhs, const BrilligOutputs::Simple &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOutputs::Simple::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOutputs::Simple>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOutputs::Simple BrilligOutputs::Simple::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOutputs::Simple>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOutputs::Simple>::serialize(const Program::BrilligOutputs::Simple &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOutputs::Simple serde::Deserializable<Program::BrilligOutputs::Simple>::deserialize(Deserializer &deserializer) {
    Program::BrilligOutputs::Simple obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const BrilligOutputs::Array &lhs, const BrilligOutputs::Array &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BrilligOutputs::Array::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BrilligOutputs::Array>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BrilligOutputs::Array BrilligOutputs::Array::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BrilligOutputs::Array>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::BrilligOutputs::Array>::serialize(const Program::BrilligOutputs::Array &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::BrilligOutputs::Array serde::Deserializable<Program::BrilligOutputs::Array>::deserialize(Deserializer &deserializer) {
    Program::BrilligOutputs::Array obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Circuit &lhs, const Circuit &rhs) {
        if (!(lhs.current_witness_index == rhs.current_witness_index)) { return false; }
        if (!(lhs.opcodes == rhs.opcodes)) { return false; }
        if (!(lhs.expression_width == rhs.expression_width)) { return false; }
        if (!(lhs.private_parameters == rhs.private_parameters)) { return false; }
        if (!(lhs.public_parameters == rhs.public_parameters)) { return false; }
        if (!(lhs.return_values == rhs.return_values)) { return false; }
        if (!(lhs.assert_messages == rhs.assert_messages)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Circuit::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Circuit>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Circuit Circuit::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Circuit>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Circuit>::serialize(const Program::Circuit &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.current_witness_index)>::serialize(obj.current_witness_index, serializer);
    serde::Serializable<decltype(obj.opcodes)>::serialize(obj.opcodes, serializer);
    serde::Serializable<decltype(obj.expression_width)>::serialize(obj.expression_width, serializer);
    serde::Serializable<decltype(obj.private_parameters)>::serialize(obj.private_parameters, serializer);
    serde::Serializable<decltype(obj.public_parameters)>::serialize(obj.public_parameters, serializer);
    serde::Serializable<decltype(obj.return_values)>::serialize(obj.return_values, serializer);
    serde::Serializable<decltype(obj.assert_messages)>::serialize(obj.assert_messages, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::Circuit serde::Deserializable<Program::Circuit>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::Circuit obj;
    obj.current_witness_index = serde::Deserializable<decltype(obj.current_witness_index)>::deserialize(deserializer);
    obj.opcodes = serde::Deserializable<decltype(obj.opcodes)>::deserialize(deserializer);
    obj.expression_width = serde::Deserializable<decltype(obj.expression_width)>::deserialize(deserializer);
    obj.private_parameters = serde::Deserializable<decltype(obj.private_parameters)>::deserialize(deserializer);
    obj.public_parameters = serde::Deserializable<decltype(obj.public_parameters)>::deserialize(deserializer);
    obj.return_values = serde::Deserializable<decltype(obj.return_values)>::deserialize(deserializer);
    obj.assert_messages = serde::Deserializable<decltype(obj.assert_messages)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const ConstantOrWitnessEnum &lhs, const ConstantOrWitnessEnum &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ConstantOrWitnessEnum::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ConstantOrWitnessEnum>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ConstantOrWitnessEnum ConstantOrWitnessEnum::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ConstantOrWitnessEnum>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ConstantOrWitnessEnum>::serialize(const Program::ConstantOrWitnessEnum &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::ConstantOrWitnessEnum serde::Deserializable<Program::ConstantOrWitnessEnum>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::ConstantOrWitnessEnum obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const ConstantOrWitnessEnum::Constant &lhs, const ConstantOrWitnessEnum::Constant &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ConstantOrWitnessEnum::Constant::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ConstantOrWitnessEnum::Constant>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ConstantOrWitnessEnum::Constant ConstantOrWitnessEnum::Constant::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ConstantOrWitnessEnum::Constant>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ConstantOrWitnessEnum::Constant>::serialize(const Program::ConstantOrWitnessEnum::Constant &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::ConstantOrWitnessEnum::Constant serde::Deserializable<Program::ConstantOrWitnessEnum::Constant>::deserialize(Deserializer &deserializer) {
    Program::ConstantOrWitnessEnum::Constant obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const ConstantOrWitnessEnum::Witness &lhs, const ConstantOrWitnessEnum::Witness &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ConstantOrWitnessEnum::Witness::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ConstantOrWitnessEnum::Witness>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ConstantOrWitnessEnum::Witness ConstantOrWitnessEnum::Witness::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ConstantOrWitnessEnum::Witness>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ConstantOrWitnessEnum::Witness>::serialize(const Program::ConstantOrWitnessEnum::Witness &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::ConstantOrWitnessEnum::Witness serde::Deserializable<Program::ConstantOrWitnessEnum::Witness>::deserialize(Deserializer &deserializer) {
    Program::ConstantOrWitnessEnum::Witness obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Expression &lhs, const Expression &rhs) {
        if (!(lhs.mul_terms == rhs.mul_terms)) { return false; }
        if (!(lhs.linear_combinations == rhs.linear_combinations)) { return false; }
        if (!(lhs.q_c == rhs.q_c)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Expression::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Expression>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Expression Expression::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Expression>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Expression>::serialize(const Program::Expression &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.mul_terms)>::serialize(obj.mul_terms, serializer);
    serde::Serializable<decltype(obj.linear_combinations)>::serialize(obj.linear_combinations, serializer);
    serde::Serializable<decltype(obj.q_c)>::serialize(obj.q_c, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::Expression serde::Deserializable<Program::Expression>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::Expression obj;
    obj.mul_terms = serde::Deserializable<decltype(obj.mul_terms)>::deserialize(deserializer);
    obj.linear_combinations = serde::Deserializable<decltype(obj.linear_combinations)>::deserialize(deserializer);
    obj.q_c = serde::Deserializable<decltype(obj.q_c)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const ExpressionOrMemory &lhs, const ExpressionOrMemory &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ExpressionOrMemory::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionOrMemory>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionOrMemory ExpressionOrMemory::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionOrMemory>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ExpressionOrMemory>::serialize(const Program::ExpressionOrMemory &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::ExpressionOrMemory serde::Deserializable<Program::ExpressionOrMemory>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::ExpressionOrMemory obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const ExpressionOrMemory::Expression &lhs, const ExpressionOrMemory::Expression &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ExpressionOrMemory::Expression::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionOrMemory::Expression>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionOrMemory::Expression ExpressionOrMemory::Expression::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionOrMemory::Expression>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ExpressionOrMemory::Expression>::serialize(const Program::ExpressionOrMemory::Expression &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::ExpressionOrMemory::Expression serde::Deserializable<Program::ExpressionOrMemory::Expression>::deserialize(Deserializer &deserializer) {
    Program::ExpressionOrMemory::Expression obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const ExpressionOrMemory::Memory &lhs, const ExpressionOrMemory::Memory &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ExpressionOrMemory::Memory::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionOrMemory::Memory>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionOrMemory::Memory ExpressionOrMemory::Memory::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionOrMemory::Memory>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ExpressionOrMemory::Memory>::serialize(const Program::ExpressionOrMemory::Memory &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::ExpressionOrMemory::Memory serde::Deserializable<Program::ExpressionOrMemory::Memory>::deserialize(Deserializer &deserializer) {
    Program::ExpressionOrMemory::Memory obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const ExpressionWidth &lhs, const ExpressionWidth &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ExpressionWidth::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionWidth>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionWidth ExpressionWidth::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionWidth>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ExpressionWidth>::serialize(const Program::ExpressionWidth &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::ExpressionWidth serde::Deserializable<Program::ExpressionWidth>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::ExpressionWidth obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const ExpressionWidth::Unbounded &lhs, const ExpressionWidth::Unbounded &rhs) {
        return true;
    }

    inline std::vector<uint8_t> ExpressionWidth::Unbounded::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionWidth::Unbounded>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionWidth::Unbounded ExpressionWidth::Unbounded::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionWidth::Unbounded>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ExpressionWidth::Unbounded>::serialize(const Program::ExpressionWidth::Unbounded &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::ExpressionWidth::Unbounded serde::Deserializable<Program::ExpressionWidth::Unbounded>::deserialize(Deserializer &deserializer) {
    Program::ExpressionWidth::Unbounded obj;
    return obj;
}

namespace Program {

    inline bool operator==(const ExpressionWidth::Bounded &lhs, const ExpressionWidth::Bounded &rhs) {
        if (!(lhs.width == rhs.width)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ExpressionWidth::Bounded::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ExpressionWidth::Bounded>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ExpressionWidth::Bounded ExpressionWidth::Bounded::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ExpressionWidth::Bounded>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ExpressionWidth::Bounded>::serialize(const Program::ExpressionWidth::Bounded &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.width)>::serialize(obj.width, serializer);
}

template <>
template <typename Deserializer>
Program::ExpressionWidth::Bounded serde::Deserializable<Program::ExpressionWidth::Bounded>::deserialize(Deserializer &deserializer) {
    Program::ExpressionWidth::Bounded obj;
    obj.width = serde::Deserializable<decltype(obj.width)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const FunctionInput &lhs, const FunctionInput &rhs) {
        if (!(lhs.input == rhs.input)) { return false; }
        if (!(lhs.num_bits == rhs.num_bits)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> FunctionInput::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<FunctionInput>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline FunctionInput FunctionInput::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<FunctionInput>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::FunctionInput>::serialize(const Program::FunctionInput &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
    serde::Serializable<decltype(obj.num_bits)>::serialize(obj.num_bits, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::FunctionInput serde::Deserializable<Program::FunctionInput>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::FunctionInput obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    obj.num_bits = serde::Deserializable<decltype(obj.num_bits)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const HeapArray &lhs, const HeapArray &rhs) {
        if (!(lhs.pointer == rhs.pointer)) { return false; }
        if (!(lhs.size == rhs.size)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> HeapArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapArray HeapArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::HeapArray>::serialize(const Program::HeapArray &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.pointer)>::serialize(obj.pointer, serializer);
    serde::Serializable<decltype(obj.size)>::serialize(obj.size, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::HeapArray serde::Deserializable<Program::HeapArray>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::HeapArray obj;
    obj.pointer = serde::Deserializable<decltype(obj.pointer)>::deserialize(deserializer);
    obj.size = serde::Deserializable<decltype(obj.size)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const HeapValueType &lhs, const HeapValueType &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> HeapValueType::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapValueType>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapValueType HeapValueType::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapValueType>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::HeapValueType>::serialize(const Program::HeapValueType &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::HeapValueType serde::Deserializable<Program::HeapValueType>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::HeapValueType obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const HeapValueType::Simple &lhs, const HeapValueType::Simple &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> HeapValueType::Simple::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapValueType::Simple>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapValueType::Simple HeapValueType::Simple::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapValueType::Simple>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::HeapValueType::Simple>::serialize(const Program::HeapValueType::Simple &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::HeapValueType::Simple serde::Deserializable<Program::HeapValueType::Simple>::deserialize(Deserializer &deserializer) {
    Program::HeapValueType::Simple obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const HeapValueType::Array &lhs, const HeapValueType::Array &rhs) {
        if (!(lhs.value_types == rhs.value_types)) { return false; }
        if (!(lhs.size == rhs.size)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> HeapValueType::Array::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapValueType::Array>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapValueType::Array HeapValueType::Array::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapValueType::Array>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::HeapValueType::Array>::serialize(const Program::HeapValueType::Array &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value_types)>::serialize(obj.value_types, serializer);
    serde::Serializable<decltype(obj.size)>::serialize(obj.size, serializer);
}

template <>
template <typename Deserializer>
Program::HeapValueType::Array serde::Deserializable<Program::HeapValueType::Array>::deserialize(Deserializer &deserializer) {
    Program::HeapValueType::Array obj;
    obj.value_types = serde::Deserializable<decltype(obj.value_types)>::deserialize(deserializer);
    obj.size = serde::Deserializable<decltype(obj.size)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const HeapValueType::Vector &lhs, const HeapValueType::Vector &rhs) {
        if (!(lhs.value_types == rhs.value_types)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> HeapValueType::Vector::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapValueType::Vector>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapValueType::Vector HeapValueType::Vector::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapValueType::Vector>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::HeapValueType::Vector>::serialize(const Program::HeapValueType::Vector &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value_types)>::serialize(obj.value_types, serializer);
}

template <>
template <typename Deserializer>
Program::HeapValueType::Vector serde::Deserializable<Program::HeapValueType::Vector>::deserialize(Deserializer &deserializer) {
    Program::HeapValueType::Vector obj;
    obj.value_types = serde::Deserializable<decltype(obj.value_types)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const HeapVector &lhs, const HeapVector &rhs) {
        if (!(lhs.pointer == rhs.pointer)) { return false; }
        if (!(lhs.size == rhs.size)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> HeapVector::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<HeapVector>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline HeapVector HeapVector::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<HeapVector>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::HeapVector>::serialize(const Program::HeapVector &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.pointer)>::serialize(obj.pointer, serializer);
    serde::Serializable<decltype(obj.size)>::serialize(obj.size, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::HeapVector serde::Deserializable<Program::HeapVector>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::HeapVector obj;
    obj.pointer = serde::Deserializable<decltype(obj.pointer)>::deserialize(deserializer);
    obj.size = serde::Deserializable<decltype(obj.size)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const IntegerBitSize &lhs, const IntegerBitSize &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> IntegerBitSize::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize IntegerBitSize::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::IntegerBitSize>::serialize(const Program::IntegerBitSize &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::IntegerBitSize serde::Deserializable<Program::IntegerBitSize>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::IntegerBitSize obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const IntegerBitSize::U1 &lhs, const IntegerBitSize::U1 &rhs) {
        return true;
    }

    inline std::vector<uint8_t> IntegerBitSize::U1::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U1>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U1 IntegerBitSize::U1::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U1>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::IntegerBitSize::U1>::serialize(const Program::IntegerBitSize::U1 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::IntegerBitSize::U1 serde::Deserializable<Program::IntegerBitSize::U1>::deserialize(Deserializer &deserializer) {
    Program::IntegerBitSize::U1 obj;
    return obj;
}

namespace Program {

    inline bool operator==(const IntegerBitSize::U8 &lhs, const IntegerBitSize::U8 &rhs) {
        return true;
    }

    inline std::vector<uint8_t> IntegerBitSize::U8::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U8>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U8 IntegerBitSize::U8::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U8>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::IntegerBitSize::U8>::serialize(const Program::IntegerBitSize::U8 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::IntegerBitSize::U8 serde::Deserializable<Program::IntegerBitSize::U8>::deserialize(Deserializer &deserializer) {
    Program::IntegerBitSize::U8 obj;
    return obj;
}

namespace Program {

    inline bool operator==(const IntegerBitSize::U16 &lhs, const IntegerBitSize::U16 &rhs) {
        return true;
    }

    inline std::vector<uint8_t> IntegerBitSize::U16::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U16>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U16 IntegerBitSize::U16::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U16>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::IntegerBitSize::U16>::serialize(const Program::IntegerBitSize::U16 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::IntegerBitSize::U16 serde::Deserializable<Program::IntegerBitSize::U16>::deserialize(Deserializer &deserializer) {
    Program::IntegerBitSize::U16 obj;
    return obj;
}

namespace Program {

    inline bool operator==(const IntegerBitSize::U32 &lhs, const IntegerBitSize::U32 &rhs) {
        return true;
    }

    inline std::vector<uint8_t> IntegerBitSize::U32::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U32>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U32 IntegerBitSize::U32::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U32>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::IntegerBitSize::U32>::serialize(const Program::IntegerBitSize::U32 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::IntegerBitSize::U32 serde::Deserializable<Program::IntegerBitSize::U32>::deserialize(Deserializer &deserializer) {
    Program::IntegerBitSize::U32 obj;
    return obj;
}

namespace Program {

    inline bool operator==(const IntegerBitSize::U64 &lhs, const IntegerBitSize::U64 &rhs) {
        return true;
    }

    inline std::vector<uint8_t> IntegerBitSize::U64::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U64>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U64 IntegerBitSize::U64::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U64>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::IntegerBitSize::U64>::serialize(const Program::IntegerBitSize::U64 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::IntegerBitSize::U64 serde::Deserializable<Program::IntegerBitSize::U64>::deserialize(Deserializer &deserializer) {
    Program::IntegerBitSize::U64 obj;
    return obj;
}

namespace Program {

    inline bool operator==(const IntegerBitSize::U128 &lhs, const IntegerBitSize::U128 &rhs) {
        return true;
    }

    inline std::vector<uint8_t> IntegerBitSize::U128::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<IntegerBitSize::U128>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline IntegerBitSize::U128 IntegerBitSize::U128::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<IntegerBitSize::U128>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::IntegerBitSize::U128>::serialize(const Program::IntegerBitSize::U128 &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Program::IntegerBitSize::U128 serde::Deserializable<Program::IntegerBitSize::U128>::deserialize(Deserializer &deserializer) {
    Program::IntegerBitSize::U128 obj;
    return obj;
}

namespace Program {

    inline bool operator==(const MemOp &lhs, const MemOp &rhs) {
        if (!(lhs.operation == rhs.operation)) { return false; }
        if (!(lhs.index == rhs.index)) { return false; }
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> MemOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<MemOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline MemOp MemOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<MemOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::MemOp>::serialize(const Program::MemOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.operation)>::serialize(obj.operation, serializer);
    serde::Serializable<decltype(obj.index)>::serialize(obj.index, serializer);
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::MemOp serde::Deserializable<Program::MemOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::MemOp obj;
    obj.operation = serde::Deserializable<decltype(obj.operation)>::deserialize(deserializer);
    obj.index = serde::Deserializable<decltype(obj.index)>::deserialize(deserializer);
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const MemoryAddress &lhs, const MemoryAddress &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> MemoryAddress::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<MemoryAddress>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline MemoryAddress MemoryAddress::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<MemoryAddress>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::MemoryAddress>::serialize(const Program::MemoryAddress &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::MemoryAddress serde::Deserializable<Program::MemoryAddress>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::MemoryAddress obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const MemoryAddress::Direct &lhs, const MemoryAddress::Direct &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> MemoryAddress::Direct::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<MemoryAddress::Direct>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline MemoryAddress::Direct MemoryAddress::Direct::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<MemoryAddress::Direct>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::MemoryAddress::Direct>::serialize(const Program::MemoryAddress::Direct &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::MemoryAddress::Direct serde::Deserializable<Program::MemoryAddress::Direct>::deserialize(Deserializer &deserializer) {
    Program::MemoryAddress::Direct obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const MemoryAddress::Relative &lhs, const MemoryAddress::Relative &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> MemoryAddress::Relative::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<MemoryAddress::Relative>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline MemoryAddress::Relative MemoryAddress::Relative::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<MemoryAddress::Relative>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::MemoryAddress::Relative>::serialize(const Program::MemoryAddress::Relative &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::MemoryAddress::Relative serde::Deserializable<Program::MemoryAddress::Relative>::deserialize(Deserializer &deserializer) {
    Program::MemoryAddress::Relative obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Opcode &lhs, const Opcode &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode Opcode::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Opcode>::serialize(const Program::Opcode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::Opcode serde::Deserializable<Program::Opcode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::Opcode obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const Opcode::AssertZero &lhs, const Opcode::AssertZero &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::AssertZero::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::AssertZero>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::AssertZero Opcode::AssertZero::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::AssertZero>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Opcode::AssertZero>::serialize(const Program::Opcode::AssertZero &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::Opcode::AssertZero serde::Deserializable<Program::Opcode::AssertZero>::deserialize(Deserializer &deserializer) {
    Program::Opcode::AssertZero obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Opcode::BlackBoxFuncCall &lhs, const Opcode::BlackBoxFuncCall &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::BlackBoxFuncCall::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::BlackBoxFuncCall>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::BlackBoxFuncCall Opcode::BlackBoxFuncCall::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::BlackBoxFuncCall>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Opcode::BlackBoxFuncCall>::serialize(const Program::Opcode::BlackBoxFuncCall &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::Opcode::BlackBoxFuncCall serde::Deserializable<Program::Opcode::BlackBoxFuncCall>::deserialize(Deserializer &deserializer) {
    Program::Opcode::BlackBoxFuncCall obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Opcode::MemoryOp &lhs, const Opcode::MemoryOp &rhs) {
        if (!(lhs.block_id == rhs.block_id)) { return false; }
        if (!(lhs.op == rhs.op)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::MemoryOp::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::MemoryOp>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::MemoryOp Opcode::MemoryOp::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::MemoryOp>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Opcode::MemoryOp>::serialize(const Program::Opcode::MemoryOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.block_id)>::serialize(obj.block_id, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
}

template <>
template <typename Deserializer>
Program::Opcode::MemoryOp serde::Deserializable<Program::Opcode::MemoryOp>::deserialize(Deserializer &deserializer) {
    Program::Opcode::MemoryOp obj;
    obj.block_id = serde::Deserializable<decltype(obj.block_id)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Opcode::MemoryInit &lhs, const Opcode::MemoryInit &rhs) {
        if (!(lhs.block_id == rhs.block_id)) { return false; }
        if (!(lhs.init == rhs.init)) { return false; }
        if (!(lhs.block_type == rhs.block_type)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::MemoryInit::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::MemoryInit>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::MemoryInit Opcode::MemoryInit::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::MemoryInit>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Opcode::MemoryInit>::serialize(const Program::Opcode::MemoryInit &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.block_id)>::serialize(obj.block_id, serializer);
    serde::Serializable<decltype(obj.init)>::serialize(obj.init, serializer);
    serde::Serializable<decltype(obj.block_type)>::serialize(obj.block_type, serializer);
}

template <>
template <typename Deserializer>
Program::Opcode::MemoryInit serde::Deserializable<Program::Opcode::MemoryInit>::deserialize(Deserializer &deserializer) {
    Program::Opcode::MemoryInit obj;
    obj.block_id = serde::Deserializable<decltype(obj.block_id)>::deserialize(deserializer);
    obj.init = serde::Deserializable<decltype(obj.init)>::deserialize(deserializer);
    obj.block_type = serde::Deserializable<decltype(obj.block_type)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Opcode::BrilligCall &lhs, const Opcode::BrilligCall &rhs) {
        if (!(lhs.id == rhs.id)) { return false; }
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::BrilligCall::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::BrilligCall>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::BrilligCall Opcode::BrilligCall::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::BrilligCall>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Opcode::BrilligCall>::serialize(const Program::Opcode::BrilligCall &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.id)>::serialize(obj.id, serializer);
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
}

template <>
template <typename Deserializer>
Program::Opcode::BrilligCall serde::Deserializable<Program::Opcode::BrilligCall>::deserialize(Deserializer &deserializer) {
    Program::Opcode::BrilligCall obj;
    obj.id = serde::Deserializable<decltype(obj.id)>::deserialize(deserializer);
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Opcode::Call &lhs, const Opcode::Call &rhs) {
        if (!(lhs.id == rhs.id)) { return false; }
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::Call::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::Call>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::Call Opcode::Call::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::Call>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Opcode::Call>::serialize(const Program::Opcode::Call &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.id)>::serialize(obj.id, serializer);
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
}

template <>
template <typename Deserializer>
Program::Opcode::Call serde::Deserializable<Program::Opcode::Call>::deserialize(Deserializer &deserializer) {
    Program::Opcode::Call obj;
    obj.id = serde::Deserializable<decltype(obj.id)>::deserialize(deserializer);
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const OpcodeLocation &lhs, const OpcodeLocation &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> OpcodeLocation::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<OpcodeLocation>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline OpcodeLocation OpcodeLocation::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<OpcodeLocation>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::OpcodeLocation>::serialize(const Program::OpcodeLocation &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::OpcodeLocation serde::Deserializable<Program::OpcodeLocation>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::OpcodeLocation obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const OpcodeLocation::Acir &lhs, const OpcodeLocation::Acir &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> OpcodeLocation::Acir::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<OpcodeLocation::Acir>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline OpcodeLocation::Acir OpcodeLocation::Acir::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<OpcodeLocation::Acir>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::OpcodeLocation::Acir>::serialize(const Program::OpcodeLocation::Acir &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::OpcodeLocation::Acir serde::Deserializable<Program::OpcodeLocation::Acir>::deserialize(Deserializer &deserializer) {
    Program::OpcodeLocation::Acir obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const OpcodeLocation::Brillig &lhs, const OpcodeLocation::Brillig &rhs) {
        if (!(lhs.acir_index == rhs.acir_index)) { return false; }
        if (!(lhs.brillig_index == rhs.brillig_index)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> OpcodeLocation::Brillig::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<OpcodeLocation::Brillig>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline OpcodeLocation::Brillig OpcodeLocation::Brillig::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<OpcodeLocation::Brillig>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::OpcodeLocation::Brillig>::serialize(const Program::OpcodeLocation::Brillig &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.acir_index)>::serialize(obj.acir_index, serializer);
    serde::Serializable<decltype(obj.brillig_index)>::serialize(obj.brillig_index, serializer);
}

template <>
template <typename Deserializer>
Program::OpcodeLocation::Brillig serde::Deserializable<Program::OpcodeLocation::Brillig>::deserialize(Deserializer &deserializer) {
    Program::OpcodeLocation::Brillig obj;
    obj.acir_index = serde::Deserializable<decltype(obj.acir_index)>::deserialize(deserializer);
    obj.brillig_index = serde::Deserializable<decltype(obj.brillig_index)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Program &lhs, const Program &rhs) {
        if (!(lhs.functions == rhs.functions)) { return false; }
        if (!(lhs.unconstrained_functions == rhs.unconstrained_functions)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Program::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Program>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Program Program::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Program>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Program>::serialize(const Program::Program &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.functions)>::serialize(obj.functions, serializer);
    serde::Serializable<decltype(obj.unconstrained_functions)>::serialize(obj.unconstrained_functions, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::Program serde::Deserializable<Program::Program>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::Program obj;
    obj.functions = serde::Deserializable<decltype(obj.functions)>::deserialize(deserializer);
    obj.unconstrained_functions = serde::Deserializable<decltype(obj.unconstrained_functions)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const PublicInputs &lhs, const PublicInputs &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> PublicInputs::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<PublicInputs>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline PublicInputs PublicInputs::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<PublicInputs>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::PublicInputs>::serialize(const Program::PublicInputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::PublicInputs serde::Deserializable<Program::PublicInputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::PublicInputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const ValueOrArray &lhs, const ValueOrArray &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ValueOrArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ValueOrArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ValueOrArray ValueOrArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ValueOrArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ValueOrArray>::serialize(const Program::ValueOrArray &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::ValueOrArray serde::Deserializable<Program::ValueOrArray>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::ValueOrArray obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Program {

    inline bool operator==(const ValueOrArray::MemoryAddress &lhs, const ValueOrArray::MemoryAddress &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ValueOrArray::MemoryAddress::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ValueOrArray::MemoryAddress>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ValueOrArray::MemoryAddress ValueOrArray::MemoryAddress::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ValueOrArray::MemoryAddress>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ValueOrArray::MemoryAddress>::serialize(const Program::ValueOrArray::MemoryAddress &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::ValueOrArray::MemoryAddress serde::Deserializable<Program::ValueOrArray::MemoryAddress>::deserialize(Deserializer &deserializer) {
    Program::ValueOrArray::MemoryAddress obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const ValueOrArray::HeapArray &lhs, const ValueOrArray::HeapArray &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ValueOrArray::HeapArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ValueOrArray::HeapArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ValueOrArray::HeapArray ValueOrArray::HeapArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ValueOrArray::HeapArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ValueOrArray::HeapArray>::serialize(const Program::ValueOrArray::HeapArray &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::ValueOrArray::HeapArray serde::Deserializable<Program::ValueOrArray::HeapArray>::deserialize(Deserializer &deserializer) {
    Program::ValueOrArray::HeapArray obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const ValueOrArray::HeapVector &lhs, const ValueOrArray::HeapVector &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> ValueOrArray::HeapVector::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<ValueOrArray::HeapVector>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline ValueOrArray::HeapVector ValueOrArray::HeapVector::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<ValueOrArray::HeapVector>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::ValueOrArray::HeapVector>::serialize(const Program::ValueOrArray::HeapVector &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Program::ValueOrArray::HeapVector serde::Deserializable<Program::ValueOrArray::HeapVector>::deserialize(Deserializer &deserializer) {
    Program::ValueOrArray::HeapVector obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Program {

    inline bool operator==(const Witness &lhs, const Witness &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Witness::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Witness>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Witness Witness::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Witness>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Program

template <>
template <typename Serializer>
void serde::Serializable<Program::Witness>::serialize(const Program::Witness &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Program::Witness serde::Deserializable<Program::Witness>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Program::Witness obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}
