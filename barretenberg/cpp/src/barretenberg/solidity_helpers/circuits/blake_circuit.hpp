#pragma once
#include "barretenberg/stdlib/hash/blake2s/blake2s.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"

template <typename Builder> class BlakeCircuit {
  public:
    using field_ct = bb::stdlib::field_t<Builder>;
    using public_witness_ct = bb::stdlib::public_witness_t<Builder>;
    using byte_array_ct = bb::stdlib::byte_array<Builder>;

    static constexpr size_t NUM_PUBLIC_INPUTS = 4;

    static Builder generate(uint256_t public_inputs[])
    {
        Builder builder;

        byte_array_ct input_buffer(&builder);
        for (size_t i = 0; i < NUM_PUBLIC_INPUTS; ++i) {
            input_buffer.write(byte_array_ct(field_ct(public_witness_ct(&builder, public_inputs[i]))));
        }

        bb::stdlib::blake2s<Builder>(input_buffer);

        return builder;
    }
};
