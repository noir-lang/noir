#pragma once

#include <crypto/hmac/hmac.hpp>
#include <crypto/pedersen/pedersen.hpp>

#include "schnorr.hpp"

namespace crypto {
namespace schnorr {

/**
 * @brief Generate the schnorr signature challenge parameter `e` given a message, signer pubkey and nonce
 *
 * @details Normal Schnorr param e = H(R.x || pubkey || message)
 * But we want to keep hash preimage to <= 64 bytes for a 32 byte message
 * (for performance reasons in our join-split circuit!)
 *
 * barretenberg schnorr defines e as the following:
 *
 * e = H(pedersen(R.x || pubkey.x || pubkey.y), message)
 *
 * pedersen is collision resistant => e can be modelled as randomly distributed
 * as long as H can be modelled as a random oracle
 *
 * @tparam Hash the hash-function used as random-oracle
 * @tparam G1 Group over which the signature is produced
 * @param message what are we signing over?
 * @param pubkey the pubkey of the signer
 * @param R the nonce
 * @return e = H(pedersen(R.x || pubkey.x || pubkey.y), message) as a 256-bit integer,
 *      represented in a container of 32 uint8_t's
 *
 *
 * @warning When the order of G1 is significantly smaller than 2²⁵⁶−1,
 * the distribution of `e` is no longer uniform over `Fr`. This mainly affects
 * the ZK property of the scheme. If signatures are never revealed (i.e. if they
 * are always private inputs to circuits) then nothing would be revealed anyway.
 */
template <typename Hash, typename G1>
static auto generate_schnorr_challenge(const std::string& message,
                                       const typename G1::affine_element& pubkey,
                                       const typename G1::affine_element& R)
{
    using Fq = typename G1::coordinate_field;
    // create challenge message pedersen_hash(R.x, pubkey)
    Fq compressed_keys = crypto::pedersen::compress_native({ R.x, pubkey.x, pubkey.y });
    std::vector<uint8_t> e_buffer;
    write(e_buffer, compressed_keys);
    std::copy(message.begin(), message.end(), std::back_inserter(e_buffer));

    // hash the result of the pedersen hash digest
    // we return auto since some hash implementation return
    // either a std::vector or a std::array with 32 bytes
    return Hash::hash(e_buffer);
}

/**
 * @brief Construct a Schnorr signature of the form (random - priv * hash, hash) using the group G1.
 *
 * @warning Proofs are not deterministic.
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
    // sanity check to ensure our hash function produces `e_raw`
    // of exactly 32 bytes.
    static_assert(Hash::OUTPUT_SIZE == 32);

    auto& public_key = account.public_key;
    auto& private_key = account.private_key;

    // sample random nonce k
    Fr k = Fr::random_element();

    typename G1::affine_element R(G1::one * k);

    auto e_raw = generate_schnorr_challenge<Hash, G1>(message, public_key, R);
    // the conversion from e_raw results in a biased field element e
    Fr e = Fr::serialize_from_buffer(&e_raw[0]);
    Fr s = k - (private_key * e);

    // we serialize e_raw rather than e, so that no binary conversion needs to be
    // performed during verification.
    // indeed, e_raw defines an integer exponent which exponentiates the public_key point.
    // if we define e_uint as the integers whose binary representation is e_raw,
    // and e = e_uint % r, where r is the order of the curve,
    // and pk as the point representing the public_key,
    // then e•pk = e_uint•pk
    signature sig;
    Fr::serialize_to_buffer(s, &sig.s[0]);
    std::copy(e_raw.begin(), e_raw.end(), sig.e.begin());
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
    // Deserializing from a 256-bit buffer will induce a bias on the order of
    // 1/(2(256-log(r))) where r is the order of Fr, since we perform a modular reduction
    Fr e = Fr::serialize_from_buffer(&sig.e[0]);

    // reading s in this way always applies the modular reduction, and
    // therefore a signature where (r,s') where s'=s+Fr::modulus would also be accepted
    // this makes our signatures malleable, but is not an issue in the context of the
    // circuits where we use these signatures
    Fr s = Fr::serialize_from_buffer(&sig.s[0]);

    if (s == 0 || e == 0) {
        return false;
    }

    // R = g^{sig.s} • pub^{sig.e}
    affine_element R(element(public_key) * e + G1::one * s);
    if (R.is_point_at_infinity()) {
        // this result implies k == 0, which would be catastrophic for the prover.
        // it is a cheap check that ensures this doesn't happen.
        return false;
    }

    // compare the _hashes_ rather than field elements modulo r

    // e = H(pedersen(r, pk.x, pk.y), m), where r = x(R)
    auto target_e = generate_schnorr_challenge<Hash, G1>(message, public_key, R);
    return std::equal(sig.e.begin(), sig.e.end(), target_e.begin(), target_e.end());
}
} // namespace schnorr
} // namespace crypto