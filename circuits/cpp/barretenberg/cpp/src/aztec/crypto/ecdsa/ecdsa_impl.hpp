#pragma once

#include <numeric/uint256/uint256.hpp>
#include <common/serialize.hpp>
#include "../hmac/hmac.hpp"

namespace crypto {
namespace ecdsa {

template <typename Hash, typename Fq, typename Fr, typename G1>
signature construct_signature(const std::string& message, const key_pair<Fr, G1>& account)
{
    signature sig;

    // use HMAC in PRF mode to derive 32-byte secret `k`
    std::vector<uint8_t> pkey_buffer;
    write(pkey_buffer, account.private_key);
    Fr k = crypto::get_unbiased_field_from_hmac<Hash, Fr>(message, pkey_buffer);

    typename G1::affine_element R(G1::one * k);
    Fq::serialize_to_buffer(R.x, &sig.r[0]);

    std::vector<uint8_t> message_buffer;
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));
    auto ev = Hash::hash(message_buffer);

    Fr z = Fr::serialize_from_buffer(&ev[0]);
    Fr r_fr = Fr::serialize_from_buffer(&sig.r[0]);
    Fr s_fr = (z + r_fr * account.private_key) / k;

    Fr::serialize_to_buffer(s_fr, &sig.s[0]);
    return sig;
}

template <typename Hash, typename Fq, typename Fr, typename G1>
bool verify_signature(const std::string& message, const typename G1::affine_element& public_key, const signature& sig)
{
    using serialize::read;
    uint256_t r_uint;
    uint256_t s_uint;
    if (!public_key.on_curve()) {
        return false;
    }
    const auto* r_buf = &sig.r[0];
    const auto* s_buf = &sig.s[0];
    read(r_buf, r_uint);
    read(s_buf, s_uint);
    // We need to check that r and s are in Field according to specification
    if ((r_uint >= Fr::modulus) || (s_uint >= Fr::modulus)) {
        return false;
    }
    if ((r_uint == 0) || (s_uint == 0)) {
        return false;
    }
    Fr r = Fr(r_uint);
    Fr s = Fr(s_uint);

    std::vector<uint8_t> message_buffer;
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));
    auto ev = Hash::hash(message_buffer);
    Fr z = Fr::serialize_from_buffer(&ev[0]);

    Fr s_inv = s.invert();

    Fr u1 = z * s_inv;
    Fr u2 = r * s_inv;

    typename G1::affine_element R(typename G1::element(public_key) * u2 + G1::one * u1);
    uint256_t Rx(R.x);
    Fr result(Rx);
    return result == r;
}
} // namespace ecdsa
} // namespace crypto