#pragma once

#include <utility>

#include "./schnorr.hpp"
namespace crypto {
namespace schnorr {

/**
 * @brief Implements the SpeedyMuSig protocol; a secure 2-round interactive multisignature scheme
 * whose signature outputs can be verified by a regular Schnorr verification algorithm.
 *
 * @tparam G1 The elliptic curve group being used to generate the multisignature
 * @tparam Hash The hash function being used. Must be able to be modelled as a random oracle! (i.e. not pedersen)
 *
 * @details SpeedyMuSig paper at https://eprint.iacr.org/2021/1375.pdf
 */
template <typename G1, typename Hash> class multisig {
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
     * This is to prevent attacks where an attacker presents a key that they do the secret to
     * (e.g. the attacker public key is a linear combination of other public keys)
     *
     */
    struct MultiSigPublicKey {
        affine_element public_key;
        signature proof_of_possession;
    };

    struct RoundOnePrivateOutput {
        Fr r;
        Fr s;
    };

    struct RoundOnePublicOutput {
        affine_element R;
        affine_element S;

        // for std::sort
        bool operator<(const RoundOnePublicOutput& other) const
        {
            if (R < other.R) {
                return true;
            } else if (R == other.R && S < other.S) {
                return true;
            }
            return false;
        }

        bool operator==(const RoundOnePublicOutput& other) const { return R == other.R && S == other.S; }
    };
    using RoundTwoPublicOutput = Fr;

  private:
    /**
     * @brief Generate nonce keypair
     *
     * @return key_pair
     */
    static key_pair generate_nonce_key_pair()
    {
        key_pair result;
        result.private_key = Fr::random_element();
        result.public_key = G1::one * result.private_key;
        return result;
    }

    /**
     * @brief Generates the Fiat-Shamir challenge `a` that is used to create a Schnorr signature nonce group element
     * [R], where [R] is a uniformly randomly distributed combination of the signer nonces
     *
     * N.B. `a` is message and signer dependent and cannot be pre-generated prior to knowing the message being signed
     * over
     * @param message
     * @param aggregate_pubkey the output of `combine_signer_pubkeys`
     * @param round_1_nonces the public outputs of round 1 from all signers
     * @return Fr the nonce challenge `a`
     */
    static Fr generate_nonce_challenge(const std::string& message,
                                       const affine_element& aggregate_pubkey,
                                       const std::vector<RoundOnePublicOutput>& round_1_nonces)
    {
        // compute nonce challenge H(X, m, {(R1, S1), ..., (Rn, Sn)})
        std::vector<uint8_t> nonce_challenge_buffer;
        write(nonce_challenge_buffer, aggregate_pubkey);
        std::copy(message.begin(), message.end(), std::back_inserter(nonce_challenge_buffer));
        for (const auto& nonce : round_1_nonces) {
            write(nonce_challenge_buffer, nonce.R);
            write(nonce_challenge_buffer, nonce.S);
        }

        auto nonce_challenge_raw = Hash::hash(nonce_challenge_buffer);
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

        element R_sum = G1::point_at_infinity;
        element S_sum = G1::point_at_infinity;
        for (const auto& nonce : round_1_nonces) {
            R_sum += nonce.R;
            S_sum += nonce.S;
        }
        affine_element R(R_sum + S_sum * a);
        return R;
    }

    /**
     * @brief Generate the schnorr signature challenge parameter `e` given a message, signer pubkey and nonce
     *
     * @param message what are we signing over?
     * @param aggregate_pubkey the aggregate pubkey of all multisig signers
     * @param R the aggregate nonce produced from the set of signer nonces
     * @return std::vector<uint8_t> fiat-shamir produced hash buffer
     */
    static std::vector<uint8_t> generate_schnorr_challenge(const std::string& message,
                                                           const affine_element& aggregate_pubkey,
                                                           const affine_element& R)
    {
        // create
        Fq compressed_keys = crypto::pedersen::compress_native({ R.x, aggregate_pubkey.x, aggregate_pubkey.y });
        std::vector<uint8_t> e_buffer;
        write(e_buffer, compressed_keys);
        std::copy(message.begin(), message.end(), std::back_inserter(e_buffer));

        auto e_raw = Hash::hash(e_buffer);

        // some hashes produce e_raw as a std::array. Convert to vector
        std::vector<uint8_t> result;
        std::copy(e_raw.begin(), e_raw.end(), std::back_inserter(result));
        return result;
    }

    template <typename T> static bool contains_duplicates(const std::vector<T>& input)
    {
        std::vector<T> copy(input.begin(), input.end());
        std::sort(copy.begin(), copy.end());
        auto it = std::unique(copy.begin(), copy.end());
        bool wasUnique = (it == copy.end());
        return !wasUnique;
    }

    static bool elements_not_on_curve(const std::vector<affine_element>& input)
    {
        bool good = true;
        for (const auto& e : input) {
            good = good && e.on_curve();
        }
        return !good;
    }

    static signature empty_signature()
    {
        signature sig;
        sig.s.fill(static_cast<uint8_t>(-1));
        sig.e.fill(static_cast<uint8_t>(-1));
        return sig;
    }

  public:
    /**
     * @brief Create a multi sig public key object.
     * Returned object contains a proof of posession signed over the public key
     *
     * @param account the signer's account
     * @return MultiSigPublicKey
     */
    static MultiSigPublicKey create_multi_sig_public_key(const key_pair& account)
    {
        MultiSigPublicKey result;
        result.public_key = account.public_key;

        std::vector<uint8_t> keybuf;
        write(keybuf, account.public_key);
        result.proof_of_possession =
            schnorr::construct_signature<Hash, Fq, Fr, G1>(std::string(keybuf.begin(), keybuf.end()), account);
        return result;
    }
    /**
     * @brief Computes the sum of all signer pubkeys. Output is the public key of the public-facing schnorr multisig
     * "signer"
     *
     * @param signer_pubkeys
     *
     * @warning if the verification fails, then the returned affine_element is the point at infinity. The caller is
     * responsible for checking this.
     *
     * @return affine_element the Schnorr aggregate "signer" public key
     */
    static affine_element validate_and_combine_signer_pubkeys(const std::vector<MultiSigPublicKey>& signer_pubkeys)
    {
        std::vector<affine_element> points;
        element aggregate_pubkey_jac = G1::point_at_infinity;
        for (const auto& [public_key, proof_of_possession] : signer_pubkeys) {
            points.push_back(public_key);
            if (!public_key.on_curve() || public_key.is_point_at_infinity()) {
                std::cerr << "Multisig signer pubkey not a valid point" << std::endl;
                return G1::affine_point_at_infinity;
            }
            std::vector<uint8_t> keybuf;
            write(keybuf, public_key);
            if (!schnorr::verify_signature<Hash, Fq, Fr, G1>(
                    std::string(keybuf.begin(), keybuf.end()), public_key, proof_of_possession)) {
                std::cerr << "Multisig proof of posession invalid" << std::endl;
                return G1::affine_point_at_infinity;
            }
            aggregate_pubkey_jac += public_key;
        }

        if (contains_duplicates(points)) {
            // can't throw an exception here as wasm build requires disabled exceptions
            std::cerr << "Multisig signer pubkeys contains duplicate values" << std::endl;
            return G1::affine_point_at_infinity;
        }
        affine_element aggregate_pubkey(aggregate_pubkey_jac);
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
        auto R = generate_nonce_key_pair();
        auto S = generate_nonce_key_pair();

        RoundOnePublicOutput pubOut{ .R = R.public_key, .S = S.public_key };

        RoundOnePrivateOutput privOut{
            .r = R.private_key,
            .s = S.private_key,
        };
        std::pair<RoundOnePublicOutput, RoundOnePrivateOutput> result;
        result.first = pubOut;
        result.second = privOut;
        return result;
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
     * @return RoundTwoPublicOutput signer's share of `s`
     *
     */
    static RoundTwoPublicOutput construct_signature_round_2(const std::string& message,
                                                            const key_pair& signer,
                                                            const RoundOnePrivateOutput& signer_round_1_private_output,
                                                            const std::vector<MultiSigPublicKey>& signer_pubkeys,
                                                            const std::vector<RoundOnePublicOutput>& round_1_nonces)
    {
        if (contains_duplicates(round_1_nonces)) {
            // can't throw an exception here as wasm build requires disabled exceptions
            std::cerr << "Multisig signer nonces contains duplicate values" << std::endl;
            return -1;
        }

        // compute aggregate key X = X_1 + ... + X_n
        affine_element aggregate_pubkey = validate_and_combine_signer_pubkeys(signer_pubkeys);

        if (aggregate_pubkey.is_point_at_infinity()) {
            // previous call has failed
            return -1;
        }
        // compute nonce challenge H(X, m, {(R1, S1), ..., (Rn, Sn)})
        Fr a = generate_nonce_challenge(message, aggregate_pubkey, round_1_nonces);

        // compute aggregate nonce R = R1 + ... + Rn + S1 * a + ... + Sn * a
        affine_element R = construct_multisig_nonce(a, round_1_nonces);

        // Now we have the multisig nonce, compute schnorr challenge e (termed `c` in the speedyMuSig paper)
        auto e_raw = generate_schnorr_challenge(message, aggregate_pubkey, R);

        Fr e = Fr::serialize_from_buffer(&e_raw[0]);

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
     */
    static signature combine_signatures(const std::string& message,
                                        const std::vector<MultiSigPublicKey>& signer_pubkeys,
                                        const std::vector<RoundOnePublicOutput>& round_1_nonces,
                                        const std::vector<RoundTwoPublicOutput>& round_2_signature_shares)
    {
        if (contains_duplicates(round_1_nonces)) {
            // can't throw an exception here as wasm build requires disabled exceptions
            std::cerr << "Multisig signer nonces contains duplicate values" << std::endl;
            return empty_signature();
        }
        if (contains_duplicates(round_2_signature_shares)) {
            // can't throw an exception here as wasm build requires disabled exceptions
            std::cerr << "Multisig signature shares contains duplicate values" << std::endl;
            return empty_signature();
        }

        signature sig;

        // compute aggregate key X = X_1 + ... + X_n
        affine_element aggregate_pubkey = validate_and_combine_signer_pubkeys(signer_pubkeys);

        if (aggregate_pubkey.is_point_at_infinity()) {
            // previous call has failed
            return empty_signature();
        }

        // compute nonce challenge H(X, m, {(R1, S1), ..., (Rn, Sn)})
        Fr a = generate_nonce_challenge(message, aggregate_pubkey, round_1_nonces);

        // compute aggregate nonce R = R1 + ... + Rn + S1 * a + ... + Sn * a
        affine_element R = construct_multisig_nonce(a, round_1_nonces);

        auto e_raw = generate_schnorr_challenge(message, aggregate_pubkey, R);
        std::copy(e_raw.begin(), e_raw.end(), sig.e.begin());

        Fr s = 0;
        for (auto& z : round_2_signature_shares) {
            s += z;
        }
        Fr::serialize_to_buffer(s, &sig.s[0]);

        return sig;
    }
};

void read(uint8_t const*& it, multisig<grumpkin::g1, Blake2sHasher>::RoundOnePublicOutput& tx)
{
    read(it, tx.R);
    read(it, tx.S);
}

template <typename B> void write(B& buf, multisig<grumpkin::g1, Blake2sHasher>::RoundOnePublicOutput& tx)
{
    write(buf, tx.R);
    write(buf, tx.S);
}

void read(uint8_t const*& it, multisig<grumpkin::g1, Blake2sHasher>::RoundOnePrivateOutput& tx)
{
    read(it, tx.r);
    read(it, tx.s);
}

template <typename B> void write(B& buf, multisig<grumpkin::g1, Blake2sHasher>::RoundOnePrivateOutput& tx)
{
    write(buf, tx.r);
    write(buf, tx.s);
}

void read(uint8_t const*& it, multisig<grumpkin::g1, Blake2sHasher>::MultiSigPublicKey& tx)
{
    read(it, tx.public_key);
    read(it, tx.proof_of_possession.s);
    read(it, tx.proof_of_possession.e);
}

template <typename B> void write(B& buf, multisig<grumpkin::g1, Blake2sHasher>::MultiSigPublicKey& tx)
{
    write(buf, tx.public_key);
    write(buf, tx.proof_of_possession.s);
    write(buf, tx.proof_of_possession.e);
}
} // namespace schnorr
} // namespace crypto