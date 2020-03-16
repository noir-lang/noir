#pragma once

#include <numeric/uint256/uint256.hpp>

namespace crypto {
namespace ecdsa {

template <typename Hash, typename Fq, typename Fr, typename G1>
signature construct_signature(const std::string& message, const key_pair<Fr, G1>& account)
{
    signature sig;
    Fr k = Fr::random_element(); // TODO replace with HMAC
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
    Fr r = Fr::serialize_from_buffer(&sig.r[0]);
    Fr s = Fr::serialize_from_buffer(&sig.s[0]);

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