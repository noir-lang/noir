#pragma once
#include "barretenberg/crypto/schnorr/schnorr.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace join_split_example {
namespace fixtures {

typedef crypto::schnorr::key_pair<grumpkin::fr, grumpkin::g1> grumpkin_key_pair;

struct user_context {
    bb::fr note_secret;
    grumpkin_key_pair owner;
    grumpkin_key_pair signing_keys[2];
    bb::fr alias_hash;
};

inline bb::fr generate_alias_hash(std::string const& alias)
{
    std::vector<uint8_t> inputv(alias.begin(), alias.end());
    auto output = blake2::blake2s(inputv);
    return bb::fr(uint256_t(from_buffer<bb::fr>(output.data())) >> 32);
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
    bb::fr note_secret = bb::fr::serialize_from_buffer(vk);
    auto alias_hash = generate_alias_hash("pebble");
    return { note_secret, create_key_pair(engine), { create_key_pair(engine), create_key_pair(engine) }, alias_hash };
}

} // namespace fixtures
} // namespace join_split_example
