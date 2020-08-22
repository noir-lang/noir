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
};

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
    return { note_secret, create_key_pair(engine), { create_key_pair(engine), create_key_pair(engine) } };
}

} // namespace fixtures
} // namespace rollup