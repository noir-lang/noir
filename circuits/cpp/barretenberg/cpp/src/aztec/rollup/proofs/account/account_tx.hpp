#pragma once
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/hash_path.hpp>
#include <stdlib/types/types.hpp>

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
    bool create;
    bool migrate;

    uint32_t account_note_index;
    plonk::stdlib::merkle_tree::fr_hash_path account_note_path;
    grumpkin::g1::affine_element signing_pub_key = grumpkin::g1::affine_one;
    crypto::schnorr::signature signature;

    fr compute_account_alias_hash_nullifier() const;
    fr compute_account_public_key_nullifier() const;
    void sign(crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> const& keys);

    bool operator==(account_tx const&) const = default;
};

template <typename B> inline void read(B& buf, account_tx& tx)
{
    using serialize::read;
    read(buf, tx.merkle_root);
    read(buf, tx.account_public_key);
    read(buf, tx.new_account_public_key);
    read(buf, tx.new_signing_pub_key_1);
    read(buf, tx.new_signing_pub_key_2);
    read(buf, tx.alias_hash);
    read(buf, tx.create);
    read(buf, tx.migrate);
    read(buf, tx.account_note_index);
    read(buf, tx.account_note_path);
    read(buf, tx.signing_pub_key);
    read(buf, tx.signature.s);
    read(buf, tx.signature.e);
}

template <typename B> inline void write(B& buf, account_tx const& tx)
{
    using serialize::write;
    write(buf, tx.merkle_root);
    write(buf, tx.account_public_key);
    write(buf, tx.new_account_public_key);
    write(buf, tx.new_signing_pub_key_1);
    write(buf, tx.new_signing_pub_key_2);
    write(buf, tx.alias_hash);
    write(buf, tx.create);
    write(buf, tx.migrate);
    write(buf, tx.account_note_index);
    write(buf, tx.account_note_path);
    write(buf, tx.signing_pub_key);
    write(buf, tx.signature.s);
    write(buf, tx.signature.e);
}

inline std::ostream& operator<<(std::ostream& os, account_tx const& tx)
{
    return os << "merkle_root: " << tx.merkle_root << "\n"
              << "account_public_key: " << tx.account_public_key << "\n"
              << "new_account_public_key: " << tx.new_account_public_key << "\n"
              << "new_signing_pub_key_1: " << tx.new_signing_pub_key_1 << "\n"
              << "new_signing_pub_key_2: " << tx.new_signing_pub_key_2 << "\n"
              << "alias_hash: " << tx.alias_hash << "\n"
              << "create: " << tx.create << "\n"
              << "migrate: " << tx.migrate << "\n"
              << "account_note_index: " << tx.account_note_index << "\n"
              << "account_note_path: " << tx.account_note_path << "\n"
              << "signing_pub_key: " << tx.signing_pub_key << "\n"
              << "signature: " << tx.signature << "\n";
}

} // namespace account
} // namespace proofs
} // namespace rollup
