#pragma once
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace proofs {
namespace account {

using namespace barretenberg;
using namespace crypto::schnorr;

struct account_tx {
    barretenberg::fr merkle_root;
    grumpkin::g1::affine_element account_public_key = grumpkin::g1::affine_one;
    grumpkin::g1::affine_element new_account_public_key = grumpkin::g1::affine_one;
    grumpkin::g1::affine_element new_signing_pub_key_1 = grumpkin::g1::affine_one;
    grumpkin::g1::affine_element new_signing_pub_key_2 = grumpkin::g1::affine_one;
    barretenberg::fr alias_hash;
    uint32_t nonce;
    bool migrate;

    uint32_t account_index;
    plonk::stdlib::merkle_tree::fr_hash_path account_path;
    grumpkin::g1::affine_element signing_pub_key = grumpkin::g1::affine_one;
    crypto::schnorr::signature signature;

    barretenberg::fr account_alias_id() const
    {
        return alias_hash + (barretenberg::fr{ (uint64_t)nonce } * barretenberg::fr(2).pow(224));
    }

    void sign(crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> const& keys);

    bool operator==(account_tx const&) const = default;
};

void read(uint8_t const*& it, account_tx& tx);
void write(std::vector<uint8_t>& buf, account_tx const& tx);

std::ostream& operator<<(std::ostream& os, account_tx const& tx);

} // namespace account
} // namespace proofs
} // namespace rollup
