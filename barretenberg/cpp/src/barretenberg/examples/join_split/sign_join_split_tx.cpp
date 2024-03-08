#include "sign_join_split_tx.hpp"
#include "barretenberg/crypto/schnorr/schnorr.hpp"
#include "compute_signing_data.hpp"

namespace bb::join_split_example::proofs::join_split {

using namespace bb::crypto;

schnorr_signature sign_join_split_tx(join_split_tx const& tx, schnorr_key_pair<grumpkin::fr, grumpkin::g1> const& keys)
{
    fr hashed = compute_signing_data(tx);

    std::vector<uint8_t> message(sizeof(fr));
    fr::serialize_to_buffer(hashed, &message[0]);

    crypto::schnorr_signature signature =
        crypto::schnorr_construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
            std::string(message.begin(), message.end()), keys);

    auto result = crypto::schnorr_verify_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        std::string(message.begin(), message.end()), keys.public_key, signature);
    ASSERT(result == true);
    return signature;
}

} // namespace bb::join_split_example::proofs::join_split
