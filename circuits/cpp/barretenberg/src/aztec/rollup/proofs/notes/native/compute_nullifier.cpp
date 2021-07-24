#include "compute_nullifier.hpp"
#include "../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/blake2s/blake2s.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

using namespace barretenberg;

fr compute_nullifier(grumpkin::fq const& note_commitment,
                     const uint32_t tree_index,
                     grumpkin::fr const& account_private_key,
                     const bool is_real_note)
{
    auto hashed_pk = crypto::pedersen::fixed_base_scalar_mul<254>(
        fr(account_private_key), GeneratorIndex::JOIN_SPLIT_NULLIFIER_ACCOUNT_PRIVATE_KEY);

    std::vector<barretenberg::fr> buf{
        note_commitment,
        hashed_pk.x,
        hashed_pk.y,
        barretenberg::fr(uint256_t((uint64_t)tree_index) + (uint256_t(is_real_note) << 64)),
    };
    auto result = crypto::pedersen::commit_native(buf, GeneratorIndex::JOIN_SPLIT_NULLIFIER);
    auto blake_result = blake2::blake2s(to_buffer(result));

    return from_buffer<fr>(blake_result);
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup