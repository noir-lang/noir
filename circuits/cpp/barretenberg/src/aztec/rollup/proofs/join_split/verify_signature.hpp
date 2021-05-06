#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace notes;

inline bool verify_signature(join_split_inputs const& inputs,
                             point_ct const& encrypted_output_note1,
                             point_ct const& encrypted_output_note2,
                             field_ct const& nullifier1,
                             field_ct const& nullifier2,
                             field_ct const& tx_fee,
                             point_ct const& owner_pub_key,
                             schnorr::signature_bits const& signature)
{
    // format message to contain:
    // * input value
    // * output value
    // * asset_id
    // * output note 1 ciphertext
    // * output note 2 ciphertext
    // * input note 1 nullifier
    // * input note 2 nullifier
    // * input owner
    // * output owner
    // * tx_fee
    std::vector<field_ct> to_compress;

    to_compress.push_back(inputs.public_input);
    to_compress.push_back(inputs.public_output);
    to_compress.push_back(inputs.asset_id);
    to_compress.push_back(encrypted_output_note1.x);
    to_compress.push_back(encrypted_output_note1.y);
    to_compress.push_back(encrypted_output_note2.x);
    to_compress.push_back(encrypted_output_note2.y);
    to_compress.push_back(nullifier1);
    to_compress.push_back(nullifier2);
    to_compress.push_back(inputs.input_owner);
    to_compress.push_back(inputs.output_owner);
    to_compress.push_back(tx_fee);

    byte_array_ct message = pedersen::compress(to_compress, true);
    return verify_signature(message, owner_pub_key, signature);
}

} // namespace join_split
} // namespace proofs
} // namespace rollup