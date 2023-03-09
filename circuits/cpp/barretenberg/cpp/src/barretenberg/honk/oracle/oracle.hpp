#pragma once

#include <cstddef>
#include <array>

namespace honk {

template <typename TranscriptType> struct Oracle {
    size_t consumed{ 0 };
    using Transcript = TranscriptType;

    using Fr = typename TranscriptType::Fr;
    Oracle(Transcript*){};

    /**
     * @brief commit data to the current challenge buffer
     *
     * @tparam T template parameter pack! List of types we're inputting
     * @param args data we want to add to the transcript
     *
     * @details Method is deliberately generic. T can be an array of types or a list of parameters. The only
     condition
     * is that T (or its inner types if T is an array) have valid serialization functions `read/write`
     *
     * e.g. all of these are valid uses of `append_data`
     *
     * ```
     *   Fr a = something_old;
     *   Fr b = something_new();
     *   append_data(a, b);
     *   append_data({a, b});
     *   G1& c = something_borrowed();
     *   std::string d = "something new";
     *   append_data({a, b}, c, d);
     * ```
     *
     */
    template <typename... T> void consume(const T&...) { ++consumed; }

    /**
     * @brief use the current value of `current_round_challenge_inputs` to generate a challenge via the Fiat-Shamir
     * heuristic
     *
     * @return Fr the generated challenge
     */
    Fr generate_challenge() { return Fr(consumed + 2); }
};
// /**
//  * @brief Oracle class wraps a Transcript and exposes an interface to generate plonk/honk challenges
//  *
//  * @tparam TranscriptType the transcript class being prametrised
//  *
//  * @details The purpose of the Oracle is to allow the proof system to generate and retrieve challenges without having
//  to
//  * keep track of a manifest (like in old Plonk)
//  *
//  * Conceptually there are two "categories" of challenges in our proof system:
//  *
//  * 1. Challenges specific to the plonk circuit arithmetisation (alpha, gamma, beta, eta)
//  * 2. Challenges local to a modular component used to parametrise a specific proof system (e.g. commitment scheme
//  * challenges, sumcheck challenges)
//  *
//  * Oracle exposes methods to explicitly create/recover challenges of the former type
//  *
//  * The latter type of challenge uses the Oracle in a more idiomatic manner, where the following assumption is made:
//  *
//  * 1. If a modular component gneerates/uses challenges, those challenges can remain local to the module and will not
//  * leak across the module boundary.
//  *
//  * e.g. challenges generated as part of the IPA commitment scheme should *not* be required elsewhere in the proof
//  system
//  * (e.g. the identity tester module)
//  *
//  * The interface to produce "local" challenges is via:
//  *
//  * consume(...input data)
//  *   followed by
//  * generate_challenge()
//  * generate_challenges<num_challenges>()
//  *
//  * ### HANDLING TRANSCRIPT DATA ###
//  *
//  * Oracle handles transcript data in a deterministic and generic manner.
//  * When challenges are generated, input data is pushed into a uint8_t buffer.
//  *
//  * This buffer can be retrieved via `export_transcript`
//  *
//  * Converting this transcript data to/from MEANINGFUL information is done via PROOF classes.
//  * e.g. see `proof_systems/standard_plonk/standard_proof.hpp`
//  *
//  * Proof classes contain explicit member variables describing the data that goes into/from a transcript.
//  * Serialization functions (read/write) are used to convert the uint8_t buffer produced by Oracle into a Proof
//  object.
//  *
//  * The serialize functions are the point at which we should check that the transcript is correctly produced
//  * e.g. all commitments are valid commitments
//  *
//  * The end result is that we do not have a manifest like in old plonk. The round structure of the proof system is
//  * derived via the order in which data is added into the transcript. This can then be explicitly checked by a
//  * proof-system-specific Proof class.
//  */
// template <typename TranscriptType> class Oracle {
//   public:
//     using Transcript = TranscriptType;
//     using Fr = typename Transcript::Fr;
//     Oracle(Transcript* _transcript)
//         : transcript(_transcript)
//     {}

//     template <typename... T> Fr generate_initialisation_challenge(const T&... args)
//     {
//         consume(args...);
//         initialisation_challenge = generate_challenge();
//         return get_initialisation_challenge();
//     }

//     template <typename... T> std::array<Fr, 2> generate_permutation_challenges(const T&... args)
//     {
//         consume(args...);
//         auto res = generate_challenges<2>();
//         beta = res[0];
//         gamma = res[1];
//         return get_permutation_challenges();
//     }

//     template <typename... T> Fr generate_plookup_challenge(const T&... args)
//     {
//         consume(args...);
//         eta = generate_challenge();
//         return get_plookup_challenge();
//     }

//     template <typename... T> Fr generate_identity_separator_challenge(const T&... args)
//     {
//         consume(args...);
//         alpha = generate_challenge();
//         return get_identity_separator_challenge();
//     }

//     /**
//      * @brief commit data to the current challenge buffer
//      *
//      * @tparam T template parameter pack! List of types we're inputting
//      * @param args data we want to add to the transcript
//      *
//      * @details Method is deliberately generic. T can be an array of types or a list of parameters. The only
//      condition
//      * is that T (or its inner types if T is an array) have valid serialization functions `read/write`
//      *
//      * e.g. all of these are valid uses of `append_data`
//      *
//      * ```
//      *   Fr a = something_old;
//      *   Fr b = something_new();
//      *   append_data(a, b);
//      *   append_data({a, b});
//      *   G1& c = something_borrowed();
//      *   std::string d = "something new";
//      *   append_data({a, b}, c, d);
//      * ```
//      *
//      */
//     template <typename... T> void consume(const T&... args) { (_append_to_buffer(args), ...); }

//     /**
//      * @brief use the current value of `current_round_challenge_inputs` to generate a challenge via the Fiat-Shamir
//      * heuristic
//      *
//      * @return Fr the generated challenge
//      */
//     Fr generate_challenge()
//     {
//         ASSERT(current_round_challenge_inputs.size() > 0);
//         const Fr result = transcript->generate_challenge(current_round_challenge_inputs);
//         current_round_challenge_inputs.clear();
//         return result;
//     }

//     /**
//      * @brief like generate_challenge but multiple challenges can be generated + returned
//      *
//      * @tparam num_challenges
//      * @return std::array<Fr, num_challenges>
//      */
//     template <size_t num_challenges> std::array<Fr, num_challenges> generate_challenges()
//     {
//         ASSERT(current_round_challenge_inputs.size() > 0);
//         const auto result = transcript->template generate_challenges<num_challenges>(current_round_challenge_inputs);
//         current_round_challenge_inputs.clear();
//         return result;
//     }

//     /**
//      * @brief Set the opening points object
//      *
//      * Opening points are one area where the Oracle abstraction is a bit leaky (TODO: find a fix??)
//      *
//      * When the IdentityTester::evaluate_identity is called, the number of challenges (and rounds) will be
//      * entirely defined by IdentityTester and is not fixed by the higher-level circuit arithmetisation.
//      * (e.g. quotient testing needs 1 challenge, identity testing needs logn challenges)
//      *
//      * However the challenges generated are *not* local to the IdentityTester module, because they represent the
//      points
//      * that we need to open our polynomials at!
//      *
//      * The current pattern is as follows:
//      *
//      * 1. The IdentityTester::evaluate_identity uses the generic `generate_challenge`/`generate_challenges` methods
//      to
//      * produce challenges
//      * 2. IdentityTester::evaluate_identity *returns* the list of opening points
//      * 3. Prover/Verifier class will then call `oracle.set_opening_points` on the return data.
//      *
//      * @param _opening_points
//      */
//     void set_opening_points(const std::vector<Fr>& _opening_points) { opening_points = _opening_points; }

//     std::vector<uint8_t> export_transcript() { return transcript->export_transcript(); }

//     Fr get_initialisation_challenge() const { return initialisation_challenge; }
//     Fr get_plookup_challenge() const { return eta; }
//     std::array<Fr, 2> get_permutation_challenges() const { return { beta, gamma }; }
//     Fr get_identity_separator_challenge() const { return alpha; }
//     std::vector<Fr> get_opening_points() const { return opening_points; }
//     Fr get_beta() const { return beta; }
//     Fr get_gamma() const { return gamma; }

//     virtual consteval Fr get_identity_permutation_coset_generator() const { return Fr::coset_generator(0); }

//   private:
//     template <typename T> void _append_to_buffer(const T& value)
//     {
//         using serialize::write;
//         // when writing into the buffer that we use to generate the fiat-shamir challenge,
//         // we first want to flatten our data structures.
//         // e.g. if T is a vector of size 2, calling write(buf, val) will append the vector size "2" into
//         // the buffer. We don't want this as it increases the size of the fiat-shamir hash with redundant
//         information.
//         // TODO ADD!
//         // if constexpr (T_is_nested_container)
//         // {
//         //     auto flattened = std::ranges::views::join(value);
//         //     _append_to_buffer(buf, flattened);
//         // }
//         write(current_round_challenge_inputs, value);
//     }
//     template <concepts::RangeCompatibleContainer T> void _append_to_buffer(const T& value)
//     {
//         using serialize::write;
//         for (const auto& param : value) {
//             write(current_round_challenge_inputs, param);
//         }
//     }

//     // TODO make private but allow tests to fiddle with values
//   public:
//     Transcript* transcript;

//     Fr initialisation_challenge = 0;
//     Fr eta = 0;
//     Fr beta = 0;
//     Fr gamma = 0;
//     Fr alpha = 0;
//     Fr nu = 0;
//     Fr u = 0;
//     std::vector<Fr> opening_points = {};

//     std::vector<Fr> public_inputs = {};

//     std::vector<uint8_t> current_round_challenge_inputs = {};

//     Fr public_input_delta = 1; // TODO FIX
// };
} // namespace honk