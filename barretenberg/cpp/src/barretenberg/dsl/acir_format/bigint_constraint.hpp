#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>
#include <vector>

namespace acir_format {

struct BigIntFromLeBytes {
    std::vector<uint32_t> inputs;
    std::vector<uint32_t> modulus;
    uint32_t result;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(inputs, result);
    friend bool operator==(BigIntFromLeBytes const& lhs, BigIntFromLeBytes const& rhs) = default;
};

enum BigIntOperationType { Add, Neg, Mul, Div };

struct BigIntOperation {
    uint32_t lhs;
    uint32_t rhs;
    uint32_t result;
    BigIntOperationType opcode;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(lhs, rhs, opcode, result);
    friend bool operator==(BigIntOperation const& lhs, BigIntOperation const& rhs) = default;
};

template <typename Builder> void create_bigint_operations_constraint(Builder& builder, const BigIntOperation& input);
template <typename Builder>
void create_bigint_from_le_bytes_constraint(Builder& builder, const BigIntFromLeBytes& input);
} // namespace acir_format