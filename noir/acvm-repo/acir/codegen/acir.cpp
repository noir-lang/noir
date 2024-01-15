#pragma once

#include "serde.hpp"
#include "bincode.hpp"

namespace Circuit {

    struct Witness {
        uint32_t value;

        friend bool operator==(const Witness&, const Witness&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Witness bincodeDeserialize(std::vector<uint8_t>);
    };

    struct FunctionInput {
        Circuit::Witness witness;
        uint32_t num_bits;

        friend bool operator==(const FunctionInput&, const FunctionInput&);
        std::vector<uint8_t> bincodeSerialize() const;
        static FunctionInput bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BlackBoxFuncCall {

        struct AND {
            Circuit::FunctionInput lhs;
            Circuit::FunctionInput rhs;
            Circuit::Witness output;

            friend bool operator==(const AND&, const AND&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AND bincodeDeserialize(std::vector<uint8_t>);
        };

        struct XOR {
            Circuit::FunctionInput lhs;
            Circuit::FunctionInput rhs;
            Circuit::Witness output;

            friend bool operator==(const XOR&, const XOR&);
            std::vector<uint8_t> bincodeSerialize() const;
            static XOR bincodeDeserialize(std::vector<uint8_t>);
        };

        struct RANGE {
            Circuit::FunctionInput input;

            friend bool operator==(const RANGE&, const RANGE&);
            std::vector<uint8_t> bincodeSerialize() const;
            static RANGE bincodeDeserialize(std::vector<uint8_t>);
        };

        struct SHA256 {
            std::vector<Circuit::FunctionInput> inputs;
            std::vector<Circuit::Witness> outputs;

            friend bool operator==(const SHA256&, const SHA256&);
            std::vector<uint8_t> bincodeSerialize() const;
            static SHA256 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Blake2s {
            std::vector<Circuit::FunctionInput> inputs;
            std::vector<Circuit::Witness> outputs;

            friend bool operator==(const Blake2s&, const Blake2s&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake2s bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Blake3 {
            std::vector<Circuit::FunctionInput> inputs;
            std::vector<Circuit::Witness> outputs;

            friend bool operator==(const Blake3&, const Blake3&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake3 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct SchnorrVerify {
            Circuit::FunctionInput public_key_x;
            Circuit::FunctionInput public_key_y;
            std::vector<Circuit::FunctionInput> signature;
            std::vector<Circuit::FunctionInput> message;
            Circuit::Witness output;

            friend bool operator==(const SchnorrVerify&, const SchnorrVerify&);
            std::vector<uint8_t> bincodeSerialize() const;
            static SchnorrVerify bincodeDeserialize(std::vector<uint8_t>);
        };

        struct PedersenCommitment {
            std::vector<Circuit::FunctionInput> inputs;
            uint32_t domain_separator;
            std::array<Circuit::Witness, 2> outputs;

            friend bool operator==(const PedersenCommitment&, const PedersenCommitment&);
            std::vector<uint8_t> bincodeSerialize() const;
            static PedersenCommitment bincodeDeserialize(std::vector<uint8_t>);
        };

        struct PedersenHash {
            std::vector<Circuit::FunctionInput> inputs;
            uint32_t domain_separator;
            Circuit::Witness output;

            friend bool operator==(const PedersenHash&, const PedersenHash&);
            std::vector<uint8_t> bincodeSerialize() const;
            static PedersenHash bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EcdsaSecp256k1 {
            std::vector<Circuit::FunctionInput> public_key_x;
            std::vector<Circuit::FunctionInput> public_key_y;
            std::vector<Circuit::FunctionInput> signature;
            std::vector<Circuit::FunctionInput> hashed_message;
            Circuit::Witness output;

            friend bool operator==(const EcdsaSecp256k1&, const EcdsaSecp256k1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256k1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EcdsaSecp256r1 {
            std::vector<Circuit::FunctionInput> public_key_x;
            std::vector<Circuit::FunctionInput> public_key_y;
            std::vector<Circuit::FunctionInput> signature;
            std::vector<Circuit::FunctionInput> hashed_message;
            Circuit::Witness output;

            friend bool operator==(const EcdsaSecp256r1&, const EcdsaSecp256r1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256r1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct FixedBaseScalarMul {
            Circuit::FunctionInput low;
            Circuit::FunctionInput high;
            std::array<Circuit::Witness, 2> outputs;

            friend bool operator==(const FixedBaseScalarMul&, const FixedBaseScalarMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static FixedBaseScalarMul bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EmbeddedCurveAdd {
            Circuit::FunctionInput input1_x;
            Circuit::FunctionInput input1_y;
            Circuit::FunctionInput input2_x;
            Circuit::FunctionInput input2_y;
            std::array<Circuit::Witness, 2> outputs;

            friend bool operator==(const EmbeddedCurveAdd&, const EmbeddedCurveAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EmbeddedCurveAdd bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EmbeddedCurveDouble {
            Circuit::FunctionInput input_x;
            Circuit::FunctionInput input_y;
            std::array<Circuit::Witness, 2> outputs;

            friend bool operator==(const EmbeddedCurveDouble&, const EmbeddedCurveDouble&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EmbeddedCurveDouble bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Keccak256 {
            std::vector<Circuit::FunctionInput> inputs;
            std::vector<Circuit::Witness> outputs;

            friend bool operator==(const Keccak256&, const Keccak256&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccak256 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Keccak256VariableLength {
            std::vector<Circuit::FunctionInput> inputs;
            Circuit::FunctionInput var_message_size;
            std::vector<Circuit::Witness> outputs;

            friend bool operator==(const Keccak256VariableLength&, const Keccak256VariableLength&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccak256VariableLength bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Keccakf1600 {
            std::vector<Circuit::FunctionInput> inputs;
            std::vector<Circuit::Witness> outputs;

            friend bool operator==(const Keccakf1600&, const Keccakf1600&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccakf1600 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct RecursiveAggregation {
            std::vector<Circuit::FunctionInput> verification_key;
            std::vector<Circuit::FunctionInput> proof;
            std::vector<Circuit::FunctionInput> public_inputs;
            Circuit::FunctionInput key_hash;

            friend bool operator==(const RecursiveAggregation&, const RecursiveAggregation&);
            std::vector<uint8_t> bincodeSerialize() const;
            static RecursiveAggregation bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<AND, XOR, RANGE, SHA256, Blake2s, Blake3, SchnorrVerify, PedersenCommitment, PedersenHash, EcdsaSecp256k1, EcdsaSecp256r1, FixedBaseScalarMul, EmbeddedCurveAdd, EmbeddedCurveDouble, Keccak256, Keccak256VariableLength, Keccakf1600, RecursiveAggregation> value;

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

    struct Expression {
        std::vector<std::tuple<std::string, Circuit::Witness, Circuit::Witness>> mul_terms;
        std::vector<std::tuple<std::string, Circuit::Witness>> linear_combinations;
        std::string q_c;

        friend bool operator==(const Expression&, const Expression&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Expression bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BrilligInputs {

        struct Single {
            Circuit::Expression value;

            friend bool operator==(const Single&, const Single&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Single bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Array {
            std::vector<Circuit::Expression> value;

            friend bool operator==(const Array&, const Array&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Array bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Single, Array> value;

        friend bool operator==(const BrilligInputs&, const BrilligInputs&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligInputs bincodeDeserialize(std::vector<uint8_t>);
    };

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

        struct Equals {
            friend bool operator==(const Equals&, const Equals&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Equals bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Add, Sub, Mul, Div, Equals> value;

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

        struct SignedDiv {
            friend bool operator==(const SignedDiv&, const SignedDiv&);
            std::vector<uint8_t> bincodeSerialize() const;
            static SignedDiv bincodeDeserialize(std::vector<uint8_t>);
        };

        struct UnsignedDiv {
            friend bool operator==(const UnsignedDiv&, const UnsignedDiv&);
            std::vector<uint8_t> bincodeSerialize() const;
            static UnsignedDiv bincodeDeserialize(std::vector<uint8_t>);
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

        std::variant<Add, Sub, Mul, SignedDiv, UnsignedDiv, Equals, LessThan, LessThanEquals, And, Or, Xor, Shl, Shr> value;

        friend bool operator==(const BinaryIntOp&, const BinaryIntOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BinaryIntOp bincodeDeserialize(std::vector<uint8_t>);
    };

    struct RegisterIndex {
        uint64_t value;

        friend bool operator==(const RegisterIndex&, const RegisterIndex&);
        std::vector<uint8_t> bincodeSerialize() const;
        static RegisterIndex bincodeDeserialize(std::vector<uint8_t>);
    };

    struct HeapArray {
        Circuit::RegisterIndex pointer;
        uint64_t size;

        friend bool operator==(const HeapArray&, const HeapArray&);
        std::vector<uint8_t> bincodeSerialize() const;
        static HeapArray bincodeDeserialize(std::vector<uint8_t>);
    };

    struct HeapVector {
        Circuit::RegisterIndex pointer;
        Circuit::RegisterIndex size;

        friend bool operator==(const HeapVector&, const HeapVector&);
        std::vector<uint8_t> bincodeSerialize() const;
        static HeapVector bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BlackBoxOp {

        struct Sha256 {
            Circuit::HeapVector message;
            Circuit::HeapArray output;

            friend bool operator==(const Sha256&, const Sha256&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Sha256 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Blake2s {
            Circuit::HeapVector message;
            Circuit::HeapArray output;

            friend bool operator==(const Blake2s&, const Blake2s&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake2s bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Blake3 {
            Circuit::HeapVector message;
            Circuit::HeapArray output;

            friend bool operator==(const Blake3&, const Blake3&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Blake3 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Keccak256 {
            Circuit::HeapVector message;
            Circuit::HeapArray output;

            friend bool operator==(const Keccak256&, const Keccak256&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccak256 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Keccakf1600 {
            Circuit::HeapVector message;
            Circuit::HeapArray output;

            friend bool operator==(const Keccakf1600&, const Keccakf1600&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Keccakf1600 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EcdsaSecp256k1 {
            Circuit::HeapVector hashed_msg;
            Circuit::HeapArray public_key_x;
            Circuit::HeapArray public_key_y;
            Circuit::HeapArray signature;
            Circuit::RegisterIndex result;

            friend bool operator==(const EcdsaSecp256k1&, const EcdsaSecp256k1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256k1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EcdsaSecp256r1 {
            Circuit::HeapVector hashed_msg;
            Circuit::HeapArray public_key_x;
            Circuit::HeapArray public_key_y;
            Circuit::HeapArray signature;
            Circuit::RegisterIndex result;

            friend bool operator==(const EcdsaSecp256r1&, const EcdsaSecp256r1&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EcdsaSecp256r1 bincodeDeserialize(std::vector<uint8_t>);
        };

        struct SchnorrVerify {
            Circuit::RegisterIndex public_key_x;
            Circuit::RegisterIndex public_key_y;
            Circuit::HeapVector message;
            Circuit::HeapVector signature;
            Circuit::RegisterIndex result;

            friend bool operator==(const SchnorrVerify&, const SchnorrVerify&);
            std::vector<uint8_t> bincodeSerialize() const;
            static SchnorrVerify bincodeDeserialize(std::vector<uint8_t>);
        };

        struct PedersenCommitment {
            Circuit::HeapVector inputs;
            Circuit::RegisterIndex domain_separator;
            Circuit::HeapArray output;

            friend bool operator==(const PedersenCommitment&, const PedersenCommitment&);
            std::vector<uint8_t> bincodeSerialize() const;
            static PedersenCommitment bincodeDeserialize(std::vector<uint8_t>);
        };

        struct PedersenHash {
            Circuit::HeapVector inputs;
            Circuit::RegisterIndex domain_separator;
            Circuit::RegisterIndex output;

            friend bool operator==(const PedersenHash&, const PedersenHash&);
            std::vector<uint8_t> bincodeSerialize() const;
            static PedersenHash bincodeDeserialize(std::vector<uint8_t>);
        };

        struct FixedBaseScalarMul {
            Circuit::RegisterIndex low;
            Circuit::RegisterIndex high;
            Circuit::HeapArray result;

            friend bool operator==(const FixedBaseScalarMul&, const FixedBaseScalarMul&);
            std::vector<uint8_t> bincodeSerialize() const;
            static FixedBaseScalarMul bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EmbeddedCurveAdd {
            Circuit::RegisterIndex input1_x;
            Circuit::RegisterIndex input1_y;
            Circuit::RegisterIndex input2_x;
            Circuit::RegisterIndex input2_y;
            Circuit::HeapArray result;

            friend bool operator==(const EmbeddedCurveAdd&, const EmbeddedCurveAdd&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EmbeddedCurveAdd bincodeDeserialize(std::vector<uint8_t>);
        };

        struct EmbeddedCurveDouble {
            Circuit::RegisterIndex input1_x;
            Circuit::RegisterIndex input1_y;
            Circuit::HeapArray result;

            friend bool operator==(const EmbeddedCurveDouble&, const EmbeddedCurveDouble&);
            std::vector<uint8_t> bincodeSerialize() const;
            static EmbeddedCurveDouble bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Sha256, Blake2s, Blake3, Keccak256, Keccakf1600, EcdsaSecp256k1, EcdsaSecp256r1, SchnorrVerify, PedersenCommitment, PedersenHash, FixedBaseScalarMul, EmbeddedCurveAdd, EmbeddedCurveDouble> value;

        friend bool operator==(const BlackBoxOp&, const BlackBoxOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BlackBoxOp bincodeDeserialize(std::vector<uint8_t>);
    };

    struct RegisterOrMemory {

        struct RegisterIndex {
            Circuit::RegisterIndex value;

            friend bool operator==(const RegisterIndex&, const RegisterIndex&);
            std::vector<uint8_t> bincodeSerialize() const;
            static RegisterIndex bincodeDeserialize(std::vector<uint8_t>);
        };

        struct HeapArray {
            Circuit::HeapArray value;

            friend bool operator==(const HeapArray&, const HeapArray&);
            std::vector<uint8_t> bincodeSerialize() const;
            static HeapArray bincodeDeserialize(std::vector<uint8_t>);
        };

        struct HeapVector {
            Circuit::HeapVector value;

            friend bool operator==(const HeapVector&, const HeapVector&);
            std::vector<uint8_t> bincodeSerialize() const;
            static HeapVector bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<RegisterIndex, HeapArray, HeapVector> value;

        friend bool operator==(const RegisterOrMemory&, const RegisterOrMemory&);
        std::vector<uint8_t> bincodeSerialize() const;
        static RegisterOrMemory bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Value {
        std::string inner;

        friend bool operator==(const Value&, const Value&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Value bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BrilligOpcode {

        struct BinaryFieldOp {
            Circuit::RegisterIndex destination;
            Circuit::BinaryFieldOp op;
            Circuit::RegisterIndex lhs;
            Circuit::RegisterIndex rhs;

            friend bool operator==(const BinaryFieldOp&, const BinaryFieldOp&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BinaryFieldOp bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BinaryIntOp {
            Circuit::RegisterIndex destination;
            Circuit::BinaryIntOp op;
            uint32_t bit_size;
            Circuit::RegisterIndex lhs;
            Circuit::RegisterIndex rhs;

            friend bool operator==(const BinaryIntOp&, const BinaryIntOp&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BinaryIntOp bincodeDeserialize(std::vector<uint8_t>);
        };

        struct JumpIfNot {
            Circuit::RegisterIndex condition;
            uint64_t location;

            friend bool operator==(const JumpIfNot&, const JumpIfNot&);
            std::vector<uint8_t> bincodeSerialize() const;
            static JumpIfNot bincodeDeserialize(std::vector<uint8_t>);
        };

        struct JumpIf {
            Circuit::RegisterIndex condition;
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

        struct Call {
            uint64_t location;

            friend bool operator==(const Call&, const Call&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Call bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Const {
            Circuit::RegisterIndex destination;
            Circuit::Value value;

            friend bool operator==(const Const&, const Const&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Const bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Return {
            friend bool operator==(const Return&, const Return&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Return bincodeDeserialize(std::vector<uint8_t>);
        };

        struct ForeignCall {
            std::string function;
            std::vector<Circuit::RegisterOrMemory> destinations;
            std::vector<Circuit::RegisterOrMemory> inputs;

            friend bool operator==(const ForeignCall&, const ForeignCall&);
            std::vector<uint8_t> bincodeSerialize() const;
            static ForeignCall bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Mov {
            Circuit::RegisterIndex destination;
            Circuit::RegisterIndex source;

            friend bool operator==(const Mov&, const Mov&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Mov bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Load {
            Circuit::RegisterIndex destination;
            Circuit::RegisterIndex source_pointer;

            friend bool operator==(const Load&, const Load&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Load bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Store {
            Circuit::RegisterIndex destination_pointer;
            Circuit::RegisterIndex source;

            friend bool operator==(const Store&, const Store&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Store bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BlackBox {
            Circuit::BlackBoxOp value;

            friend bool operator==(const BlackBox&, const BlackBox&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BlackBox bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Trap {
            friend bool operator==(const Trap&, const Trap&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Trap bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Stop {
            friend bool operator==(const Stop&, const Stop&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Stop bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<BinaryFieldOp, BinaryIntOp, JumpIfNot, JumpIf, Jump, Call, Const, Return, ForeignCall, Mov, Load, Store, BlackBox, Trap, Stop> value;

        friend bool operator==(const BrilligOpcode&, const BrilligOpcode&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligOpcode bincodeDeserialize(std::vector<uint8_t>);
    };

    struct BrilligOutputs {

        struct Simple {
            Circuit::Witness value;

            friend bool operator==(const Simple&, const Simple&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Simple bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Array {
            std::vector<Circuit::Witness> value;

            friend bool operator==(const Array&, const Array&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Array bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<Simple, Array> value;

        friend bool operator==(const BrilligOutputs&, const BrilligOutputs&);
        std::vector<uint8_t> bincodeSerialize() const;
        static BrilligOutputs bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Brillig {
        std::vector<Circuit::BrilligInputs> inputs;
        std::vector<Circuit::BrilligOutputs> outputs;
        std::vector<Circuit::BrilligOpcode> bytecode;
        std::optional<Circuit::Expression> predicate;

        friend bool operator==(const Brillig&, const Brillig&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Brillig bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Directive {

        struct ToLeRadix {
            Circuit::Expression a;
            std::vector<Circuit::Witness> b;
            uint32_t radix;

            friend bool operator==(const ToLeRadix&, const ToLeRadix&);
            std::vector<uint8_t> bincodeSerialize() const;
            static ToLeRadix bincodeDeserialize(std::vector<uint8_t>);
        };

        struct PermutationSort {
            std::vector<std::vector<Circuit::Expression>> inputs;
            uint32_t tuple;
            std::vector<Circuit::Witness> bits;
            std::vector<uint32_t> sort_by;

            friend bool operator==(const PermutationSort&, const PermutationSort&);
            std::vector<uint8_t> bincodeSerialize() const;
            static PermutationSort bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<ToLeRadix, PermutationSort> value;

        friend bool operator==(const Directive&, const Directive&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Directive bincodeDeserialize(std::vector<uint8_t>);
    };

    struct MemOp {
        Circuit::Expression operation;
        Circuit::Expression index;
        Circuit::Expression value;

        friend bool operator==(const MemOp&, const MemOp&);
        std::vector<uint8_t> bincodeSerialize() const;
        static MemOp bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Opcode {

        struct AssertZero {
            Circuit::Expression value;

            friend bool operator==(const AssertZero&, const AssertZero&);
            std::vector<uint8_t> bincodeSerialize() const;
            static AssertZero bincodeDeserialize(std::vector<uint8_t>);
        };

        struct BlackBoxFuncCall {
            Circuit::BlackBoxFuncCall value;

            friend bool operator==(const BlackBoxFuncCall&, const BlackBoxFuncCall&);
            std::vector<uint8_t> bincodeSerialize() const;
            static BlackBoxFuncCall bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Directive {
            Circuit::Directive value;

            friend bool operator==(const Directive&, const Directive&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Directive bincodeDeserialize(std::vector<uint8_t>);
        };

        struct Brillig {
            Circuit::Brillig value;

            friend bool operator==(const Brillig&, const Brillig&);
            std::vector<uint8_t> bincodeSerialize() const;
            static Brillig bincodeDeserialize(std::vector<uint8_t>);
        };

        struct MemoryOp {
            Circuit::BlockId block_id;
            Circuit::MemOp op;
            std::optional<Circuit::Expression> predicate;

            friend bool operator==(const MemoryOp&, const MemoryOp&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryOp bincodeDeserialize(std::vector<uint8_t>);
        };

        struct MemoryInit {
            Circuit::BlockId block_id;
            std::vector<Circuit::Witness> init;

            friend bool operator==(const MemoryInit&, const MemoryInit&);
            std::vector<uint8_t> bincodeSerialize() const;
            static MemoryInit bincodeDeserialize(std::vector<uint8_t>);
        };

        std::variant<AssertZero, BlackBoxFuncCall, Directive, Brillig, MemoryOp, MemoryInit> value;

        friend bool operator==(const Opcode&, const Opcode&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Opcode bincodeDeserialize(std::vector<uint8_t>);
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
        std::vector<Circuit::Witness> value;

        friend bool operator==(const PublicInputs&, const PublicInputs&);
        std::vector<uint8_t> bincodeSerialize() const;
        static PublicInputs bincodeDeserialize(std::vector<uint8_t>);
    };

    struct Circuit {
        uint32_t current_witness_index;
        std::vector<Circuit::Opcode> opcodes;
        std::vector<Circuit::Witness> private_parameters;
        Circuit::PublicInputs public_parameters;
        Circuit::PublicInputs return_values;
        std::vector<std::tuple<Circuit::OpcodeLocation, std::string>> assert_messages;

        friend bool operator==(const Circuit&, const Circuit&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Circuit bincodeDeserialize(std::vector<uint8_t>);
    };

} // end of namespace Circuit


namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryFieldOp>::serialize(const Circuit::BinaryFieldOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::BinaryFieldOp serde::Deserializable<Circuit::BinaryFieldOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::BinaryFieldOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryFieldOp::Add>::serialize(const Circuit::BinaryFieldOp::Add &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryFieldOp::Add serde::Deserializable<Circuit::BinaryFieldOp::Add>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryFieldOp::Add obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryFieldOp::Sub>::serialize(const Circuit::BinaryFieldOp::Sub &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryFieldOp::Sub serde::Deserializable<Circuit::BinaryFieldOp::Sub>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryFieldOp::Sub obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryFieldOp::Mul>::serialize(const Circuit::BinaryFieldOp::Mul &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryFieldOp::Mul serde::Deserializable<Circuit::BinaryFieldOp::Mul>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryFieldOp::Mul obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryFieldOp::Div>::serialize(const Circuit::BinaryFieldOp::Div &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryFieldOp::Div serde::Deserializable<Circuit::BinaryFieldOp::Div>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryFieldOp::Div obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryFieldOp::Equals>::serialize(const Circuit::BinaryFieldOp::Equals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryFieldOp::Equals serde::Deserializable<Circuit::BinaryFieldOp::Equals>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryFieldOp::Equals obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp>::serialize(const Circuit::BinaryIntOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp serde::Deserializable<Circuit::BinaryIntOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::BinaryIntOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::Add>::serialize(const Circuit::BinaryIntOp::Add &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::Add serde::Deserializable<Circuit::BinaryIntOp::Add>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::Add obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::Sub>::serialize(const Circuit::BinaryIntOp::Sub &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::Sub serde::Deserializable<Circuit::BinaryIntOp::Sub>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::Sub obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::Mul>::serialize(const Circuit::BinaryIntOp::Mul &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::Mul serde::Deserializable<Circuit::BinaryIntOp::Mul>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::Mul obj;
    return obj;
}

namespace Circuit {

    inline bool operator==(const BinaryIntOp::SignedDiv &lhs, const BinaryIntOp::SignedDiv &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::SignedDiv::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::SignedDiv>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::SignedDiv BinaryIntOp::SignedDiv::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::SignedDiv>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::SignedDiv>::serialize(const Circuit::BinaryIntOp::SignedDiv &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::SignedDiv serde::Deserializable<Circuit::BinaryIntOp::SignedDiv>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::SignedDiv obj;
    return obj;
}

namespace Circuit {

    inline bool operator==(const BinaryIntOp::UnsignedDiv &lhs, const BinaryIntOp::UnsignedDiv &rhs) {
        return true;
    }

    inline std::vector<uint8_t> BinaryIntOp::UnsignedDiv::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BinaryIntOp::UnsignedDiv>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BinaryIntOp::UnsignedDiv BinaryIntOp::UnsignedDiv::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BinaryIntOp::UnsignedDiv>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::UnsignedDiv>::serialize(const Circuit::BinaryIntOp::UnsignedDiv &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::UnsignedDiv serde::Deserializable<Circuit::BinaryIntOp::UnsignedDiv>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::UnsignedDiv obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::Equals>::serialize(const Circuit::BinaryIntOp::Equals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::Equals serde::Deserializable<Circuit::BinaryIntOp::Equals>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::Equals obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::LessThan>::serialize(const Circuit::BinaryIntOp::LessThan &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::LessThan serde::Deserializable<Circuit::BinaryIntOp::LessThan>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::LessThan obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::LessThanEquals>::serialize(const Circuit::BinaryIntOp::LessThanEquals &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::LessThanEquals serde::Deserializable<Circuit::BinaryIntOp::LessThanEquals>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::LessThanEquals obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::And>::serialize(const Circuit::BinaryIntOp::And &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::And serde::Deserializable<Circuit::BinaryIntOp::And>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::And obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::Or>::serialize(const Circuit::BinaryIntOp::Or &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::Or serde::Deserializable<Circuit::BinaryIntOp::Or>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::Or obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::Xor>::serialize(const Circuit::BinaryIntOp::Xor &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::Xor serde::Deserializable<Circuit::BinaryIntOp::Xor>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::Xor obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::Shl>::serialize(const Circuit::BinaryIntOp::Shl &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::Shl serde::Deserializable<Circuit::BinaryIntOp::Shl>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::Shl obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BinaryIntOp::Shr>::serialize(const Circuit::BinaryIntOp::Shr &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BinaryIntOp::Shr serde::Deserializable<Circuit::BinaryIntOp::Shr>::deserialize(Deserializer &deserializer) {
    Circuit::BinaryIntOp::Shr obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall>::serialize(const Circuit::BlackBoxFuncCall &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall serde::Deserializable<Circuit::BlackBoxFuncCall>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::BlackBoxFuncCall obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::AND>::serialize(const Circuit::BlackBoxFuncCall::AND &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::AND serde::Deserializable<Circuit::BlackBoxFuncCall::AND>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::AND obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::XOR>::serialize(const Circuit::BlackBoxFuncCall::XOR &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::XOR serde::Deserializable<Circuit::BlackBoxFuncCall::XOR>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::XOR obj;
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::RANGE>::serialize(const Circuit::BlackBoxFuncCall::RANGE &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input)>::serialize(obj.input, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::RANGE serde::Deserializable<Circuit::BlackBoxFuncCall::RANGE>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::RANGE obj;
    obj.input = serde::Deserializable<decltype(obj.input)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::SHA256 &lhs, const BlackBoxFuncCall::SHA256 &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::SHA256::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::SHA256>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::SHA256 BlackBoxFuncCall::SHA256::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::SHA256>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::SHA256>::serialize(const Circuit::BlackBoxFuncCall::SHA256 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::SHA256 serde::Deserializable<Circuit::BlackBoxFuncCall::SHA256>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::SHA256 obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::Blake2s>::serialize(const Circuit::BlackBoxFuncCall::Blake2s &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::Blake2s serde::Deserializable<Circuit::BlackBoxFuncCall::Blake2s>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::Blake2s obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::Blake3>::serialize(const Circuit::BlackBoxFuncCall::Blake3 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::Blake3 serde::Deserializable<Circuit::BlackBoxFuncCall::Blake3>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::Blake3 obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::SchnorrVerify &lhs, const BlackBoxFuncCall::SchnorrVerify &rhs) {
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::SchnorrVerify::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::SchnorrVerify>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::SchnorrVerify BlackBoxFuncCall::SchnorrVerify::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::SchnorrVerify>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::SchnorrVerify>::serialize(const Circuit::BlackBoxFuncCall::SchnorrVerify &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::SchnorrVerify serde::Deserializable<Circuit::BlackBoxFuncCall::SchnorrVerify>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::SchnorrVerify obj;
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::PedersenCommitment &lhs, const BlackBoxFuncCall::PedersenCommitment &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.domain_separator == rhs.domain_separator)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::PedersenCommitment::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::PedersenCommitment>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::PedersenCommitment BlackBoxFuncCall::PedersenCommitment::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::PedersenCommitment>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::PedersenCommitment>::serialize(const Circuit::BlackBoxFuncCall::PedersenCommitment &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.domain_separator)>::serialize(obj.domain_separator, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::PedersenCommitment serde::Deserializable<Circuit::BlackBoxFuncCall::PedersenCommitment>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::PedersenCommitment obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.domain_separator = serde::Deserializable<decltype(obj.domain_separator)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::PedersenHash &lhs, const BlackBoxFuncCall::PedersenHash &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.domain_separator == rhs.domain_separator)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::PedersenHash::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::PedersenHash>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::PedersenHash BlackBoxFuncCall::PedersenHash::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::PedersenHash>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::PedersenHash>::serialize(const Circuit::BlackBoxFuncCall::PedersenHash &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.domain_separator)>::serialize(obj.domain_separator, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::PedersenHash serde::Deserializable<Circuit::BlackBoxFuncCall::PedersenHash>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::PedersenHash obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.domain_separator = serde::Deserializable<decltype(obj.domain_separator)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::EcdsaSecp256k1>::serialize(const Circuit::BlackBoxFuncCall::EcdsaSecp256k1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.hashed_message)>::serialize(obj.hashed_message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::EcdsaSecp256k1 serde::Deserializable<Circuit::BlackBoxFuncCall::EcdsaSecp256k1>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::EcdsaSecp256k1 obj;
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.hashed_message = serde::Deserializable<decltype(obj.hashed_message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::EcdsaSecp256r1>::serialize(const Circuit::BlackBoxFuncCall::EcdsaSecp256r1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.hashed_message)>::serialize(obj.hashed_message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::EcdsaSecp256r1 serde::Deserializable<Circuit::BlackBoxFuncCall::EcdsaSecp256r1>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::EcdsaSecp256r1 obj;
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.hashed_message = serde::Deserializable<decltype(obj.hashed_message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::FixedBaseScalarMul &lhs, const BlackBoxFuncCall::FixedBaseScalarMul &rhs) {
        if (!(lhs.low == rhs.low)) { return false; }
        if (!(lhs.high == rhs.high)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::FixedBaseScalarMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::FixedBaseScalarMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::FixedBaseScalarMul BlackBoxFuncCall::FixedBaseScalarMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::FixedBaseScalarMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::FixedBaseScalarMul>::serialize(const Circuit::BlackBoxFuncCall::FixedBaseScalarMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.low)>::serialize(obj.low, serializer);
    serde::Serializable<decltype(obj.high)>::serialize(obj.high, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::FixedBaseScalarMul serde::Deserializable<Circuit::BlackBoxFuncCall::FixedBaseScalarMul>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::FixedBaseScalarMul obj;
    obj.low = serde::Deserializable<decltype(obj.low)>::deserialize(deserializer);
    obj.high = serde::Deserializable<decltype(obj.high)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::EmbeddedCurveAdd &lhs, const BlackBoxFuncCall::EmbeddedCurveAdd &rhs) {
        if (!(lhs.input1_x == rhs.input1_x)) { return false; }
        if (!(lhs.input1_y == rhs.input1_y)) { return false; }
        if (!(lhs.input2_x == rhs.input2_x)) { return false; }
        if (!(lhs.input2_y == rhs.input2_y)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::EmbeddedCurveAdd>::serialize(const Circuit::BlackBoxFuncCall::EmbeddedCurveAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input1_x)>::serialize(obj.input1_x, serializer);
    serde::Serializable<decltype(obj.input1_y)>::serialize(obj.input1_y, serializer);
    serde::Serializable<decltype(obj.input2_x)>::serialize(obj.input2_x, serializer);
    serde::Serializable<decltype(obj.input2_y)>::serialize(obj.input2_y, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::EmbeddedCurveAdd serde::Deserializable<Circuit::BlackBoxFuncCall::EmbeddedCurveAdd>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::EmbeddedCurveAdd obj;
    obj.input1_x = serde::Deserializable<decltype(obj.input1_x)>::deserialize(deserializer);
    obj.input1_y = serde::Deserializable<decltype(obj.input1_y)>::deserialize(deserializer);
    obj.input2_x = serde::Deserializable<decltype(obj.input2_x)>::deserialize(deserializer);
    obj.input2_y = serde::Deserializable<decltype(obj.input2_y)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::EmbeddedCurveDouble &lhs, const BlackBoxFuncCall::EmbeddedCurveDouble &rhs) {
        if (!(lhs.input_x == rhs.input_x)) { return false; }
        if (!(lhs.input_y == rhs.input_y)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::EmbeddedCurveDouble::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::EmbeddedCurveDouble>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::EmbeddedCurveDouble BlackBoxFuncCall::EmbeddedCurveDouble::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::EmbeddedCurveDouble>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::EmbeddedCurveDouble>::serialize(const Circuit::BlackBoxFuncCall::EmbeddedCurveDouble &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input_x)>::serialize(obj.input_x, serializer);
    serde::Serializable<decltype(obj.input_y)>::serialize(obj.input_y, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::EmbeddedCurveDouble serde::Deserializable<Circuit::BlackBoxFuncCall::EmbeddedCurveDouble>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::EmbeddedCurveDouble obj;
    obj.input_x = serde::Deserializable<decltype(obj.input_x)>::deserialize(deserializer);
    obj.input_y = serde::Deserializable<decltype(obj.input_y)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::Keccak256 &lhs, const BlackBoxFuncCall::Keccak256 &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Keccak256::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Keccak256>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Keccak256 BlackBoxFuncCall::Keccak256::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Keccak256>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::Keccak256>::serialize(const Circuit::BlackBoxFuncCall::Keccak256 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::Keccak256 serde::Deserializable<Circuit::BlackBoxFuncCall::Keccak256>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::Keccak256 obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::Keccak256VariableLength &lhs, const BlackBoxFuncCall::Keccak256VariableLength &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.var_message_size == rhs.var_message_size)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxFuncCall::Keccak256VariableLength::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxFuncCall::Keccak256VariableLength>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxFuncCall::Keccak256VariableLength BlackBoxFuncCall::Keccak256VariableLength::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxFuncCall::Keccak256VariableLength>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::Keccak256VariableLength>::serialize(const Circuit::BlackBoxFuncCall::Keccak256VariableLength &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.var_message_size)>::serialize(obj.var_message_size, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::Keccak256VariableLength serde::Deserializable<Circuit::BlackBoxFuncCall::Keccak256VariableLength>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::Keccak256VariableLength obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.var_message_size = serde::Deserializable<decltype(obj.var_message_size)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::Keccakf1600>::serialize(const Circuit::BlackBoxFuncCall::Keccakf1600 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::Keccakf1600 serde::Deserializable<Circuit::BlackBoxFuncCall::Keccakf1600>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::Keccakf1600 obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxFuncCall::RecursiveAggregation &lhs, const BlackBoxFuncCall::RecursiveAggregation &rhs) {
        if (!(lhs.verification_key == rhs.verification_key)) { return false; }
        if (!(lhs.proof == rhs.proof)) { return false; }
        if (!(lhs.public_inputs == rhs.public_inputs)) { return false; }
        if (!(lhs.key_hash == rhs.key_hash)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxFuncCall::RecursiveAggregation>::serialize(const Circuit::BlackBoxFuncCall::RecursiveAggregation &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.verification_key)>::serialize(obj.verification_key, serializer);
    serde::Serializable<decltype(obj.proof)>::serialize(obj.proof, serializer);
    serde::Serializable<decltype(obj.public_inputs)>::serialize(obj.public_inputs, serializer);
    serde::Serializable<decltype(obj.key_hash)>::serialize(obj.key_hash, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxFuncCall::RecursiveAggregation serde::Deserializable<Circuit::BlackBoxFuncCall::RecursiveAggregation>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxFuncCall::RecursiveAggregation obj;
    obj.verification_key = serde::Deserializable<decltype(obj.verification_key)>::deserialize(deserializer);
    obj.proof = serde::Deserializable<decltype(obj.proof)>::deserialize(deserializer);
    obj.public_inputs = serde::Deserializable<decltype(obj.public_inputs)>::deserialize(deserializer);
    obj.key_hash = serde::Deserializable<decltype(obj.key_hash)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp>::serialize(const Circuit::BlackBoxOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp serde::Deserializable<Circuit::BlackBoxOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::BlackBoxOp obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::Sha256 &lhs, const BlackBoxOp::Sha256 &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Sha256::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Sha256>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Sha256 BlackBoxOp::Sha256::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Sha256>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::Sha256>::serialize(const Circuit::BlackBoxOp::Sha256 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::Sha256 serde::Deserializable<Circuit::BlackBoxOp::Sha256>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::Sha256 obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::Blake2s>::serialize(const Circuit::BlackBoxOp::Blake2s &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::Blake2s serde::Deserializable<Circuit::BlackBoxOp::Blake2s>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::Blake2s obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::Blake3>::serialize(const Circuit::BlackBoxOp::Blake3 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::Blake3 serde::Deserializable<Circuit::BlackBoxOp::Blake3>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::Blake3 obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::Keccak256 &lhs, const BlackBoxOp::Keccak256 &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::Keccak256::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::Keccak256>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::Keccak256 BlackBoxOp::Keccak256::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::Keccak256>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::Keccak256>::serialize(const Circuit::BlackBoxOp::Keccak256 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::Keccak256 serde::Deserializable<Circuit::BlackBoxOp::Keccak256>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::Keccak256 obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::Keccakf1600 &lhs, const BlackBoxOp::Keccakf1600 &rhs) {
        if (!(lhs.message == rhs.message)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::Keccakf1600>::serialize(const Circuit::BlackBoxOp::Keccakf1600 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::Keccakf1600 serde::Deserializable<Circuit::BlackBoxOp::Keccakf1600>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::Keccakf1600 obj;
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::EcdsaSecp256k1>::serialize(const Circuit::BlackBoxOp::EcdsaSecp256k1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.hashed_msg)>::serialize(obj.hashed_msg, serializer);
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::EcdsaSecp256k1 serde::Deserializable<Circuit::BlackBoxOp::EcdsaSecp256k1>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::EcdsaSecp256k1 obj;
    obj.hashed_msg = serde::Deserializable<decltype(obj.hashed_msg)>::deserialize(deserializer);
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::EcdsaSecp256r1>::serialize(const Circuit::BlackBoxOp::EcdsaSecp256r1 &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.hashed_msg)>::serialize(obj.hashed_msg, serializer);
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::EcdsaSecp256r1 serde::Deserializable<Circuit::BlackBoxOp::EcdsaSecp256r1>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::EcdsaSecp256r1 obj;
    obj.hashed_msg = serde::Deserializable<decltype(obj.hashed_msg)>::deserialize(deserializer);
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::SchnorrVerify &lhs, const BlackBoxOp::SchnorrVerify &rhs) {
        if (!(lhs.public_key_x == rhs.public_key_x)) { return false; }
        if (!(lhs.public_key_y == rhs.public_key_y)) { return false; }
        if (!(lhs.message == rhs.message)) { return false; }
        if (!(lhs.signature == rhs.signature)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::SchnorrVerify::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::SchnorrVerify>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::SchnorrVerify BlackBoxOp::SchnorrVerify::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::SchnorrVerify>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::SchnorrVerify>::serialize(const Circuit::BlackBoxOp::SchnorrVerify &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.public_key_x)>::serialize(obj.public_key_x, serializer);
    serde::Serializable<decltype(obj.public_key_y)>::serialize(obj.public_key_y, serializer);
    serde::Serializable<decltype(obj.message)>::serialize(obj.message, serializer);
    serde::Serializable<decltype(obj.signature)>::serialize(obj.signature, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::SchnorrVerify serde::Deserializable<Circuit::BlackBoxOp::SchnorrVerify>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::SchnorrVerify obj;
    obj.public_key_x = serde::Deserializable<decltype(obj.public_key_x)>::deserialize(deserializer);
    obj.public_key_y = serde::Deserializable<decltype(obj.public_key_y)>::deserialize(deserializer);
    obj.message = serde::Deserializable<decltype(obj.message)>::deserialize(deserializer);
    obj.signature = serde::Deserializable<decltype(obj.signature)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::PedersenCommitment &lhs, const BlackBoxOp::PedersenCommitment &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.domain_separator == rhs.domain_separator)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::PedersenCommitment::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::PedersenCommitment>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::PedersenCommitment BlackBoxOp::PedersenCommitment::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::PedersenCommitment>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::PedersenCommitment>::serialize(const Circuit::BlackBoxOp::PedersenCommitment &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.domain_separator)>::serialize(obj.domain_separator, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::PedersenCommitment serde::Deserializable<Circuit::BlackBoxOp::PedersenCommitment>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::PedersenCommitment obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.domain_separator = serde::Deserializable<decltype(obj.domain_separator)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::PedersenHash &lhs, const BlackBoxOp::PedersenHash &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.domain_separator == rhs.domain_separator)) { return false; }
        if (!(lhs.output == rhs.output)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::PedersenHash::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::PedersenHash>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::PedersenHash BlackBoxOp::PedersenHash::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::PedersenHash>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::PedersenHash>::serialize(const Circuit::BlackBoxOp::PedersenHash &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.domain_separator)>::serialize(obj.domain_separator, serializer);
    serde::Serializable<decltype(obj.output)>::serialize(obj.output, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::PedersenHash serde::Deserializable<Circuit::BlackBoxOp::PedersenHash>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::PedersenHash obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.domain_separator = serde::Deserializable<decltype(obj.domain_separator)>::deserialize(deserializer);
    obj.output = serde::Deserializable<decltype(obj.output)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::FixedBaseScalarMul &lhs, const BlackBoxOp::FixedBaseScalarMul &rhs) {
        if (!(lhs.low == rhs.low)) { return false; }
        if (!(lhs.high == rhs.high)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::FixedBaseScalarMul::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::FixedBaseScalarMul>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::FixedBaseScalarMul BlackBoxOp::FixedBaseScalarMul::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::FixedBaseScalarMul>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::FixedBaseScalarMul>::serialize(const Circuit::BlackBoxOp::FixedBaseScalarMul &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.low)>::serialize(obj.low, serializer);
    serde::Serializable<decltype(obj.high)>::serialize(obj.high, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::FixedBaseScalarMul serde::Deserializable<Circuit::BlackBoxOp::FixedBaseScalarMul>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::FixedBaseScalarMul obj;
    obj.low = serde::Deserializable<decltype(obj.low)>::deserialize(deserializer);
    obj.high = serde::Deserializable<decltype(obj.high)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::EmbeddedCurveAdd &lhs, const BlackBoxOp::EmbeddedCurveAdd &rhs) {
        if (!(lhs.input1_x == rhs.input1_x)) { return false; }
        if (!(lhs.input1_y == rhs.input1_y)) { return false; }
        if (!(lhs.input2_x == rhs.input2_x)) { return false; }
        if (!(lhs.input2_y == rhs.input2_y)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::EmbeddedCurveAdd>::serialize(const Circuit::BlackBoxOp::EmbeddedCurveAdd &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input1_x)>::serialize(obj.input1_x, serializer);
    serde::Serializable<decltype(obj.input1_y)>::serialize(obj.input1_y, serializer);
    serde::Serializable<decltype(obj.input2_x)>::serialize(obj.input2_x, serializer);
    serde::Serializable<decltype(obj.input2_y)>::serialize(obj.input2_y, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::EmbeddedCurveAdd serde::Deserializable<Circuit::BlackBoxOp::EmbeddedCurveAdd>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::EmbeddedCurveAdd obj;
    obj.input1_x = serde::Deserializable<decltype(obj.input1_x)>::deserialize(deserializer);
    obj.input1_y = serde::Deserializable<decltype(obj.input1_y)>::deserialize(deserializer);
    obj.input2_x = serde::Deserializable<decltype(obj.input2_x)>::deserialize(deserializer);
    obj.input2_y = serde::Deserializable<decltype(obj.input2_y)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BlackBoxOp::EmbeddedCurveDouble &lhs, const BlackBoxOp::EmbeddedCurveDouble &rhs) {
        if (!(lhs.input1_x == rhs.input1_x)) { return false; }
        if (!(lhs.input1_y == rhs.input1_y)) { return false; }
        if (!(lhs.result == rhs.result)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> BlackBoxOp::EmbeddedCurveDouble::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<BlackBoxOp::EmbeddedCurveDouble>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline BlackBoxOp::EmbeddedCurveDouble BlackBoxOp::EmbeddedCurveDouble::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<BlackBoxOp::EmbeddedCurveDouble>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlackBoxOp::EmbeddedCurveDouble>::serialize(const Circuit::BlackBoxOp::EmbeddedCurveDouble &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.input1_x)>::serialize(obj.input1_x, serializer);
    serde::Serializable<decltype(obj.input1_y)>::serialize(obj.input1_y, serializer);
    serde::Serializable<decltype(obj.result)>::serialize(obj.result, serializer);
}

template <>
template <typename Deserializer>
Circuit::BlackBoxOp::EmbeddedCurveDouble serde::Deserializable<Circuit::BlackBoxOp::EmbeddedCurveDouble>::deserialize(Deserializer &deserializer) {
    Circuit::BlackBoxOp::EmbeddedCurveDouble obj;
    obj.input1_x = serde::Deserializable<decltype(obj.input1_x)>::deserialize(deserializer);
    obj.input1_y = serde::Deserializable<decltype(obj.input1_y)>::deserialize(deserializer);
    obj.result = serde::Deserializable<decltype(obj.result)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BlockId>::serialize(const Circuit::BlockId &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::BlockId serde::Deserializable<Circuit::BlockId>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::BlockId obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

    inline bool operator==(const Brillig &lhs, const Brillig &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.outputs == rhs.outputs)) { return false; }
        if (!(lhs.bytecode == rhs.bytecode)) { return false; }
        if (!(lhs.predicate == rhs.predicate)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Brillig::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Brillig>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Brillig Brillig::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Brillig>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Brillig>::serialize(const Circuit::Brillig &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.outputs)>::serialize(obj.outputs, serializer);
    serde::Serializable<decltype(obj.bytecode)>::serialize(obj.bytecode, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::Brillig serde::Deserializable<Circuit::Brillig>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::Brillig obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.outputs = serde::Deserializable<decltype(obj.outputs)>::deserialize(deserializer);
    obj.bytecode = serde::Deserializable<decltype(obj.bytecode)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligInputs>::serialize(const Circuit::BrilligInputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::BrilligInputs serde::Deserializable<Circuit::BrilligInputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::BrilligInputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligInputs::Single>::serialize(const Circuit::BrilligInputs::Single &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligInputs::Single serde::Deserializable<Circuit::BrilligInputs::Single>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligInputs::Single obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligInputs::Array>::serialize(const Circuit::BrilligInputs::Array &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligInputs::Array serde::Deserializable<Circuit::BrilligInputs::Array>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligInputs::Array obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode>::serialize(const Circuit::BrilligOpcode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode serde::Deserializable<Circuit::BrilligOpcode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::BrilligOpcode obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::BinaryFieldOp>::serialize(const Circuit::BrilligOpcode::BinaryFieldOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::BinaryFieldOp serde::Deserializable<Circuit::BrilligOpcode::BinaryFieldOp>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::BinaryFieldOp obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::BinaryIntOp>::serialize(const Circuit::BrilligOpcode::BinaryIntOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
    serde::Serializable<decltype(obj.bit_size)>::serialize(obj.bit_size, serializer);
    serde::Serializable<decltype(obj.lhs)>::serialize(obj.lhs, serializer);
    serde::Serializable<decltype(obj.rhs)>::serialize(obj.rhs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::BinaryIntOp serde::Deserializable<Circuit::BrilligOpcode::BinaryIntOp>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::BinaryIntOp obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    obj.bit_size = serde::Deserializable<decltype(obj.bit_size)>::deserialize(deserializer);
    obj.lhs = serde::Deserializable<decltype(obj.lhs)>::deserialize(deserializer);
    obj.rhs = serde::Deserializable<decltype(obj.rhs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::JumpIfNot>::serialize(const Circuit::BrilligOpcode::JumpIfNot &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.condition)>::serialize(obj.condition, serializer);
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::JumpIfNot serde::Deserializable<Circuit::BrilligOpcode::JumpIfNot>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::JumpIfNot obj;
    obj.condition = serde::Deserializable<decltype(obj.condition)>::deserialize(deserializer);
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::JumpIf>::serialize(const Circuit::BrilligOpcode::JumpIf &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.condition)>::serialize(obj.condition, serializer);
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::JumpIf serde::Deserializable<Circuit::BrilligOpcode::JumpIf>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::JumpIf obj;
    obj.condition = serde::Deserializable<decltype(obj.condition)>::deserialize(deserializer);
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Jump>::serialize(const Circuit::BrilligOpcode::Jump &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Jump serde::Deserializable<Circuit::BrilligOpcode::Jump>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Jump obj;
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Call>::serialize(const Circuit::BrilligOpcode::Call &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.location)>::serialize(obj.location, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Call serde::Deserializable<Circuit::BrilligOpcode::Call>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Call obj;
    obj.location = serde::Deserializable<decltype(obj.location)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BrilligOpcode::Const &lhs, const BrilligOpcode::Const &rhs) {
        if (!(lhs.destination == rhs.destination)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Const>::serialize(const Circuit::BrilligOpcode::Const &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Const serde::Deserializable<Circuit::BrilligOpcode::Const>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Const obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Return>::serialize(const Circuit::BrilligOpcode::Return &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Return serde::Deserializable<Circuit::BrilligOpcode::Return>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Return obj;
    return obj;
}

namespace Circuit {

    inline bool operator==(const BrilligOpcode::ForeignCall &lhs, const BrilligOpcode::ForeignCall &rhs) {
        if (!(lhs.function == rhs.function)) { return false; }
        if (!(lhs.destinations == rhs.destinations)) { return false; }
        if (!(lhs.inputs == rhs.inputs)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::ForeignCall>::serialize(const Circuit::BrilligOpcode::ForeignCall &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.function)>::serialize(obj.function, serializer);
    serde::Serializable<decltype(obj.destinations)>::serialize(obj.destinations, serializer);
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::ForeignCall serde::Deserializable<Circuit::BrilligOpcode::ForeignCall>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::ForeignCall obj;
    obj.function = serde::Deserializable<decltype(obj.function)>::deserialize(deserializer);
    obj.destinations = serde::Deserializable<decltype(obj.destinations)>::deserialize(deserializer);
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Mov>::serialize(const Circuit::BrilligOpcode::Mov &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Mov serde::Deserializable<Circuit::BrilligOpcode::Mov>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Mov obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Load>::serialize(const Circuit::BrilligOpcode::Load &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination)>::serialize(obj.destination, serializer);
    serde::Serializable<decltype(obj.source_pointer)>::serialize(obj.source_pointer, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Load serde::Deserializable<Circuit::BrilligOpcode::Load>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Load obj;
    obj.destination = serde::Deserializable<decltype(obj.destination)>::deserialize(deserializer);
    obj.source_pointer = serde::Deserializable<decltype(obj.source_pointer)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Store>::serialize(const Circuit::BrilligOpcode::Store &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.destination_pointer)>::serialize(obj.destination_pointer, serializer);
    serde::Serializable<decltype(obj.source)>::serialize(obj.source, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Store serde::Deserializable<Circuit::BrilligOpcode::Store>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Store obj;
    obj.destination_pointer = serde::Deserializable<decltype(obj.destination_pointer)>::deserialize(deserializer);
    obj.source = serde::Deserializable<decltype(obj.source)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::BlackBox>::serialize(const Circuit::BrilligOpcode::BlackBox &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::BlackBox serde::Deserializable<Circuit::BrilligOpcode::BlackBox>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::BlackBox obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const BrilligOpcode::Trap &lhs, const BrilligOpcode::Trap &rhs) {
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Trap>::serialize(const Circuit::BrilligOpcode::Trap &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Trap serde::Deserializable<Circuit::BrilligOpcode::Trap>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Trap obj;
    return obj;
}

namespace Circuit {

    inline bool operator==(const BrilligOpcode::Stop &lhs, const BrilligOpcode::Stop &rhs) {
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOpcode::Stop>::serialize(const Circuit::BrilligOpcode::Stop &obj, Serializer &serializer) {
}

template <>
template <typename Deserializer>
Circuit::BrilligOpcode::Stop serde::Deserializable<Circuit::BrilligOpcode::Stop>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOpcode::Stop obj;
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOutputs>::serialize(const Circuit::BrilligOutputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::BrilligOutputs serde::Deserializable<Circuit::BrilligOutputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::BrilligOutputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOutputs::Simple>::serialize(const Circuit::BrilligOutputs::Simple &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOutputs::Simple serde::Deserializable<Circuit::BrilligOutputs::Simple>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOutputs::Simple obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::BrilligOutputs::Array>::serialize(const Circuit::BrilligOutputs::Array &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::BrilligOutputs::Array serde::Deserializable<Circuit::BrilligOutputs::Array>::deserialize(Deserializer &deserializer) {
    Circuit::BrilligOutputs::Array obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const Circuit &lhs, const Circuit &rhs) {
        if (!(lhs.current_witness_index == rhs.current_witness_index)) { return false; }
        if (!(lhs.opcodes == rhs.opcodes)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Circuit>::serialize(const Circuit::Circuit &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.current_witness_index)>::serialize(obj.current_witness_index, serializer);
    serde::Serializable<decltype(obj.opcodes)>::serialize(obj.opcodes, serializer);
    serde::Serializable<decltype(obj.private_parameters)>::serialize(obj.private_parameters, serializer);
    serde::Serializable<decltype(obj.public_parameters)>::serialize(obj.public_parameters, serializer);
    serde::Serializable<decltype(obj.return_values)>::serialize(obj.return_values, serializer);
    serde::Serializable<decltype(obj.assert_messages)>::serialize(obj.assert_messages, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::Circuit serde::Deserializable<Circuit::Circuit>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::Circuit obj;
    obj.current_witness_index = serde::Deserializable<decltype(obj.current_witness_index)>::deserialize(deserializer);
    obj.opcodes = serde::Deserializable<decltype(obj.opcodes)>::deserialize(deserializer);
    obj.private_parameters = serde::Deserializable<decltype(obj.private_parameters)>::deserialize(deserializer);
    obj.public_parameters = serde::Deserializable<decltype(obj.public_parameters)>::deserialize(deserializer);
    obj.return_values = serde::Deserializable<decltype(obj.return_values)>::deserialize(deserializer);
    obj.assert_messages = serde::Deserializable<decltype(obj.assert_messages)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

    inline bool operator==(const Directive &lhs, const Directive &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Directive::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Directive>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Directive Directive::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Directive>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Directive>::serialize(const Circuit::Directive &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::Directive serde::Deserializable<Circuit::Directive>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::Directive obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

    inline bool operator==(const Directive::ToLeRadix &lhs, const Directive::ToLeRadix &rhs) {
        if (!(lhs.a == rhs.a)) { return false; }
        if (!(lhs.b == rhs.b)) { return false; }
        if (!(lhs.radix == rhs.radix)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Directive::ToLeRadix::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Directive::ToLeRadix>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Directive::ToLeRadix Directive::ToLeRadix::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Directive::ToLeRadix>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Directive::ToLeRadix>::serialize(const Circuit::Directive::ToLeRadix &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.a)>::serialize(obj.a, serializer);
    serde::Serializable<decltype(obj.b)>::serialize(obj.b, serializer);
    serde::Serializable<decltype(obj.radix)>::serialize(obj.radix, serializer);
}

template <>
template <typename Deserializer>
Circuit::Directive::ToLeRadix serde::Deserializable<Circuit::Directive::ToLeRadix>::deserialize(Deserializer &deserializer) {
    Circuit::Directive::ToLeRadix obj;
    obj.a = serde::Deserializable<decltype(obj.a)>::deserialize(deserializer);
    obj.b = serde::Deserializable<decltype(obj.b)>::deserialize(deserializer);
    obj.radix = serde::Deserializable<decltype(obj.radix)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const Directive::PermutationSort &lhs, const Directive::PermutationSort &rhs) {
        if (!(lhs.inputs == rhs.inputs)) { return false; }
        if (!(lhs.tuple == rhs.tuple)) { return false; }
        if (!(lhs.bits == rhs.bits)) { return false; }
        if (!(lhs.sort_by == rhs.sort_by)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Directive::PermutationSort::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Directive::PermutationSort>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Directive::PermutationSort Directive::PermutationSort::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Directive::PermutationSort>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Directive::PermutationSort>::serialize(const Circuit::Directive::PermutationSort &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.inputs)>::serialize(obj.inputs, serializer);
    serde::Serializable<decltype(obj.tuple)>::serialize(obj.tuple, serializer);
    serde::Serializable<decltype(obj.bits)>::serialize(obj.bits, serializer);
    serde::Serializable<decltype(obj.sort_by)>::serialize(obj.sort_by, serializer);
}

template <>
template <typename Deserializer>
Circuit::Directive::PermutationSort serde::Deserializable<Circuit::Directive::PermutationSort>::deserialize(Deserializer &deserializer) {
    Circuit::Directive::PermutationSort obj;
    obj.inputs = serde::Deserializable<decltype(obj.inputs)>::deserialize(deserializer);
    obj.tuple = serde::Deserializable<decltype(obj.tuple)>::deserialize(deserializer);
    obj.bits = serde::Deserializable<decltype(obj.bits)>::deserialize(deserializer);
    obj.sort_by = serde::Deserializable<decltype(obj.sort_by)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Expression>::serialize(const Circuit::Expression &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.mul_terms)>::serialize(obj.mul_terms, serializer);
    serde::Serializable<decltype(obj.linear_combinations)>::serialize(obj.linear_combinations, serializer);
    serde::Serializable<decltype(obj.q_c)>::serialize(obj.q_c, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::Expression serde::Deserializable<Circuit::Expression>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::Expression obj;
    obj.mul_terms = serde::Deserializable<decltype(obj.mul_terms)>::deserialize(deserializer);
    obj.linear_combinations = serde::Deserializable<decltype(obj.linear_combinations)>::deserialize(deserializer);
    obj.q_c = serde::Deserializable<decltype(obj.q_c)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

    inline bool operator==(const FunctionInput &lhs, const FunctionInput &rhs) {
        if (!(lhs.witness == rhs.witness)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::FunctionInput>::serialize(const Circuit::FunctionInput &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.witness)>::serialize(obj.witness, serializer);
    serde::Serializable<decltype(obj.num_bits)>::serialize(obj.num_bits, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::FunctionInput serde::Deserializable<Circuit::FunctionInput>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::FunctionInput obj;
    obj.witness = serde::Deserializable<decltype(obj.witness)>::deserialize(deserializer);
    obj.num_bits = serde::Deserializable<decltype(obj.num_bits)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::HeapArray>::serialize(const Circuit::HeapArray &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.pointer)>::serialize(obj.pointer, serializer);
    serde::Serializable<decltype(obj.size)>::serialize(obj.size, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::HeapArray serde::Deserializable<Circuit::HeapArray>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::HeapArray obj;
    obj.pointer = serde::Deserializable<decltype(obj.pointer)>::deserialize(deserializer);
    obj.size = serde::Deserializable<decltype(obj.size)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::HeapVector>::serialize(const Circuit::HeapVector &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.pointer)>::serialize(obj.pointer, serializer);
    serde::Serializable<decltype(obj.size)>::serialize(obj.size, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::HeapVector serde::Deserializable<Circuit::HeapVector>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::HeapVector obj;
    obj.pointer = serde::Deserializable<decltype(obj.pointer)>::deserialize(deserializer);
    obj.size = serde::Deserializable<decltype(obj.size)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::MemOp>::serialize(const Circuit::MemOp &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.operation)>::serialize(obj.operation, serializer);
    serde::Serializable<decltype(obj.index)>::serialize(obj.index, serializer);
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::MemOp serde::Deserializable<Circuit::MemOp>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::MemOp obj;
    obj.operation = serde::Deserializable<decltype(obj.operation)>::deserialize(deserializer);
    obj.index = serde::Deserializable<decltype(obj.index)>::deserialize(deserializer);
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Opcode>::serialize(const Circuit::Opcode &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::Opcode serde::Deserializable<Circuit::Opcode>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::Opcode obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Opcode::AssertZero>::serialize(const Circuit::Opcode::AssertZero &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::Opcode::AssertZero serde::Deserializable<Circuit::Opcode::AssertZero>::deserialize(Deserializer &deserializer) {
    Circuit::Opcode::AssertZero obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Opcode::BlackBoxFuncCall>::serialize(const Circuit::Opcode::BlackBoxFuncCall &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::Opcode::BlackBoxFuncCall serde::Deserializable<Circuit::Opcode::BlackBoxFuncCall>::deserialize(Deserializer &deserializer) {
    Circuit::Opcode::BlackBoxFuncCall obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const Opcode::Directive &lhs, const Opcode::Directive &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::Directive::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::Directive>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::Directive Opcode::Directive::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::Directive>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Opcode::Directive>::serialize(const Circuit::Opcode::Directive &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::Opcode::Directive serde::Deserializable<Circuit::Opcode::Directive>::deserialize(Deserializer &deserializer) {
    Circuit::Opcode::Directive obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const Opcode::Brillig &lhs, const Opcode::Brillig &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Opcode::Brillig::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Opcode::Brillig>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Opcode::Brillig Opcode::Brillig::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Opcode::Brillig>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Opcode::Brillig>::serialize(const Circuit::Opcode::Brillig &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::Opcode::Brillig serde::Deserializable<Circuit::Opcode::Brillig>::deserialize(Deserializer &deserializer) {
    Circuit::Opcode::Brillig obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Opcode::MemoryOp>::serialize(const Circuit::Opcode::MemoryOp &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.block_id)>::serialize(obj.block_id, serializer);
    serde::Serializable<decltype(obj.op)>::serialize(obj.op, serializer);
    serde::Serializable<decltype(obj.predicate)>::serialize(obj.predicate, serializer);
}

template <>
template <typename Deserializer>
Circuit::Opcode::MemoryOp serde::Deserializable<Circuit::Opcode::MemoryOp>::deserialize(Deserializer &deserializer) {
    Circuit::Opcode::MemoryOp obj;
    obj.block_id = serde::Deserializable<decltype(obj.block_id)>::deserialize(deserializer);
    obj.op = serde::Deserializable<decltype(obj.op)>::deserialize(deserializer);
    obj.predicate = serde::Deserializable<decltype(obj.predicate)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const Opcode::MemoryInit &lhs, const Opcode::MemoryInit &rhs) {
        if (!(lhs.block_id == rhs.block_id)) { return false; }
        if (!(lhs.init == rhs.init)) { return false; }
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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Opcode::MemoryInit>::serialize(const Circuit::Opcode::MemoryInit &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.block_id)>::serialize(obj.block_id, serializer);
    serde::Serializable<decltype(obj.init)>::serialize(obj.init, serializer);
}

template <>
template <typename Deserializer>
Circuit::Opcode::MemoryInit serde::Deserializable<Circuit::Opcode::MemoryInit>::deserialize(Deserializer &deserializer) {
    Circuit::Opcode::MemoryInit obj;
    obj.block_id = serde::Deserializable<decltype(obj.block_id)>::deserialize(deserializer);
    obj.init = serde::Deserializable<decltype(obj.init)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::OpcodeLocation>::serialize(const Circuit::OpcodeLocation &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::OpcodeLocation serde::Deserializable<Circuit::OpcodeLocation>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::OpcodeLocation obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::OpcodeLocation::Acir>::serialize(const Circuit::OpcodeLocation::Acir &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::OpcodeLocation::Acir serde::Deserializable<Circuit::OpcodeLocation::Acir>::deserialize(Deserializer &deserializer) {
    Circuit::OpcodeLocation::Acir obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::OpcodeLocation::Brillig>::serialize(const Circuit::OpcodeLocation::Brillig &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.acir_index)>::serialize(obj.acir_index, serializer);
    serde::Serializable<decltype(obj.brillig_index)>::serialize(obj.brillig_index, serializer);
}

template <>
template <typename Deserializer>
Circuit::OpcodeLocation::Brillig serde::Deserializable<Circuit::OpcodeLocation::Brillig>::deserialize(Deserializer &deserializer) {
    Circuit::OpcodeLocation::Brillig obj;
    obj.acir_index = serde::Deserializable<decltype(obj.acir_index)>::deserialize(deserializer);
    obj.brillig_index = serde::Deserializable<decltype(obj.brillig_index)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::PublicInputs>::serialize(const Circuit::PublicInputs &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::PublicInputs serde::Deserializable<Circuit::PublicInputs>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::PublicInputs obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

    inline bool operator==(const RegisterIndex &lhs, const RegisterIndex &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> RegisterIndex::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<RegisterIndex>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline RegisterIndex RegisterIndex::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<RegisterIndex>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::RegisterIndex>::serialize(const Circuit::RegisterIndex &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::RegisterIndex serde::Deserializable<Circuit::RegisterIndex>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::RegisterIndex obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

    inline bool operator==(const RegisterOrMemory &lhs, const RegisterOrMemory &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> RegisterOrMemory::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<RegisterOrMemory>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline RegisterOrMemory RegisterOrMemory::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<RegisterOrMemory>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::RegisterOrMemory>::serialize(const Circuit::RegisterOrMemory &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::RegisterOrMemory serde::Deserializable<Circuit::RegisterOrMemory>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::RegisterOrMemory obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

    inline bool operator==(const RegisterOrMemory::RegisterIndex &lhs, const RegisterOrMemory::RegisterIndex &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> RegisterOrMemory::RegisterIndex::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<RegisterOrMemory::RegisterIndex>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline RegisterOrMemory::RegisterIndex RegisterOrMemory::RegisterIndex::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<RegisterOrMemory::RegisterIndex>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::RegisterOrMemory::RegisterIndex>::serialize(const Circuit::RegisterOrMemory::RegisterIndex &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::RegisterOrMemory::RegisterIndex serde::Deserializable<Circuit::RegisterOrMemory::RegisterIndex>::deserialize(Deserializer &deserializer) {
    Circuit::RegisterOrMemory::RegisterIndex obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const RegisterOrMemory::HeapArray &lhs, const RegisterOrMemory::HeapArray &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> RegisterOrMemory::HeapArray::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<RegisterOrMemory::HeapArray>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline RegisterOrMemory::HeapArray RegisterOrMemory::HeapArray::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<RegisterOrMemory::HeapArray>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::RegisterOrMemory::HeapArray>::serialize(const Circuit::RegisterOrMemory::HeapArray &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::RegisterOrMemory::HeapArray serde::Deserializable<Circuit::RegisterOrMemory::HeapArray>::deserialize(Deserializer &deserializer) {
    Circuit::RegisterOrMemory::HeapArray obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const RegisterOrMemory::HeapVector &lhs, const RegisterOrMemory::HeapVector &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> RegisterOrMemory::HeapVector::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<RegisterOrMemory::HeapVector>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline RegisterOrMemory::HeapVector RegisterOrMemory::HeapVector::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<RegisterOrMemory::HeapVector>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::RegisterOrMemory::HeapVector>::serialize(const Circuit::RegisterOrMemory::HeapVector &obj, Serializer &serializer) {
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
}

template <>
template <typename Deserializer>
Circuit::RegisterOrMemory::HeapVector serde::Deserializable<Circuit::RegisterOrMemory::HeapVector>::deserialize(Deserializer &deserializer) {
    Circuit::RegisterOrMemory::HeapVector obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    return obj;
}

namespace Circuit {

    inline bool operator==(const Value &lhs, const Value &rhs) {
        if (!(lhs.inner == rhs.inner)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Value::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Value>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Value Value::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Value>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Value>::serialize(const Circuit::Value &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.inner)>::serialize(obj.inner, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::Value serde::Deserializable<Circuit::Value>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::Value obj;
    obj.inner = serde::Deserializable<decltype(obj.inner)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Circuit {

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

} // end of namespace Circuit

template <>
template <typename Serializer>
void serde::Serializable<Circuit::Witness>::serialize(const Circuit::Witness &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Circuit::Witness serde::Deserializable<Circuit::Witness>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Circuit::Witness obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}
