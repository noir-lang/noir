#include "./pedersen_note.hpp"
#include "../pedersen/pedersen.hpp"

using namespace barretenberg;

namespace crypto {
namespace pedersen_note {
grumpkin::g1::affine_element encrypt_note(const private_note& plaintext)
{
    grumpkin::g1::element p_1 = pedersen::fixed_base_scalar_mul<32>(uint256_t(plaintext.value, 0, 0, 0), 0);
    grumpkin::g1::element p_2 = pedersen::fixed_base_scalar_mul<250>(plaintext.secret, 1);

    grumpkin::g1::element sum;
    if (plaintext.value > 0) {
        sum = p_1 + p_2;
    } else {
        sum = p_2;
    }
    grumpkin::g1::affine_element p_3 = pedersen::compress_to_point_native(plaintext.owner.x, plaintext.owner.y, 0);

    sum += p_3;

    sum = sum.normalize();

    return { sum.x, sum.y };
}
} // namespace pedersen_note
} // namespace crypto