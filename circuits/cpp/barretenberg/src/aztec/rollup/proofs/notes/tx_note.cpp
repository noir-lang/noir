#include "tx_note.hpp"
#include "pedersen_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <numeric/uint256/uint256.hpp>

using namespace barretenberg;

namespace rollup {
namespace proofs {
namespace notes {

grumpkin::g1::affine_element tx_note::encrypt_note() const
{
    grumpkin::g1::element p_1 = crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(value, 0);
    grumpkin::g1::element p_2 = crypto::pedersen::fixed_base_scalar_mul<250>(secret, 1);
    grumpkin::g1::element p_4 = crypto::pedersen::fixed_base_scalar_mul<32>((uint64_t)asset_id, 2);
    grumpkin::g1::element sum;
    if (value > 0) {
        sum = p_1 + p_2;
    } else {
        sum = p_2;
    }
    if (asset_id > 0)
    {
        sum += p_4;
    }
    grumpkin::g1::affine_element p_3 =
        crypto::pedersen::compress_to_point_native(owner.x, owner.y, 3);

    sum += p_3;

    sum = sum.normalize();

    return { sum.x, sum.y };
}

uint128_t tx_note::compute_nullifier(const uint32_t tree_index, const bool is_real_note) const
{
    const auto enc_note = encrypt_note();
    std::vector<barretenberg::fr> buf{
        enc_note.x,
        secret,
        barretenberg::fr(uint256_t((uint64_t)tree_index) + (uint256_t(is_real_note) << 64)),
    };
    uint256_t result = (crypto::pedersen::compress_native(buf, rollup::proofs::notes::TX_NOTE_NULLIFIER_INDEX));
    return uint128_t(result);
}

/**
 * Brute force decryption up to values of 1000.
 */
bool decrypt_note(grumpkin::g1::affine_element const& encrypted_note,
                  grumpkin::fr const& private_key,
                  fr const& viewing_key,
                  uint32_t const asset_id,
                  uint256_t& r)
{
    grumpkin::g1::affine_element public_key = grumpkin::g1::one * private_key;
    for (uint256_t value = 0; value <= 1000; ++value) {
        grumpkin::g1::element p_1 = crypto::pedersen::fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(value, 0);
        grumpkin::g1::element p_2 = crypto::pedersen::fixed_base_scalar_mul<250>(viewing_key, 1);
        grumpkin::g1::element p_4 = crypto::pedersen::fixed_base_scalar_mul<32>((uint64_t)asset_id, 2);

        grumpkin::g1::element sum;
        if (value > 0) {
            sum = p_1 + p_2;
        } else {
            sum = p_2;
        }
        sum += p_4;
        grumpkin::g1::affine_element p_3 = crypto::pedersen::encrypt_native({ public_key.x, public_key.y }, 3);

        sum += p_3;

        if (grumpkin::g1::affine_element(sum) == encrypted_note) {
            r = value;
            return true;
        }
    }

    return false;
}

} // namespace notes
} // namespace proofs
} // namespace rollup