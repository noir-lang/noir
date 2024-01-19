#pragma once
#include "barretenberg/common/streams.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/ecc/groups/affine_element.hpp"

namespace bb::plonk {
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
    typename bb::g1::affine_element P0 = bb::g1::affine_one;
    typename bb::g1::affine_element P1 = bb::g1::affine_one;
    std::vector<bb::fr> public_inputs;
    std::vector<uint32_t> proof_witness_indices;
    bool has_data = false;

    // For serialization, update with new fields
    MSGPACK_FIELDS(P0, P1, public_inputs, proof_witness_indices, has_data);
    bool operator==(native_aggregation_state const& other) const
    {
        return P0 == other.P0 && P1 == other.P1 && public_inputs == other.public_inputs &&
               proof_witness_indices == other.proof_witness_indices && has_data == other.has_data;
    };
};

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
} // namespace bb::plonk
