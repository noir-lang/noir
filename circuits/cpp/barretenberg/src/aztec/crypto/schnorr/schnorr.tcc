#pragma once

#include <common/serialize.hpp>
#include <ecc/curves/bn254/fq.hpp>
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/fields/field.hpp>

#include <crypto/hmac/hmac.hpp>

namespace crypto {
namespace schnorr {

/**
 * @brief Construct a Schnorr signature of the form (random - priv * hash, hash) using the group G1.
 *
 * @tparam Hash: A function std::vector<uint8_t> -> std::array<uint8_t, 32>
 * @tparam Fq:   The field over which points of G1 are defined.
 * @tparam Fr:   A class with a random element generator, where the multiplication
 * G1::one * k is defined for any randomly-generated class member.
 * @tparam G1:   A group with a generator G1:one, where an element R is assumed
 * to posses an 'x-coordinate' R.x lying in the field Fq. It is also assumed that
 * G1 comes with a notion of an 'affine element'.
 * @param message A standard library string reference.
 * @param account A private key-public key pair in Fr × {affine elements of G1}.
 * @return signature
 */
template <typename Hash, typename Fq, typename Fr, typename G1>
signature construct_signature(const std::string& message, const key_pair<Fr, G1>& account)
{
    signature sig;
    auto& public_key = account.public_key;
    auto& private_key = account.private_key;

    // use HMAC in PRF mode to derive 32-byte secret `k`
    std::vector<uint8_t> pkey_buffer;
    write(pkey_buffer, private_key);

    Fr k = crypto::get_unbiased_field_from_hmac<Hash, Fr>(message, pkey_buffer);

    typename G1::affine_element R(G1::one * k);

    std::vector<uint8_t> message_buffer;

    /**
     * Normal schorr param e = H(r.x || pub_key || message)
     * But we want to keep hash preimage to <= 64 bytes for a 32 byte message
     * (for performance reasons in our join-split circuit!)
     *
     * barretenberg schnorr defines e as the following:
     *
     * e = H(pedersen(r.x || pub_key.x || pub_key.y), message)
     *
     * pedersen is collision resistant => e can be modelled as randomly distributed
     * as long as H can be modelled as a random oracle
     */
    Fq compressed_keys = crypto::pedersen::compress_native({ R.x, public_key.x, public_key.y });
    write(message_buffer, compressed_keys);
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));

    auto ev = Hash::hash(message_buffer);
    std::copy(ev.begin(), ev.end(), sig.e.begin());

    Fr e = Fr::serialize_from_buffer(&sig.e[0]);

    Fr s = k - (private_key * e);
    Fr::serialize_to_buffer(s, &sig.s[0]);
    return sig;
}

/**
 * @brief Verify a Schnorr signature of the sort produced by construct_signature.
 */
template <typename Hash, typename Fq, typename Fr, typename G1>
bool verify_signature(const std::string& message, const typename G1::affine_element& public_key, const signature& sig)
{
    using affine_element = typename G1::affine_element;
    using element = typename G1::element;

    if (!public_key.on_curve() || public_key.is_point_at_infinity()) {
        return false;
    }
    // e = H(pedersen(r, pk.x, pk.y), m) r = x(R)
    // R = g^s • pub^e
    Fr s = Fr::serialize_from_buffer(&sig.s[0]);
    Fr source_e = Fr::serialize_from_buffer(&sig.e[0]);

    if (s == 0 || source_e == 0) {
        return false;
    }
    affine_element R(element(public_key) * source_e + G1::one * s);
    if (R.is_point_at_infinity()) {
        // this result implies k == 0
        return false;
    }
    Fq compressed_keys = crypto::pedersen::compress_native({ R.x, public_key.x, public_key.y });

    std::vector<uint8_t> message_buffer;
    write(message_buffer, compressed_keys);
    std::copy(message.begin(), message.end(), std::back_inserter(message_buffer));

    auto e_vec = Hash::hash(message_buffer);
    Fr target_e = Fr::serialize_from_buffer(&e_vec[0]);

    return source_e == target_e;
}
} // namespace schnorr
} // namespace crypto