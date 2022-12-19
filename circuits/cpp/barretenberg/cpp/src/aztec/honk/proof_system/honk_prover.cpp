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

    // TODO(luke): add public inputs to trascript for Honk
    // const polynomial& public_wires_source = key->polynomial_cache.get("w_2_lagrange");
    // std::vector<fr> public_wires;
    // for (size_t i = 0; i < key->num_public_inputs; ++i) {
    //     public_wires.push_back(public_wires_source[i]);
    // }
    // transcript.add_element("public_inputs", ::to_buffer(public_wires));
}

/**
 * For Plonk systems:
 * - added some initial data to transcript: circuit size and PI size
 * - added randomness to lagrange wires
 * - performed ifft to get monomial wires
 *
 * For Honk:
 * - Still add initial data to transcript? That's it?
 *
 * */
template <typename settings> void HonkProver<settings>::execute_preamble_round()
{
    // TODO(lde): add some initial data to transcript (circuit size and PI size)
}

/**
 * For Plonk systems this does 1 thing: compute wire commitments.
 * This can probably be the identical thing for Honk.
 * */
template <typename settings> void HonkProver<settings>::execute_first_round()
{
    // TODO(luke): compute_wire_polynomial_commitments()
}

/**
 * For Plonk systems this
 * - Do Fiat-Shamir to get "eta" challenge
 * - does stuff related only to lookups (compute 's' etc and do some RAM/ROM stuff with w_4).
 *
 * For Standard Honk, this is a non-op (jsut like for Standard/Turbo Plonk).
 * */
template <typename settings> void HonkProver<settings>::execute_second_round()
{
    // TODO(luke): no-op (eta is not needed in Standard)
}

/**
 * For Plonk systems:
 * - Do Fiat-Shamir to get "beta" challenge
 * - Compute grand product polynomials (permutation and lookup)
 * - Compute grand product commitments
 *
 * For Honk, we should
 * - Do Fiat-Shamir to get "beta" challenge
 * - Compute grand product polynomials (permutation only)
 * - Compute grand product commitment (permutation only)
 *
 * */
template <typename settings> void HonkProver<settings>::execute_third_round()
{
    // TODO(luke): compute beta/gamma challenge (Note: gamma = beta^2)
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
 *
 * */
template <typename settings> void HonkProver<settings>::execute_fourth_round()
{
    // TODO(luke): Do Fiat-Shamir to get "alpha" challenge
    // TODO(luke): Run Sumcheck
}

/**
 * For Plonk systems (no linearization):
 * - Do Fiat-Shamir to get "z" challenge
 * - Compute evaluation of quotient polynomial
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
 * - Do Fiat-Shamir to get "nu" challenge?
 * - engage in Gemini?
 *
 * */
template <typename settings> void HonkProver<settings>::execute_sixth_round()
{
    // TODO(luke): Do Fiat-Shamir to get "nu" challenge?
    // TODO(luke): Gemini?
}

template <typename settings> waffle::plonk_proof& HonkProver<settings>::export_proof()
{
    proof.proof_data = transcript.export_transcript();
    return proof;
}

template <typename settings> waffle::plonk_proof& HonkProver<settings>::construct_proof()
{
    // Execute init round. Randomize witness polynomials.
    execute_preamble_round();
    queue.process_queue();

    // Compute wire precommitments and sometimes random widget round commitments
    execute_first_round();
    queue.process_queue();

    // Fiat-Shamir eta + execute random widgets.
    execute_second_round();
    queue.process_queue();

    // Fiat-Shamir beta & gamma, execute random widgets (Permutation widget is executed here)
    // and fft the witnesses
    execute_third_round();
    queue.process_queue();

    // Fiat-Shamir alpha, compute & commit to quotient polynomial.
    execute_fourth_round();
    queue.process_queue();

    execute_fifth_round();

    execute_sixth_round();
    queue.process_queue();

    queue.flush_queue();

    return export_proof();
}

// TODO(luke): Need to define a 'standard_settings' analog for Standard Honk
template class HonkProver<waffle::standard_settings>;

} // namespace honk
