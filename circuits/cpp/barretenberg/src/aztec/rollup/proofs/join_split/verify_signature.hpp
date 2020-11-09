#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace notes;

bool verify_signature(std::array<point_ct, 4> const& notes,
                      field_ct const& output_owner,
                      point_ct const& owner_pub_key,
                      schnorr::signature_bits const& signature)
{
    std::array<field_ct, 9> to_compress;
    for (size_t i = 0; i < 4; ++i) {
        to_compress[i * 2] = notes[i].x;
        to_compress[i * 2 + 1] = notes[i].y;
    }
    to_compress[8] = output_owner;
    byte_array_ct message = pedersen::compress(to_compress);
    return verify_signature(message, owner_pub_key, signature);
}

} // namespace join_split
} // namespace proofs
} // namespace rollup