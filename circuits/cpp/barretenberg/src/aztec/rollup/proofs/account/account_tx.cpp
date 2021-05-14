#include "account_tx.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace proofs {
namespace account {

using namespace barretenberg;
using namespace crypto::schnorr;
using namespace crypto::pedersen;

void account_tx::sign(key_pair<grumpkin::fr, grumpkin::g1> const& keys)
{
    std::vector<grumpkin::fq> to_compress = {
        account_alias_id(),      account_public_key.x,    new_account_public_key.x,
        new_signing_pub_key_1.x, new_signing_pub_key_2.x,
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
    write(buf, tx.num_new_keys);
    write(buf, tx.new_signing_pub_key_1);
    write(buf, tx.new_signing_pub_key_2);
    write(buf, tx.alias_hash);
    write(buf, tx.nonce);
    write(buf, tx.migrate);
    write(buf, tx.gibberish);
    write(buf, tx.account_index);
    write(buf, tx.account_path);
    write(buf, tx.signing_pub_key);
    write(buf, tx.signature);
}

void read(uint8_t const*& buf, account_tx& tx)
{
    using serialize::read;
    read(buf, tx.merkle_root);
    read(buf, tx.account_public_key);
    read(buf, tx.new_account_public_key);
    read(buf, tx.num_new_keys);
    read(buf, tx.new_signing_pub_key_1);
    read(buf, tx.new_signing_pub_key_2);
    read(buf, tx.alias_hash);
    read(buf, tx.nonce);
    read(buf, tx.migrate);
    read(buf, tx.gibberish);
    read(buf, tx.account_index);
    read(buf, tx.account_path);
    read(buf, tx.signing_pub_key);
    read(buf, tx.signature);
}

std::ostream& operator<<(std::ostream& os, account_tx const& tx)
{
    return os << "merkle_root: " << tx.merkle_root << "\n"
              << "account_public_key: " << tx.account_public_key << "\n"
              << "new_account_public_key: " << tx.new_account_public_key << "\n"
              << "num_new_keys: " << tx.num_new_keys << "\n"
              << "new_signing_pub_key_1: " << tx.new_signing_pub_key_1 << "\n"
              << "new_signing_pub_key_2: " << tx.new_signing_pub_key_2 << "\n"
              << "alias_hash: " << tx.alias_hash << "\n"
              << "nonce: " << tx.nonce << "\n"
              << "migrate: " << tx.migrate << "\n"
              << "gibberish: " << tx.gibberish << "\n"
              << "account_index: " << tx.account_index << "\n"
              << "account_path: " << tx.account_path << "\n"
              << "signing_pub_key: " << tx.signing_pub_key << "\n"
              << "signature: " << tx.signature << "\n";
}

} // namespace account
} // namespace proofs
} // namespace rollup
