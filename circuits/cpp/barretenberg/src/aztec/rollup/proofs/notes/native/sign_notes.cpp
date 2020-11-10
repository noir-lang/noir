#include "sign_notes.hpp"
#include "encrypt_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace proofs {
namespace notes {
namespace native {

using namespace crypto::schnorr;
using namespace crypto::pedersen;

signature sign_notes(std::array<value_note, 4> const& notes,
                     fr const& output_owner,
                     key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                     numeric::random::Engine* engine)
{
    std::array<grumpkin::fq, 9> to_compress;
    for (size_t i = 0; i < 4; ++i) {
        auto encrypted = encrypt_note(notes[i]);
        to_compress[i * 2] = encrypted.x;
        to_compress[i * 2 + 1] = encrypted.y;
    }
    to_compress[8] = output_owner;
    fr compressed = compress_native(to_compress);
    std::vector<uint8_t> message(sizeof(fr));
    fr::serialize_to_buffer(compressed, &message[0]);

    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            std::string(message.begin(), message.end()), keys, engine);
    return signature;
}

} // namespace native
} // namespace notes
} // namespace proofs
} // namespace rollup