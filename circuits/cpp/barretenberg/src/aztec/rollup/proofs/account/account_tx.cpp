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
        return compress_native({ new_account_public_key.x, new_account_public_key.y },
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

void write(std::vector<uint8_t>& buf, account_tx const& tx)
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
    write(buf, tx.signature);
}

void read(uint8_t const*& buf, account_tx& tx)
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
    read(buf, tx.signature);
}

std::ostream& operator<<(std::ostream& os, account_tx const& tx)
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
