#include <rollup/pedersen_note/pedersen_note.hpp>
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace rollup {
namespace client_proofs {
namespace escape_hatch {

using namespace rollup::pedersen_note;

bool verify_signature(Composer& composer,
                      std::array<public_note, 2> const& notes,
                      grumpkin::g1::affine_element const& pub_key,
                      crypto::schnorr::signature const& sig)
{
    point_ct owner_pub_key = { witness_ct(&composer, pub_key.x), witness_ct(&composer, pub_key.y) };
    stdlib::schnorr::signature_bits signature = stdlib::schnorr::convert_signature(&composer, sig);
    std::array<field_ct, 4> to_compress;
    for (size_t i = 0; i < 2; ++i) {
        to_compress[i * 2] = notes[i].ciphertext.x;
        to_compress[i * 2 + 1] = notes[i].ciphertext.y;
    }
    byte_array_ct message = pedersen::compress(to_compress);
    return stdlib::schnorr::verify_signature(message, owner_pub_key, signature);
}

} // namespace escape_hatch
} // namespace client_proofs
} // namespace rollup