#pragma once

#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "../../primitives/uint/uint.hpp"
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
namespace bb::stdlib {

template <typename Builder> struct ecdsa_signature {
    stdlib::byte_array<Builder> r;
    stdlib::byte_array<Builder> s;
    stdlib::uint8<Builder> v;
};

template <typename Builder, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Builder> ecdsa_verify_signature(const stdlib::byte_array<Builder>& message,
                                       const G1& public_key,
                                       const ecdsa_signature<Builder>& sig);

template <typename Builder, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Builder> ecdsa_verify_signature_noassert(const stdlib::byte_array<Builder>& message,
                                                const G1& public_key,
                                                const ecdsa_signature<Builder>& sig);
template <typename Builder, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Builder> ecdsa_verify_signature_prehashed_message_noassert(const stdlib::byte_array<Builder>& hashed_message,
                                                                  const G1& public_key,
                                                                  const ecdsa_signature<Builder>& sig);

template <typename Builder>
static ecdsa_signature<Builder> ecdsa_from_witness(Builder* ctx, const crypto::ecdsa_signature& input)
{
    std::vector<uint8_t> r_vec(std::begin(input.r), std::end(input.r));
    std::vector<uint8_t> s_vec(std::begin(input.s), std::end(input.s));
    stdlib::byte_array<Builder> r(ctx, r_vec);
    stdlib::byte_array<Builder> s(ctx, s_vec);
    stdlib::uint8<Builder> v(ctx, input.v);
    ecdsa_signature<Builder> out;
    out.r = r;
    out.s = s;
    out.v = v;
    return out;
}

} // namespace bb::stdlib

#include "./ecdsa_impl.hpp"