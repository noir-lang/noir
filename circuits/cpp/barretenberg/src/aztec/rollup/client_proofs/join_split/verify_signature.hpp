#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include <rollup/pedersen_note/pedersen_note.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace rollup::pedersen_note;

void verify_signature(Composer& composer,
                      std::array<public_note, 4> const& notes,
                      grumpkin::g1::affine_element const& pub_key,
                      crypto::schnorr::signature const& sig)
{
    point owner_pub_key = { witness_ct(&composer, pub_key.x), witness_ct(&composer, pub_key.y) };
    stdlib::schnorr::signature_bits signature = stdlib::schnorr::convert_signature(&composer, sig);
    std::array<field_ct, 8> to_compress;
    for (size_t i = 0; i < 4; ++i) {
        info("note_ct ", i, " :", notes[i].ciphertext.x, " ", notes[i].ciphertext.y);
        to_compress[i * 2] = notes[i].ciphertext.x;
        to_compress[i * 2 + 1] = notes[i].ciphertext.y;
    }
    byte_array_ct message = plonk::stdlib::pedersen::compress_eight(to_compress);
    byte_array_ct message2(&composer, message.bits().rbegin(), message.bits().rend());
    std::cout << "message " << message << std::endl;
    std::cout << "message rev " << message2 << std::endl;
    stdlib::schnorr::verify_signature(message2, owner_pub_key, signature);
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup