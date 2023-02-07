#include "compute_nullifier.hpp"
#include "../../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/blake2s/blake2s.hpp>

namespace join_split_example {
namespace proofs {
namespace notes {
namespace native {

using namespace barretenberg;

/**
 * Computes a nullifier for a _value_ note
 */
fr compute_nullifier(grumpkin::fq const& note_commitment,
                     grumpkin::fr const& account_private_key,
                     const bool is_note_in_use)
{
    auto hashed_pk = crypto::pedersen::fixed_base_scalar_mul<254>(
        fr(account_private_key), GeneratorIndex::JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY);

    std::vector<barretenberg::fr> buf{
        note_commitment,
        hashed_pk.x,
        hashed_pk.y,
        is_note_in_use,
    };
    auto compressed_inputs = crypto::pedersen::compress_native(buf, GeneratorIndex::JOIN_SPLIT_NULLIFIER);
    auto blake_result = blake2::blake2s(to_buffer(compressed_inputs));

    return from_buffer<fr>(blake_result);
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace join_split_example