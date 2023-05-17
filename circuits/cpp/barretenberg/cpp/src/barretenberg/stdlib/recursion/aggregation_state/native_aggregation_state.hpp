#pragma once
#include "barretenberg/ecc/groups/affine_element.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/common/streams.hpp"

namespace proof_system::plonk {
namespace stdlib {
namespace recursion {

/**
 * Native aggregation state contains the following:
 *   (P0, P1): the aggregated elements storing the verification results of proofs in the past
 *   proof_witness_indices: witness indices that point to (P0, P1)
 *   public_inputs: the public inputs of the inner proof, these become the private inputs to the recursive circuit
 *   has_data: indicates if this aggregation state contain past (P0, P1)
 */
struct native_aggregation_state {
    typename barretenberg::g1::affine_element P0 = barretenberg::g1::affine_one;
    typename barretenberg::g1::affine_element P1 = barretenberg::g1::affine_one;
    std::vector<barretenberg::fr> public_inputs;
    std::vector<uint32_t> proof_witness_indices;
    bool has_data = false;

    MSGPACK_FIELDS(P0, P1, public_inputs, proof_witness_indices, has_data);
    bool operator==(native_aggregation_state const& other) const
    {
        return P0 == other.P0 && P1 == other.P1 && public_inputs == other.public_inputs &&
               proof_witness_indices == other.proof_witness_indices && has_data == other.has_data;
    };
};

inline void read(uint8_t const*& it, native_aggregation_state& state)
{
    using serialize::read;

    read(it, state.P0);
    read(it, state.P1);
    read(it, state.public_inputs);
    read(it, state.proof_witness_indices);
    read(it, state.has_data);
};

template <typename B> inline void write(B& buf, native_aggregation_state const& state)
{
    using serialize::write;
    write(buf, state.P0);
    write(buf, state.P1);
    write(buf, state.public_inputs);
    write(buf, state.proof_witness_indices);
    write(buf, state.has_data);
}

inline std::ostream& operator<<(std::ostream& os, native_aggregation_state const& obj)
{
    return os << "P0: " << obj.P0 << "\n"
              << "P1: " << obj.P1 << "\n"
              << "public_inputs: " << obj.public_inputs << "\n"
              << "proof_witness_indices: " << obj.proof_witness_indices << "\n"
              << "has_data: " << obj.has_data << "\n";
};

} // namespace recursion
} // namespace stdlib
} // namespace proof_system::plonk