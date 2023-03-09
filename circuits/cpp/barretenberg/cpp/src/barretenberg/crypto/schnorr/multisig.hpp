#pragma once

#include <algorithm>
#include <cstdint>
#include <numeric>
#include <optional>
#include <utility>
#include <vector>

#include "schnorr.hpp"
#include "proof_of_possession.hpp"

namespace crypto::schnorr {

/**
 * @brief Implements the SpeedyMuSig protocol; a secure 2-round interactive multisignature scheme
 * whose signature outputs can be verified by a regular Schnorr verification algorithm.
 *
 * @tparam G1 The elliptic curve group being used to generate the multisignature
 * @tparam HashRegNon Hash function used to model H_reg and H_non. It must be different from H_sig for proper domain
 * separation.
 * @tparam HashSig Hash function used generate the Fiat-Shamir challenge for the signature (H_sig).
 *
 * @details SpeedyMuSig paper at https://eprint.iacr.org/2021/1375.pdf
 */
template <typename G1, typename HashRegNon, typename HashSig = Blake2sHasher> class multisig {

    // ensure that a different hash function is used for signature and proof of possession/nonce.
    // we can apply domain separation for HashRegNon but not for HashSig, so this ensures all hash functions
    // are modeled as different random oracles.
    static_assert(!std::is_same_v<HashRegNon, HashSig>);

  public:
    using Fq = typename G1::coordinate_field;
    using Fr = typename G1::subgroup_field;
    using affine_element = typename G1::affine_element;
    using element = typename G1::element;
    using key_pair = crypto::schnorr::key_pair<Fr, G1>;

    /**
     * @brief MultiSigPublicKey wraps a signer's public key g1::affine_element
     * along with a proof of posession: a signature whose message is the public key,
     * signed by the corresponding private key.
     *
     * This is to prevent attacks where an attacker presents a key that they do not know the secret to
     * (e.g. the attacker public key is a linear combination of other public keys)
     *
     */
    struct MultiSigPublicKey {
        affine_element public_key = G1::affine_point_at_infinity;
        // proof of knowledge of the secret_key for public_key
        ProofOfPossession<G1, HashRegNon> proof_of_possession;

        // restore default constructor to enable deserialization
        MultiSigPublicKey() = default;
        // create a MultiSigPublicKey with a proof of possession associated with public_key of account
        MultiSigPublicKey(const key_pair& account)
            : public_key(account.public_key)
            , proof_of_possession(account)
        {}
    };

    struct RoundOnePrivateOutput {
        Fr r;
        Fr s;
    };

    struct RoundOnePublicOutput {
        // R = r⋅G
        affine_element R;
        // S = s⋅G
        affine_element S;

        // for std::sort
        bool operator<(const RoundOnePublicOutput& other) const
        {
            return ((R < other.R) || ((R == other.R) && S < (other.S)));
        }

        bool operator==(const RoundOnePublicOutput& other) const { return (R == other.R) && (S == other.S); }
    };
    // corresponds to z = r + as - ex,
    using RoundTwoPublicOutput = Fr;

  private:
    /**
     * @brief given a list of commitments to nonces produced in round 1, we check that all points are valid and that the
     * list does not contain duplicates
     *
     * @param round1_public_outputs a list of pairs of points {(R1,S1), ...., (Rn,Sn)}
     * @return bool whether or not the list is valid.
     */
    static bool valid_round1_nonces(const std::vector<RoundOnePublicOutput>& round1_public_outputs)
    {
        for (size_t i = 0; i < round1_public_outputs.size(); ++i) {
            auto& [R_user, S_user] = round1_public_outputs[i];
            if (!R_user.on_curve() || R_user.is_point_at_infinity()) {
                info("Round 1 commitments contains invalid R at index ", i);
                return false;
            }
            if (!S_user.on_curve() || S_user.is_point_at_infinity()) {
                info("Round 1 commitments contains invalid S at index ", i);
                return false;
            }
        }
        if (auto duplicated = duplicated_indices(round1_public_outputs); duplicated.size() > 0) {
            info("Round 1 commitments contains duplicate values at indices ", duplicated);
            return false;
        }
        return true;
    }

    /**
     * @brief Generates the Fiat-Shamir challenge `a` that is used to create a Schnorr signature nonce group element
     * [R], where [R] is a uniformly randomly distributed combination of the signer nonces
     *
     * N.B. `a` is message and signer dependent and cannot be pre-generated prior to knowing the message being
     * signed over
     *
     * @warning the resulting 'a' suffers from a slight bias as we apply %r on the 256 bit hash output.
     *
     * @param message
     * @param aggregate_pubkey the output of `combine_signer_pubkeys`
     * @param round_1_nonces the public outputs of round 1 from all signers
     * @return Fr the nonce challenge `a = int(H_non(G, X_agg, "m_start", m.size(), m, "m_end" {(R1, S1), ..., (Rn,
     * Sn)})) % r ` where r is the field order
     */
    static Fr generate_nonce_challenge(const std::string& message,
                                       const affine_element& aggregate_pubkey,
                                       const std::vector<RoundOnePublicOutput>& round_1_nonces)
    {
        // Domain separation for H_non
        const std::string domain_separator_nonce("h_nonce");

        // compute nonce challenge
        // H('domain_separator_nonce', G, X, "m_start", m.size(), m, "m_end", {(R1, S1), ..., (Rn, Sn)})
        std::vector<uint8_t> nonce_challenge_buffer;
        // write domain separator
        std::copy(
            domain_separator_nonce.begin(), domain_separator_nonce.end(), std::back_inserter(nonce_challenge_buffer));

        // write the group generator
        write(nonce_challenge_buffer, G1::affine_one);

        // write X
        write(nonce_challenge_buffer, aggregate_pubkey);

        // we slightly deviate from the protocol when including 'm', since the length of 'm' is variable
        // by writing a prefix and a suffix, we prevent the message from being interpreted as coming from a different
        // session.

        // write "m_start"
        const std::string m_start = "m_start";
        std::copy(m_start.begin(), m_start.end(), std::back_inserter(nonce_challenge_buffer));
        // write m.size()
        write(nonce_challenge_buffer, static_cast<uint32_t>(message.size()));
        // write message
        std::copy(message.begin(), message.end(), std::back_inserter(nonce_challenge_buffer));
        // write "m_end"
        const std::string m_end = "m_end";
        std::copy(m_end.begin(), m_end.end(), std::back_inserter(nonce_challenge_buffer));

        // write  {(R1, S1), ..., (Rn, Sn)}
        for (const auto& nonce : round_1_nonces) {
            write(nonce_challenge_buffer, nonce.R);
            write(nonce_challenge_buffer, nonce.S);
        }

        // uses the different hash function for proper domain separation
        auto nonce_challenge_raw = HashRegNon::hash(nonce_challenge_buffer);
        // this results in a slight bias
        Fr nonce_challenge = Fr::serialize_from_buffer(&nonce_challenge_raw[0]);
        return nonce_challenge;
    }

    /**
     * @brief Compute the Schnorr signature scheme's nonce group element [R], given each signer's public nonces
     * [R_user], [S_user] and the nonce challenge `a`
     *
     * @param a the nonce challenge
     * @param round_1_nonces the public outputs of round 1 from all signers
     * @return affine_element Schnorr nonce [R]
     */
    static affine_element construct_multisig_nonce(const Fr& a, const std::vector<RoundOnePublicOutput>& round_1_nonces)
    {
        element R_sum = round_1_nonces[0].R;
        element S_sum = round_1_nonces[0].S;
        for (size_t i = 1; i < round_1_nonces.size(); ++i) {
            const auto& [R, S] = round_1_nonces[i];
            R_sum += R;
            S_sum += S;
        }
        affine_element R(R_sum + S_sum * a);
        return R;
    }

    /**
     * @brief Returns a vector of indices of elements in input that are included more than once.
     *
     * @warning The returned list may include an index more than once.
     *
     * @tparam T implements operator<
     * @param input list of elements possibly containing duplicates
     * @return std::vector<size_t> a list of indices of input which are included more than once
     */
    template <typename T> static std::vector<size_t> duplicated_indices(const std::vector<T>& input)
    {
        const size_t num_inputs = input.size();
        // indices = [0,1,..., num_inputs-1]
        std::vector<size_t> indices(num_inputs);
        std::iota(indices.begin(), indices.end(), 0);

        // sort indices according to input.
        // input[indices[i-1]] <= input[indices[i]]
        std::sort(indices.begin(), indices.end(), [&](size_t a, size_t b) { return input[a] < input[b]; });

        // This loop will include multiple copies of the same index if an element appears more than twice.
        std::vector<size_t> duplicates;
        for (size_t i = 1; i < num_inputs; ++i) {
            const size_t idx1 = indices[i - 1];
            const size_t idx2 = indices[i];
            if (input[idx1] == input[idx2]) {
                duplicates.push_back(idx1);
                duplicates.push_back(idx2);
            }
        }
        return duplicates;
    }

  public:
    /**
     * @brief Computes the sum of all signer pubkeys. Output is the public key of the public-facing schnorr multisig
     * "signer"
     *
     * @param signer_pubkeys
     *
     * @return std::optional<affine_element> the Schnorr aggregate "signer" public key, if all keys are valid.
     */
    static std::optional<affine_element> validate_and_combine_signer_pubkeys(
        const std::vector<MultiSigPublicKey>& signer_pubkeys)
    {
        std::vector<affine_element> points;
        for (const auto& [public_key, proof_of_possession] : signer_pubkeys) {
            points.push_back(public_key);
        }

        if (auto duplicated = duplicated_indices(points); duplicated.size() > 0) {
            info("Duplicated public keys at indices ", duplicated);
            return std::nullopt;
        }

        element aggregate_pubkey_jac = G1::point_at_infinity;
        for (size_t i = 0; i < signer_pubkeys.size(); ++i) {
            const auto& [public_key, proof_of_possession] = signer_pubkeys[i];
            if (!public_key.on_curve() || public_key.is_point_at_infinity()) {
                info("Multisig signer pubkey not a valid point at index ", i);
                return std::nullopt;
            }
            if (!proof_of_possession.verify(public_key)) {
                info("Multisig proof of posession invalid at index ", i);
                return std::nullopt;
            }
            aggregate_pubkey_jac += public_key;
        }

        // This would prevent accidentally creating an aggregate key for the point at inifinity,
        // with the trivial secret key.
        // While it shouldn't happen, it is a cheap check.
        affine_element aggregate_pubkey(aggregate_pubkey_jac);
        if (aggregate_pubkey.is_point_at_infinity()) {
            info("Multisig aggregate public key is invalid");
            return std::nullopt;
        }
        return aggregate_pubkey;
    }

    /**
     * @brief First round of SpeedyMuSig. Signers generate random nonce keypairs R = {r, [R]}, S = {s, [S]}
     *
     * @param message
     * @return RoundOnePublicOutput group elements [R_user], [S_user]
     * @return RoundOnePrivateOutput field elements [r_user], [s_user]
     *
     */
    static std::pair<RoundOnePublicOutput, RoundOnePrivateOutput> construct_signature_round_1()
    {
        // r_user ← 𝔽
        // TODO: securely erase `r_user`
        Fr r_user = Fr::random_element();
        // R_user ← r_user⋅G
        affine_element R_user = G1::one * r_user;

        // s_user ← 𝔽
        // TODO: securely erase `s_user`
        Fr s_user = Fr::random_element();
        // S_user ← s_user⋅G
        affine_element S_user = G1::one * s_user;

        RoundOnePublicOutput pubOut{ R_user, S_user };
        RoundOnePrivateOutput privOut{ r_user, s_user };
        return { pubOut, privOut };
    }

    /**
     * @brief Second round of SpeedyMuSig. Given the signer pubkeys and the output of round 1,
     * round 2 has each signer compute a share of the Schnorr signature scheme's `s` parameter
     *
     * @param message
     * @param signer
     * @param signer_round_1_private_output the signer's secreet nonce values r, s
     * @param signer_pubkeys
     * @param round_1_nonces the output fro round 1
     * @return std::optional<RoundTwoPublicOutput> signer's share of `s`, if round 2 succeeds
     *
     */
    static std::optional<RoundTwoPublicOutput> construct_signature_round_2(
        const std::string& message,
        const key_pair& signer,
        const RoundOnePrivateOutput& signer_round_1_private_output,
        const std::vector<MultiSigPublicKey>& signer_pubkeys,
        const std::vector<RoundOnePublicOutput>& round_1_nonces)
    {
        const size_t num_signers = signer_pubkeys.size();
        if (round_1_nonces.size() != num_signers) {
            info("Multisig mismatch round_1_nonces and signers");
            return std::nullopt;
        }

        // check that round_1_nonces does not contain duplicates and that all points are valid
        if (!valid_round1_nonces(round_1_nonces)) {
            return std::nullopt;
        }

        // compute aggregate key X = X_1 + ... + X_n
        auto aggregate_pubkey = validate_and_combine_signer_pubkeys(signer_pubkeys);
        if (!aggregate_pubkey.has_value()) {
            // previous call has failed
            return std::nullopt;
        }

        // compute nonce challenge H_non(G, X, "m_start", m, "m_end", {(R1, S1), ..., (Rn, Sn)})
        Fr a = generate_nonce_challenge(message, *aggregate_pubkey, round_1_nonces);

        // compute aggregate nonce R = R1 + ... + Rn + S1 * a + ... + Sn * a
        affine_element R = construct_multisig_nonce(a, round_1_nonces);

        // Now we have the multisig nonce, compute schnorr challenge e (termed `c` in the speedyMuSig paper)
        auto e_buf = generate_schnorr_challenge<HashSig, G1>(message, *aggregate_pubkey, R);
        Fr e = Fr::serialize_from_buffer(&e_buf[0]);

        // output of round 2 is z
        Fr z = signer_round_1_private_output.r + signer_round_1_private_output.s * a - signer.private_key * e;
        return z;
    }

    /**
     * @brief the final step in the SpeedyMuSig multisig scheme. Can be computed by an untrusted 3rd party.
     * Combines the message, signer pubkeys and round1 outputs to compute the Schnorr signature parameter `e`.
     * Combines the outputs of round 2 to compose the total Schnorr signature parameter `s`
     *
     * @param message
     * @param signer_pubkeys
     * @param round_1_nonces The outputs of round 1
     * @param round_2_signature_shares The outputs of round 2
     * @return signature it's a Schnorr signature! Looks identical to a regular non-multisig Schnorr signature.
     * @return std::nullopt if any of the signature shares are invalid
     */
    static std::optional<signature> combine_signatures(
        const std::string& message,
        const std::vector<MultiSigPublicKey>& signer_pubkeys,
        const std::vector<RoundOnePublicOutput>& round_1_nonces,
        const std::vector<RoundTwoPublicOutput>& round_2_signature_shares)
    {
        const size_t num_signers = signer_pubkeys.size();
        if (round_1_nonces.size() != num_signers) {
            info("Invalid number of round1 messages");
            return std::nullopt;
        }
        if (round_2_signature_shares.size() != num_signers) {
            info("Invalid number of round2 messages");
            return std::nullopt;
        }
        if (!valid_round1_nonces(round_1_nonces)) {
            return std::nullopt;
        }

        // compute aggregate key X = X_1 + ... + X_n
        auto aggregate_pubkey = validate_and_combine_signer_pubkeys(signer_pubkeys);
        if (!aggregate_pubkey.has_value()) {
            // previous call has failed
            return std::nullopt;
        }

        // compute nonce challenge H(X, m, {(R1, S1), ..., (Rn, Sn)})
        Fr a = generate_nonce_challenge(message, *aggregate_pubkey, round_1_nonces);

        // compute aggregate nonce R = R1 + ... + Rn + S1 * a + ... + Sn * a
        affine_element R = construct_multisig_nonce(a, round_1_nonces);

        auto e_buf = generate_schnorr_challenge<HashSig, G1>(message, *aggregate_pubkey, R);

        signature sig;
        // copy e as its raw bit representation (without modular reduction)
        std::copy(e_buf.begin(), e_buf.end(), sig.e.begin());

        Fr s = 0;
        for (auto& z : round_2_signature_shares) {
            s += z;
        }
        // write s, which will always produce an integer < r
        Fr::serialize_to_buffer(s, &sig.s[0]);

        // verify the final signature before returning
        if (!verify_signature<HashSig, Fq, Fr, G1>(message, *aggregate_pubkey, sig)) {
            return std::nullopt;
        }

        return sig;
    }
};

template <typename B>
inline void read(B& it, multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>::RoundOnePublicOutput& tx)
{
    read(it, tx.R);
    read(it, tx.S);
}

template <typename B>
inline void write(B& buf, multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>::RoundOnePublicOutput const& tx)
{
    write(buf, tx.R);
    write(buf, tx.S);
}

template <typename B>
inline void read(B& it, multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>::RoundOnePrivateOutput& tx)
{
    read(it, tx.r);
    read(it, tx.s);
}

template <typename B>
inline void write(B& buf, multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>::RoundOnePrivateOutput const& tx)
{
    write(buf, tx.r);
    write(buf, tx.s);
}

template <typename B>
inline void read(B& it, multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>::MultiSigPublicKey& tx)
{
    read(it, tx.public_key);
    read(it, tx.proof_of_possession);
}

template <typename B>
inline void write(B& buf, multisig<grumpkin::g1, KeccakHasher, Blake2sHasher>::MultiSigPublicKey const& tx)
{
    write(buf, tx.public_key);
    write(buf, tx.proof_of_possession);
}
} // namespace crypto::schnorr