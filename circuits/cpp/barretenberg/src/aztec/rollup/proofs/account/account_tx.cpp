#include "account_tx.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include "../notes/constants.hpp"

namespace rollup {
namespace proofs {
namespace account {

using namespace barretenberg;
using namespace crypto::schnorr;
using namespace crypto::pedersen;

fr account_tx::compute_account_alias_hash_nullifier() const
{
    if (create) {
        return compress_native({ alias_hash }, rollup::proofs::notes::GeneratorIndex::ACCOUNT_ALIAS_HASH_NULLIFIER);
    }
    return 0;
}

fr account_tx::compute_account_public_key_nullifier() const
{
    if (create || migrate) {
        return compress_native({ new_account_public_key.x },
                               rollup::proofs::notes::GeneratorIndex::ACCOUNT_PUBLIC_KEY_NULLIFIER);
    }
    return 0;
}

void account_tx::sign(key_pair<grumpkin::fr, grumpkin::g1> const& keys)
{
    auto nullifier_1 = compute_account_alias_hash_nullifier();
    auto nullifier_2 = compute_account_public_key_nullifier();
    std::vector<grumpkin::fq> to_compress = {
        alias_hash,  account_public_key.x, new_account_public_key.x, new_signing_pub_key_1.x, new_signing_pub_key_2.x,
        nullifier_1, nullifier_2
    };
    fr compressed = compress_native(to_compress);
    auto message = to_buffer(compressed);
    signing_pub_key = keys.public_key;
    signature = crypto::schnorr::construct_signature<Blake2sHasher, grumpkin::fq, grumpkin::fr, grumpkin::g1>(
        std::string(message.begin(), message.end()), keys);
}

} // namespace account
} // namespace proofs
} // namespace rollup
