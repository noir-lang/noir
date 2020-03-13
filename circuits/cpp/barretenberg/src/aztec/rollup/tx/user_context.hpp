#pragma once
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace tx {

struct user_context {
    barretenberg::fr note_secret;
    grumpkin::fr private_key;
    grumpkin::g1::affine_element public_key;
};

inline user_context create_user_context()
{
    barretenberg::fr note_secret = { { 0x11111111, 0x11111111, 0x11111111, 0x11111111 } };
    grumpkin::fr owner_secret = { { 0x55555555, 0x55555555, 0x55555555, 0x55555555 } };
    grumpkin::g1::affine_element owner_pub_key = grumpkin::g1::one * owner_secret;
    return { note_secret, owner_secret, owner_pub_key };
}

} // namespace tx
} // namespace rollup