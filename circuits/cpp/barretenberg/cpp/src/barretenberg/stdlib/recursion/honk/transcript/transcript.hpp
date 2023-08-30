#pragma once

#include "barretenberg/ecc/curves/bn254/fq.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/bn254/g1.hpp"
#include "barretenberg/honk/sumcheck/polynomials/univariate.hpp"
#include "barretenberg/honk/transcript/transcript.hpp"

#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/utility/utility.hpp"

// TODO(luke): this namespace will be sensible once stdlib is moved out of the plonk namespace
namespace proof_system::plonk::stdlib::recursion::honk {
template <typename Builder> class Transcript {
  public:
    using field_ct = field_t<Builder>;
    using FF = barretenberg::fr;
    using VerifierTranscript = proof_system::honk::VerifierTranscript<FF>;
    using StdlibTypes = utility::StdlibTypesUtility<Builder>;

    static constexpr size_t HASH_OUTPUT_SIZE = VerifierTranscript::HASH_OUTPUT_SIZE;

    VerifierTranscript native_transcript;
    Builder* builder;

    Transcript() = default;

    Transcript(Builder* builder, auto proof_data)
        : native_transcript(proof_data)
        , builder(builder){};

    /**
     * @brief Get the underlying native transcript manifest (primarily for debugging)
     *
     */
    auto get_manifest() const { return native_transcript.get_manifest(); };

    /**
     * @brief Compute the challenges (more than 1) indicated by labels
     *
     * @tparam Strings
     * @param labels Names of the challenges to be computed
     * @return std::array<FF, sizeof...(Strings)> Array of challenges
     */
    template <typename... Strings> std::array<field_ct, sizeof...(Strings)> get_challenges(const Strings&... labels)
    {
        // Compute the indicated challenges from the native transcript
        constexpr size_t num_challenges = sizeof...(Strings);
        std::array<FF, num_challenges> native_challenges{};
        native_challenges = native_transcript.get_challenges(labels...);

        /*
         * TODO(#1351): Do stdlib hashing here. E.g., for the current pedersen/blake setup, we could write data into a
         * byte_array as it is received from prover, then compress via pedersen and apply blake3s. Not doing this now
         * since it's a pain and we'll be revamping our hashing anyway. For now, simply convert the native hashes to
         * stdlib types without adding any hashing constraints.
         */
        std::array<field_ct, num_challenges> challenges;
        for (size_t i = 0; i < num_challenges; ++i) {
            challenges[i] = native_challenges[i];
        }

        return challenges;
    }

    /**
     * @brief Compute the single challenge indicated by the input label
     *
     * @param label Name of challenge
     * @return field_ct Challenge
     */
    field_ct get_challenge(const std::string& label)
    {
        // Compute the indicated challenge from the native transcript
        auto native_challenge = native_transcript.get_challenge(label);

        // TODO(1351): Stdlib hashing here...

        return field_ct::from_witness(builder, native_challenge);
    }

    /**
     * @brief Extract a native element from the transcript and return a corresponding stdlib type
     *
     * @tparam T Type of the native element to be extracted
     * @param label Name of the element
     * @return The corresponding element of appropriate stdlib type
     */
    template <class T> auto receive_from_prover(const std::string& label)
    {
        // Get native type corresponding to input type
        using NativeType = typename StdlibTypes::template NativeType<T>::type;

        // Extract the native element from the native transcript
        NativeType element = native_transcript.template receive_from_prover<NativeType>(label);

        // Return the corresponding stdlib type
        return StdlibTypes::from_witness(builder, element);
    }
};
} // namespace proof_system::plonk::stdlib::recursion::honk
