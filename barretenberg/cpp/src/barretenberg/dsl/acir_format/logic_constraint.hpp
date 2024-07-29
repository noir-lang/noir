#pragma once
#include "barretenberg/dsl/acir_format/witness_constant.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"
#include <cstdint>

namespace acir_format {

using Builder = bb::UltraCircuitBuilder;

struct LogicConstraint {
    WitnessOrConstant<bb::fr> a;
    WitnessOrConstant<bb::fr> b;
    uint32_t result;
    uint32_t num_bits;
    uint32_t is_xor_gate;

    friend bool operator==(LogicConstraint const& lhs, LogicConstraint const& rhs) = default;

    // for serialization, update with any new fields
    MSGPACK_FIELDS(a, b, result, num_bits, is_xor_gate);
};

template <typename Builder>
void create_logic_gate(Builder& builder,
                       WitnessOrConstant<bb::fr> a,
                       WitnessOrConstant<bb::fr> b,
                       uint32_t result,
                       std::size_t num_bits,
                       bool is_xor_gate);

void xor_gate(Builder& builder, WitnessOrConstant<bb::fr> a, WitnessOrConstant<bb::fr> b, uint32_t result);

void and_gate(Builder& builder, WitnessOrConstant<bb::fr> a, WitnessOrConstant<bb::fr> b, uint32_t result);
} // namespace acir_format
