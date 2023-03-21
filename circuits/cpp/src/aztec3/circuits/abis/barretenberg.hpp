#pragma once
#include <common/serialize.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/recursion/verifier/verifier.hpp>

namespace serialize {

inline void read(uint8_t const*& it, plonk::stdlib::types::NativeTypes::Proof& proof)
{
    using serialize::read;

    read(it, proof.proof_data);
};

inline void read(uint8_t const*& it, plonk::stdlib::types::NativeTypes::AggregationObject& obj)
{
    using serialize::read;

    read(it, obj.P0);
    read(it, obj.P1);
    read(it, obj.public_inputs);
    read(it, obj.proof_witness_indices);
    read(it, obj.has_data);
};

} // namespace serialize