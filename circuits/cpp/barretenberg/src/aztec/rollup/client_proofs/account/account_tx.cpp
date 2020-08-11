#include "account_tx.hpp"
#include <crypto/pedersen/pedersen.hpp>

namespace rollup {
namespace client_proofs {
namespace account {

using namespace barretenberg;
using namespace crypto::schnorr;
using namespace crypto::pedersen;

void account_tx::sign(key_pair<grumpkin::fr, grumpkin::g1> const& keys)
{
    std::vector<grumpkin::fq> to_compress = {
        owner_pub_key.x, new_signing_pub_key_1.x, new_signing_pub_key_2.x, alias, nullified_key.x
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
    write(buf, tx.owner_pub_key);
    write(buf, tx.num_new_keys);
    write(buf, tx.new_signing_pub_key_1);
    write(buf, tx.new_signing_pub_key_2);
    write(buf, tx.register_alias);
    write(buf, tx.alias);
    write(buf, tx.nullify_key);
    write(buf, tx.nullified_key);
    write(buf, tx.account_index);
    write(buf, tx.signing_pub_key);
    write(buf, tx.account_path);
    write(buf, tx.signature);
}

void read(uint8_t const*& buf, account_tx& tx)
{
    using serialize::read;
    read(buf, tx.merkle_root);
    read(buf, tx.owner_pub_key);
    read(buf, tx.num_new_keys);
    read(buf, tx.new_signing_pub_key_1);
    read(buf, tx.new_signing_pub_key_2);
    read(buf, tx.register_alias);
    read(buf, tx.alias);
    read(buf, tx.nullify_key);
    read(buf, tx.nullified_key);
    read(buf, tx.account_index);
    read(buf, tx.signing_pub_key);
    read(buf, tx.account_path);
    read(buf, tx.signature);
}

bool operator==(account_tx const& lhs, account_tx const& rhs)
{
    // clang-format off
    return lhs.merkle_root == rhs.merkle_root
        && lhs.owner_pub_key == rhs.owner_pub_key
        && lhs.num_new_keys == rhs.num_new_keys
        && lhs.new_signing_pub_key_1 == rhs.new_signing_pub_key_1
        && lhs.new_signing_pub_key_2 == rhs.new_signing_pub_key_2
        && lhs.register_alias == rhs.register_alias
        && lhs.alias == rhs.alias
        && lhs.nullify_key == rhs.nullify_key
        && lhs.nullified_key == rhs.nullified_key
        && lhs.account_index == rhs.account_index
        && lhs.signing_pub_key == rhs.signing_pub_key
        && lhs.account_path == rhs.account_path
        && lhs.signature == rhs.signature;
    // clang-format on
}

std::ostream& operator<<(std::ostream& os, account_tx const& tx)
{
    return os << "merkle_root: " << tx.merkle_root << "\n"
              << "owner_pub_key: " << tx.owner_pub_key << "\n"
              << "num_new_keys: " << tx.num_new_keys << "\n"
              << "new_signing_pub_key_1: " << tx.new_signing_pub_key_1 << "\n"
              << "new_signing_pub_key_2: " << tx.new_signing_pub_key_2 << "\n"
              << "register_alias: " << tx.register_alias << "\n"
              << "alias: " << tx.alias << "\n"
              << "nullify_key: " << tx.nullify_key << "\n"
              << "nullified_key: " << tx.nullified_key << "\n"
              << "account_index: " << tx.account_index << "\n"
              << "signing_pub_key: " << tx.signing_pub_key << "\n"
              << "account_path: " << tx.account_path << "\n"
              << "signature: " << tx.signature << "\n";
}

} // namespace account
} // namespace client_proofs
} // namespace rollup
