#pragma once

#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "../../primitives/uint/uint.hpp"
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
namespace proof_system::plonk {
namespace stdlib {
namespace ecdsa {

template <typename Composer> struct signature {
    stdlib::byte_array<Composer> r;
    stdlib::byte_array<Composer> s;
    stdlib::uint8<Composer> v;
};

template <typename Composer, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Composer> verify_signature(const stdlib::byte_array<Composer>& message,
                                  const G1& public_key,
                                  const signature<Composer>& sig);

template <typename Composer, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Composer> verify_signature_noassert(const stdlib::byte_array<Composer>& message,
                                           const G1& public_key,
                                           const signature<Composer>& sig);
template <typename Composer, typename Curve, typename Fq, typename Fr, typename G1>
bool_t<Composer> verify_signature_prehashed_message_noassert(const stdlib::byte_array<Composer>& hashed_message,
                                                             const G1& public_key,
                                                             const signature<Composer>& sig);

template <typename Composer>
static signature<Composer> from_witness(Composer* ctx, const crypto::ecdsa::signature& input)
{
    std::vector<uint8_t> r_vec(std::begin(input.r), std::end(input.r));
    std::vector<uint8_t> s_vec(std::begin(input.s), std::end(input.s));
    stdlib::byte_array<Composer> r(ctx, r_vec);
    stdlib::byte_array<Composer> s(ctx, s_vec);
    stdlib::uint8<Composer> v(ctx, input.v);
    signature<Composer> out;
    out.r = r;
    out.s = s;
    out.v = v;
    return out;
}

} // namespace ecdsa
} // namespace stdlib
} // namespace proof_system::plonk

#include "./ecdsa_impl.hpp"