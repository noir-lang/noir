#include "tx_note.hpp"
#include <crypto/pedersen/pedersen.hpp>

using namespace barretenberg;

namespace rollup {
namespace client_proofs {
namespace join_split {

grumpkin::g1::affine_element encrypt_note(const tx_note& plaintext)
{
    grumpkin::g1::element p_1 = crypto::pedersen::fixed_base_scalar_mul<32>(uint256_t(plaintext.value, 0, 0, 0), 0);
    grumpkin::g1::element p_2 = crypto::pedersen::fixed_base_scalar_mul<250>(plaintext.secret, 1);

    grumpkin::g1::element sum;
    if (plaintext.value > 0) {
        sum = p_1 + p_2;
    } else {
        sum = p_2;
    }
    grumpkin::g1::affine_element p_3 =
        crypto::pedersen::compress_to_point_native(plaintext.owner.x, plaintext.owner.y, 0);

    sum += p_3;

    sum = sum.normalize();

    return { sum.x, sum.y };
}

bool decrypt_note(grumpkin::g1::affine_element const& encrypted_note,
                  grumpkin::fr const& private_key,
                  fr const& viewing_key,
                  uint32_t& r)
{
    grumpkin::g1::affine_element public_key = grumpkin::g1::one * private_key;
    for (uint32_t value = 0; value <= 1000; ++value) {
        grumpkin::g1::element p_1 = crypto::pedersen::fixed_base_scalar_mul<32>(uint256_t(value, 0, 0, 0), 0);
        grumpkin::g1::element p_2 = crypto::pedersen::fixed_base_scalar_mul<250>(viewing_key, 1);

        grumpkin::g1::element sum;
        if (value > 0) {
            sum = p_1 + p_2;
        } else {
            sum = p_2;
        }
        grumpkin::g1::affine_element p_3 = crypto::pedersen::encrypt_native({ public_key.x, public_key.y }, 0);

        sum += p_3;

        sum = sum.normalize();

        if (sum == encrypted_note) {
            r = value;
            return true;
        }
    }

    return false;
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup