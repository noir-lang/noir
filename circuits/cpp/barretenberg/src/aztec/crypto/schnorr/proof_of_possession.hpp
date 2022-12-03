#pragma once

#include <utility>

#include "common/serialize.hpp"
#include "schnorr.hpp"

namespace crypto::schnorr {

/**
 * @brief A proof of possession is a Schnorr proof of knowledge of a secret key corresponding to a given public key.
 *
 * @details This implementation follows the specification detailed in https://eprint.iacr.org/2021/1375.pdf
 *
 * @tparam G1 group over which the key pair was generated
 * @tparam Hash function used to derive the Fiat-Shamir challenge
 */
template <typename G1, typename Hash> struct ProofOfPossession {
    using Fq = typename G1::coordinate_field;
    using Fr = typename G1::subgroup_field;
    using affine_element = typename G1::affine_element;
    using element = typename G1::element;
    using key_pair = crypto::schnorr::key_pair<Fr, G1>;

    // challenge = e = H_reg(pk,R)
    std::array<uint8_t, 32> challenge;
    // response = z = k - e * sk
    Fr response = Fr::zero();

    // restore default constructor to enable deserialization
    ProofOfPossession() = default;

    /**
     * @brief Create a new proof of possession for a given account.
     *
     * @warning Proofs are not deterministic.
     *
     * @param account a key_pair (secret_key, public_key)
     */
    ProofOfPossession(const key_pair& account)
    {
        auto secret_key = account.private_key;
        auto public_key = account.public_key;

        Fr k = Fr::random_element();

        affine_element R = G1::one * k;

        auto challenge_bytes = generate_challenge(public_key, R);
        std::copy(challenge_bytes.begin(), challenge_bytes.end(), challenge.begin());

        Fr challenge_fr = Fr::serialize_from_buffer(&challenge_bytes[0]);
        response = k - challenge_fr * secret_key;
    }

    /**
     * @brief verifies that an unserialized signature is valid
     *
     * @param public_key the public key for which this proof is intended
     * @return whether the proof is correct
     */
    bool verify(const affine_element& public_key) const
    {
        Fr challenge_fr = Fr::serialize_from_buffer(&challenge[0]);
        // this ensures that a default constructed proof is invalid
        if (response.is_zero())
            return false;

        if (!public_key.on_curve() || public_key.is_point_at_infinity())
            return false;

        // R = e•pk + z•G
        affine_element R = element(public_key) * challenge_fr + G1::one * response;
        if (R.is_point_at_infinity())
            return false;

        // recompute the challenge e
        auto challenge_computed = generate_challenge(public_key, R);
        return std::equal(challenge.begin(), challenge.end(), challenge_computed.begin(), challenge_computed.end());
    }

  private:
    /**
     * @brief Generate the Fiat-Shamir challenge e = H_reg(G,X,R)
     *
     * @param public_key X = secret_key•G
     * @param R the commitment R = k•G
     * @return e = H_reg(X,R)
     */
    static auto generate_challenge(const affine_element& public_key, const affine_element& R)
    {
        // Domain separation challenges
        const std::string domain_separator_pop("h_reg");

        // buffer containing (domain_sep, G, X, R)
        std::vector<uint8_t> challenge_buf;

        // write domain separator
        std::copy(domain_separator_pop.begin(), domain_separator_pop.end(), std::back_inserter(challenge_buf));

        // write the group generator
        write(challenge_buf, G1::affine_one);

        // write X (only once, differing from the paper)
        write(challenge_buf, public_key);

        // write R
        write(challenge_buf, R);

        // generate the raw bits of H_reg(X,R)
        return Hash::hash(challenge_buf);
    }
};

template <typename B, typename G1, typename Hash>
inline void read(B& it, ProofOfPossession<G1, Hash>& proof_of_possession)
{
    read(it, proof_of_possession.challenge);
    read(it, proof_of_possession.response);
}

template <typename B, typename G1, typename Hash>
inline void write(B& buf, ProofOfPossession<G1, Hash> const& proof_of_possession)
{
    write(buf, proof_of_possession.challenge);
    write(buf, proof_of_possession.response);
}

} // namespace crypto::schnorr