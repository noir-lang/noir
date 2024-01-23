#include "barretenberg/stdlib/encryption/schnorr/schnorr.hpp"
#include "barretenberg/stdlib/hash/pedersen/pedersen.hpp"

namespace bb::join_split_example::proofs::join_split {

using namespace notes;

inline void verify_signature(field_ct const& public_value,
                             field_ct const& public_owner,
                             field_ct const& public_asset_id,
                             field_ct const& output_note1_commitment,
                             field_ct const& output_note2_commitment,
                             field_ct const& nullifier1,
                             field_ct const& nullifier2,
                             group_ct const& owner_pub_key,
                             field_ct const& backward_link,
                             field_ct const& allow_chain,
                             schnorr_signature_bits const& signature)
{
    std::vector<field_ct> to_compress = {
        public_value, public_owner,  public_asset_id, output_note1_commitment, output_note2_commitment, nullifier1,
        nullifier2,   backward_link, allow_chain,
    };
    byte_array_ct message = pedersen_hash::hash(to_compress);
    schnorr_verify_signature(message, owner_pub_key, signature);
}

} // namespace bb::join_split_example::proofs::join_split
