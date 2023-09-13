#pragma once
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include <cstdint>

namespace acir_format {

struct LogicConstraint {
    uint32_t a;
    uint32_t b;
    uint32_t result;
    uint32_t num_bits;
    uint32_t is_xor_gate;

    friend bool operator==(LogicConstraint const& lhs, LogicConstraint const& rhs) = default;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(a, b, result, num_bits, is_xor_gate);
};

void create_logic_gate(Builder& builder, uint32_t a, uint32_t b, uint32_t result, size_t num_bits, bool is_xor_gate);

void xor_gate(Builder& builder, uint32_t a, uint32_t b, uint32_t result);

void and_gate(Builder& builder, uint32_t a, uint32_t b, uint32_t result);
} // namespace acir_format
