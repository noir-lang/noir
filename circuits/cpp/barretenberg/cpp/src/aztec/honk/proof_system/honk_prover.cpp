#include "honk_prover.hpp"

namespace honk {

/**
 * Create HonkProver from proving key, witness and manifest.
 *
 * @param input_key Proving key.
 * @param input_manifest Input manifest
 *
 * @tparam settings Settings class.
 * */
template <typename settings>
HonkProver<settings>::HonkProver(std::shared_ptr<waffle::proving_key> input_key,
                                 const transcript::Manifest& input_manifest)
    : n(input_key == nullptr ? 0 : input_key->n)
    , transcript(input_manifest, settings::hash_type, settings::num_challenge_bytes)
    , key(input_key)
    , queue(key.get(), &transcript)
{}

/**
 * For Plonk systems:
 * - Compute commitments to wires 1,2,3
 * - Get public inputs (which were stored in w_2_lagrange) and add to transcript
 *
 * For Honk, we should
 * - Commit to wires 1,2,3
 * - Add PI to transcript (I guess PI will stay in w_2 for now?)
 *
 * */
template <typename settings> void HonkProver<settings>::compute_wire_commitments()
{
    // TODO(luke): Compute wire commitments
    // for (size_t i = 0; i < settings::program_width; ++i) {
    //     std::string wire_tag = "w_" + std::to_string(i + 1);
    //     std::string commit_tag = "W_" + std::to_string(i + 1);
    //     barretenberg::fr* coefficients = key->polynomial_cache.get(wire_tag).get_coefficients();
    //     // This automatically saves the computed point to the transcript
    //     commitment_scheme->commit(coefficients, commit_tag, work_queue::MSMSize::N, queue);
    // }
}

/**
 * For Plonk systems:
 * - added some initial data to transcript: circuit size and PI size
 * - added randomness to lagrange wires
 * - performed ifft to get monomial wires
 *
 * For Honk:
 * - Add circuit size and PI size to transcript. That's it?
 *
 * */
template <typename settings> void HonkProver<settings>::execute_preamble_round()
{
    // Add some initial data to transcript (circuit size and PI size)
    queue.flush_queue();

    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(n >> 24),
                             static_cast<uint8_t>(n >> 16),
                             static_cast<uint8_t>(n >> 8),
                             static_cast<uint8_t>(n) });

    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(key->num_public_inputs >> 24),
                             static_cast<uint8_t>(key->num_public_inputs >> 16),
                             static_cast<uint8_t>(key->num_public_inputs >> 8),
                             static_cast<uint8_t>(key->num_public_inputs) });

    transcript.apply_fiat_shamir("init");
}

/**
 * For Plonk systems:
 * - compute wire commitments
 * - add public inputs to transcript (done in compute_wire_commitments() for some reason)
 *
 * For Honk:
 * - compute wire commitments
 * - add public inputs to transcript (done explicitly in execute_first_round())
 * */
template <typename settings> void HonkProver<settings>::execute_first_round()
{
    queue.flush_queue();

    // TODO(luke): compute_wire_polynomial_commitments()

    // Add public inputs to transcript
    const barretenberg::polynomial& public_wires_source = key->polynomial_cache.get("w_2_lagrange");
    std::vector<barretenberg::fr> public_wires;
    for (size_t i = 0; i < key->num_public_inputs; ++i) {
        public_wires.push_back(public_wires_source[i]);
    }
    transcript.add_element("public_inputs", ::to_buffer(public_wires));
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "eta" challenge (done regardless of arithmetization but only required for Ultra)
 * - does stuff related only to lookups (compute 's' etc and do some RAM/ROM stuff with w_4).
 *
 * For Standard Honk, this is a non-op (just like for Standard/Turbo Plonk).
 * */
template <typename settings> void HonkProver<settings>::execute_second_round()
{
    queue.flush_queue();
    // No operations are needed here for Standard Honk
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "beta" challenge
 * - Compute grand product polynomials (permutation and lookup) and commitments
 * - Compute wire polynomial coset FFTs
 *
 * For Honk:
 * - Do Fiat-Shamir to get "beta" challenge (Note: gamma = beta^2)
 * - Compute grand product polynomial (permutation only) and commitment
 * */
template <typename settings> void HonkProver<settings>::execute_third_round()
{
    queue.flush_queue();

    // Compute beta/gamma challenge (Note: gamma = beta^2)
    transcript.apply_fiat_shamir("beta");

    // TODO(luke): compute_grand_product_polynomial
    // TODO(luke): compute_grand_product_polynomial_commitment
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "alpha" challenge
 * - Compute coset_fft(L_1)
 * - Compute quotient polynomial (with blinding)
 * - Compute quotient polynomial commitment
 *
 * For Honk
 * - Do Fiat-Shamir to get "alpha" challenge
 * - Run Sumcheck
 * */
template <typename settings> void HonkProver<settings>::execute_fourth_round()
{
    queue.flush_queue();

    // Compute alpha challenge
    transcript.apply_fiat_shamir("alpha");

    // TODO(luke): Run Sumcheck
}

/**
 * For Plonk systems (no linearization):
 * - Do Fiat-Shamir to get "z" challenge
 * - Compute evaluation of quotient polynomial and add it to transcript
 *
 * For Honk:
 * - I don't think there's anything to do here. The analog should all occur in Sumcheck
 * - Maybe some pre-processing for Gemini?
 *
 * */
template <typename settings> void HonkProver<settings>::execute_fifth_round()
{
    // TODO(luke): Is there anything to do here? Possible some pre-processing for Gemini?
}

/**
 * For Plonk systems (no linearization):
 * - Do Fiat-Shamir to get "nu" challenge
 * - Perform batch opening
 *
 * For Honk:
 * - engage in Gemini?
 *
 * */
template <typename settings> void HonkProver<settings>::execute_sixth_round()
{
    // TODO(luke): Gemini
}

template <typename settings> waffle::plonk_proof& HonkProver<settings>::export_proof()
{
    proof.proof_data = transcript.export_transcript();
    return proof;
}

template <typename settings> waffle::plonk_proof& HonkProver<settings>::construct_proof()
{
    // Add circuit size and public input size to transcript.
    execute_preamble_round();
    queue.process_queue();

    // Compute wire commitments; Add PI to transcript
    execute_first_round();
    queue.process_queue();

    // This is currently a non-op (for Standard Honk)
    execute_second_round();
    queue.process_queue();

    // Compute challenges beta & gamma; Compute permutation grand product polynomial and its commitment
    execute_third_round();
    queue.process_queue();

    // Compute challenge alpha; Run Sumcheck protocol
    execute_fourth_round();
    queue.process_queue();

    // TBD: possibly some pre-processing for Gemini
    execute_fifth_round();

    // Execute Gemini
    execute_sixth_round();
    queue.process_queue();

    queue.flush_queue();

    return export_proof();
}

// TODO(luke): Need to define a 'standard_settings' analog for Standard Honk
template class HonkProver<waffle::standard_settings>;

} // namespace honk
