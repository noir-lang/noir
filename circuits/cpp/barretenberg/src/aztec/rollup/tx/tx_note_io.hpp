#pragma once
#include <common/net.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <crypto/pedersen/pedersen.hpp>
#include "tx_note.hpp"

namespace rollup {
namespace tx {

inline barretenberg::fr hton(barretenberg::fr const& value)
{
    barretenberg::fr input = value.from_montgomery_form();
    barretenberg::fr be_value;
    be_value.data[0] = htonll(input.data[3]);
    be_value.data[1] = htonll(input.data[2]);
    be_value.data[2] = htonll(input.data[1]);
    be_value.data[3] = htonll(input.data[0]);
    return be_value;
}

inline barretenberg::fr ntoh(barretenberg::fr const& be_value)
{
    barretenberg::fr value;
    value.data[0] = ntohll(be_value.data[3]);
    value.data[1] = ntohll(be_value.data[2]);
    value.data[2] = ntohll(be_value.data[1]);
    value.data[3] = ntohll(be_value.data[0]);
    return value.to_montgomery_form();
}

inline grumpkin::g1::affine_element hton(grumpkin::g1::affine_element const& value)
{
    return { hton(value.x), hton(value.y) };
}

inline grumpkin::g1::affine_element ntoh(grumpkin::g1::affine_element const& value)
{
    return { ntoh(value.x), ntoh(value.y) };
}

inline tx_note hton(tx_note const& value)
{
    return { hton(value.owner), htonl(value.value), hton(value.secret) };
}

inline tx_note ntoh(tx_note const& value)
{
    return { ntoh(value.owner), ntohl(value.value), ntoh(value.secret) };
}

} // namespace rollup
}