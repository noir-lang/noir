#pragma once

#include "../hmac/hmac.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"

namespace bb::crypto {

template <typename Hash, typename Fq, typename Fr, typename G1>
ecdsa_signature ecdsa_construct_signature(const std::string& message, const ecdsa_key_pair<Fr, G1>& account)
{
    ecdsa_signature sig;

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

    // Ensure that the value of s is "low", i.e. s := min{ s_fr, (|Fr| - s_fr) }
    const bool is_s_low = (uint256_t(s_fr) < (uint256_t(Fr::modulus) / 2));
    uint256_t s_uint256 = is_s_low ? uint256_t(s_fr) : (uint256_t(Fr::modulus) - uint256_t(s_fr));

    Fr::serialize_to_buffer(Fr(s_uint256), &sig.s[0]);

    // compute recovery_id: given R = (x, y)
    //   0: y is even  &&  x < |Fr|
    //   1: y is odd   &&  x < |Fr|
    //   2: y is even  &&  |Fr| <= x < |Fq|
    //   3: y is odd   &&  |Fr| <= x < |Fq|
    // v = offset + recovery_id
    Fq r_fq = Fq(R.x);
    bool is_r_finite = (uint256_t(r_fq) == uint256_t(r_fr));
    bool y_parity = uint256_t(R.y).get_bit(0);
    bool recovery_bit = y_parity ^ is_s_low;
    constexpr uint8_t offset = 27;

    int value = offset + recovery_bit + static_cast<uint8_t>(2) * !is_r_finite;
    ASSERT(value <= UINT8_MAX);
    sig.v = static_cast<uint8_t>(value);
    return sig;
}

template <typename Hash, typename Fq, typename Fr, typename G1>
typename G1::affine_element ecdsa_recover_public_key(const std::string& message, const ecdsa_signature& sig)
{
    using serialize::read;
    uint256_t r_uint;
    uint256_t s_uint;
    uint8_t v_uint;
    uint256_t mod = uint256_t(Fr::modulus);

    const auto* r_buf = &sig.r[0];
    const auto* s_buf = &sig.s[0];
    const auto* v_buf = &sig.v;
    read(r_buf, r_uint);
    read(s_buf, s_uint);
    read(v_buf, v_uint);

    // We need to check that r and s are in Field according to specification
    if ((r_uint >= mod) || (s_uint >= mod)) {
        throw_or_abort("r or s value exceeds the modulus");
    }
    if ((r_uint == 0) || (s_uint == 0)) {
        throw_or_abort("r or s value is zero");
    }

    // Check that the s value is less than |Fr| / 2
    if (s_uint * 2 > mod) {
        throw_or_abort("s value is not less than curve order by 2");
    }

    // Check that v must either be in {27, 28, 29, 30}
    Fr r = Fr(r_uint);
    Fr s = Fr(s_uint);
    Fq r_fq = Fq(r_uint);
    bool is_r_finite = true;

    if ((v_uint == 27) || (v_uint == 28)) {
        ASSERT(uint256_t(r) == uint256_t(r_fq));
    } else if ((v_uint == 29) || (v_uint == 30)) {
        ASSERT(uint256_t(r) < uint256_t(r_fq));
        is_r_finite = false;
    } else {
        throw_or_abort("v value is not in {27, 28, 29, 30}");
    }

    // Decompress the x-coordinate r_uint to get two possible R points
    // The first uncompressed R point is selected when r < |Fr|
    // The second uncompressed R point is selected when |Fr| <= r < |Fq|
    // Note that the second condition can occur with probability 1/2^128 so its highly unlikely.
    auto uncompressed_points = G1::affine_element::from_compressed_unsafe(r_uint);
    typename G1::affine_element point_R = uncompressed_points[!is_r_finite];

    // Negate the y-coordinate point of R based on the parity of v
    bool y_parity_R = uint256_t(point_R.y).get_bit(0);
    if ((v_uint & 1) ^ y_parity_R) {
        point_R.y = -point_R.y;
    }

    // Start key recovery algorithm
    std::vector<uint8_t> message_buffer;
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));
    auto ev = Hash::hash(message_buffer);
    Fr z = Fr::serialize_from_buffer(&ev[0]);

    Fr r_inv = r.invert();

    Fr u1 = -(z * r_inv);
    Fr u2 = s * r_inv;

    typename G1::affine_element recovered_public_key(typename G1::element(point_R) * u2 + G1::one * u1);
    return recovered_public_key;
}

template <typename Hash, typename Fq, typename Fr, typename G1>
bool ecdsa_verify_signature(const std::string& message,
                            const typename G1::affine_element& public_key,
                            const ecdsa_signature& sig)
{
    using serialize::read;
    uint256_t r_uint;
    uint256_t s_uint;
    uint256_t mod = uint256_t(Fr::modulus);
    if (!public_key.on_curve()) {
        return false;
    }
    const auto* r_buf = &sig.r[0];
    const auto* s_buf = &sig.s[0];
    read(r_buf, r_uint);
    read(s_buf, s_uint);
    // We need to check that r and s are in Field according to specification
    if ((r_uint >= mod) || (s_uint >= mod)) {
        return false;
    }
    if ((r_uint == 0) || (s_uint == 0)) {
        return false;
    }

    // Check that the s value is less than |Fr| / 2
    if (s_uint * 2 > mod) {
        throw_or_abort("s value is not less than curve order by 2");
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
} // namespace bb::crypto
