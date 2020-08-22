#include "tx_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace client_proofs {
namespace join_split {

using namespace crypto::schnorr;
using namespace crypto::pedersen;

signature sign_notes(std::array<tx_note, 4> const& notes,
                     key_pair<grumpkin::fr, grumpkin::g1> const& keys,
                     numeric::random::Engine* engine)
{
    std::array<grumpkin::fq, 8> to_compress;
    for (size_t i = 0; i < 4; ++i) {
        auto encrypted = encrypt_note(notes[i]);
        to_compress[i * 2] = encrypted.x;
        to_compress[i * 2 + 1] = encrypted.y;
    }
    fr compressed = compress_native(to_compress);
    std::vector<uint8_t> message(sizeof(fr));
    fr::serialize_to_buffer(compressed, &message[0]);
    crypto::schnorr::signature signature =
        crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            std::string(message.begin(), message.end()), keys, engine);
    return signature;
}

} // namespace join_split
} // namespace client_proofs
} // namespace rollup