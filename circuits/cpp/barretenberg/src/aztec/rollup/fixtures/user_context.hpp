#pragma once
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <crypto/schnorr/schnorr.hpp>

namespace rollup {
namespace fixtures {

typedef crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> grumpkin_key_pair;

struct user_context {
    barretenberg::fr note_secret;
    grumpkin_key_pair owner;
    grumpkin_key_pair signing_keys[2];
    barretenberg::fr alias_hash;
};

inline barretenberg::fr generate_alias_hash(std::string const& alias)
{
    std::vector<uint8_t> alias_buffer;
    auto alias_hash = blake2::blake2s({ alias.begin(), alias.end() });
    alias_buffer.resize(4, 0);
    alias_buffer.insert(alias_buffer.end(), alias_hash.begin(), alias_hash.end() - 4);
    return from_buffer<barretenberg::fr>(alias_buffer);
}

inline barretenberg::fr generate_account_id(barretenberg::fr const& alias_hash, uint32_t nonce = 0)
{
    return alias_hash + (barretenberg::fr{ (uint64_t)nonce } * barretenberg::fr(2).pow(224));
}

inline grumpkin_key_pair create_key_pair(numeric::random::Engine* engine)
{
    grumpkin::fr priv_key = grumpkin::fr::random_element(engine);
    grumpkin::g1::affine_element pub_key = grumpkin::g1::one * priv_key;
    return { priv_key, pub_key };
}

inline user_context create_user_context(numeric::random::Engine* engine = nullptr)
{
    uint8_t vk[] = { 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11,
                     0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11 };
    barretenberg::fr note_secret = barretenberg::fr::serialize_from_buffer(vk);
    auto alias_hash = generate_alias_hash("pebble");
    return { note_secret, create_key_pair(engine), { create_key_pair(engine), create_key_pair(engine) }, alias_hash };
}

} // namespace fixtures
} // namespace rollup