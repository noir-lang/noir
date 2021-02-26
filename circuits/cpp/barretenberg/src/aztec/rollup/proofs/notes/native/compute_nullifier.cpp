#include "compute_nullifier.hpp"
#include "../constants.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/blake2s/blake2s.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

using namespace barretenberg;

fr compute_nullifier(grumpkin::g1::affine_element const& encrypted_note,
                     const uint32_t tree_index,
                     grumpkin::fr const& account_private_key,
                     const bool is_real_note)
{
    auto hashed_pk =
        crypto::pedersen::fixed_base_scalar_mul<254>(fr(account_private_key), TX_NOTE_ACCOUNT_PRIVATE_KEY_INDEX);

    std::vector<barretenberg::fr> buf{
        encrypted_note.x,
        hashed_pk.x,
        hashed_pk.y,
        barretenberg::fr(uint256_t((uint64_t)tree_index) + (uint256_t(is_real_note) << 64)),
    };
    auto result = crypto::pedersen::encrypt_native(buf, TX_NOTE_NULLIFIER_INDEX);
    auto blake_input = to_buffer(result.x);
    auto blake_input_b = to_buffer(result.y);
    std::copy(blake_input_b.begin(), blake_input_b.end(), std::back_inserter(blake_input));
    auto blake_result = blake2::blake2s(blake_input);
    return from_buffer<fr>(blake_result);
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup