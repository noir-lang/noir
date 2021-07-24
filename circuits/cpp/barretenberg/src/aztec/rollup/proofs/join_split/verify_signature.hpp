#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace notes;

inline bool verify_signature(field_ct const& public_input,
                             field_ct const& public_output,
                             field_ct const& asset_id,
                             field_ct const& output_note1_commitment,
                             field_ct const& output_note2_commitment,
                             field_ct const& nullifier1,
                             field_ct const& nullifier2,
                             field_ct const& tx_fee,
                             point_ct const& owner_pub_key,
                             field_ct const& input_owner,
                             field_ct const& output_owner,
                             schnorr::signature_bits const& signature)
{
    std::vector<field_ct> to_compress = {
        public_input, public_output, asset_id, output_note1_commitment, output_note2_commitment, nullifier1, nullifier2,
        input_owner,  output_owner,  tx_fee,
    };
    byte_array_ct message = pedersen::compress(to_compress, true);
    return verify_signature(message, owner_pub_key, signature);
}

} // namespace join_split
} // namespace proofs
} // namespace rollup