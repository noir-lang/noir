
#pragma once
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/crypto/hashers/hashers.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp"
#include "barretenberg/stdlib/encryption/ecdsa/ecdsa_impl.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/curves/secp256k1.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"

template <typename Builder> class EcdsaCircuit {
  public:
    using field_ct = stdlib::field_t<Builder>;
    using bool_ct = stdlib::bool_t<Builder>;
    using public_witness_ct = stdlib::public_witness_t<Builder>;
    using byte_array_ct = stdlib::byte_array<Builder>;
    using curve = stdlib::secp256k1<Builder>;

    static constexpr size_t NUM_PUBLIC_INPUTS = 6;

    static Builder generate(uint256_t public_inputs[])
    {
        Builder builder;

        // IN CIRCUIT
        // Create an input buffer the same size as our inputs
        typename curve::byte_array_ct input_buffer(&builder, NUM_PUBLIC_INPUTS);
        for (size_t i = 0; i < NUM_PUBLIC_INPUTS; ++i) {
            input_buffer.set_byte(i, public_witness_ct(&builder, public_inputs[i]));
        }

        // This is the message that we would like to confirm
        std::string message_string = "goblin";
        auto message = typename curve::byte_array_ct(&builder, message_string);

        // Assert that the public inputs buffer matches the message we want
        for (size_t i = 0; i < NUM_PUBLIC_INPUTS; ++i) {
            input_buffer[i].assert_equal(message[i]);
        }

        // UNCONSTRAINED: create a random keypair to sign with
        crypto::ecdsa_key_pair<typename curve::fr, typename curve::g1> account;
        account.private_key = curve::fr::random_element();
        account.public_key = curve::g1::one * account.private_key;

        // UNCONSTRAINED: create a sig
        crypto::ecdsa_signature signature =
            crypto::ecdsa_construct_signature<Sha256Hasher, typename curve::fq, typename curve::fr, typename curve::g1>(
                message_string, account);

        // UNCONSTRAINED: verify the created signature
        bool dry_run =
            crypto::ecdsa_verify_signature<Sha256Hasher, typename curve::fq, typename curve::fr, typename curve::g1>(
                message_string, account.public_key, signature);
        if (!dry_run) {
            throw_or_abort("[non circuit]: Sig verification failed");
        }

        // IN CIRCUIT: create a witness with the pub key in our circuit
        typename curve::g1_bigfr_ct public_key = curve::g1_bigfr_ct::from_witness(&builder, account.public_key);

        std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
        std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
        uint8_t vv = signature.v;

        // IN CIRCUIT: create a witness with the sig in our circuit
        stdlib::ecdsa_signature<Builder> sig{ typename curve::byte_array_ct(&builder, rr),
                                              typename curve::byte_array_ct(&builder, ss),
                                              stdlib::uint8<Builder>(&builder, vv) };

        // IN CIRCUIT: verify the signature
        typename curve::bool_ct signature_result = stdlib::ecdsa_verify_signature<Builder,
                                                                                  curve,
                                                                                  typename curve::fq_ct,
                                                                                  typename curve::bigfr_ct,
                                                                                  typename curve::g1_bigfr_ct>(
            // input_buffer, public_key, sig);
            input_buffer,
            public_key,
            sig);

        // Assert the signature is true, we hash the message inside the verify sig stdlib call
        bool_ct is_true = bool_ct(1);
        signature_result.must_imply(is_true, "signature verification failed");

        return builder;
    }
};