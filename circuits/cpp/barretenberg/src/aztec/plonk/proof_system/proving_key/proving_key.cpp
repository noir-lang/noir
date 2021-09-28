#include "proving_key.hpp"
#include <polynomials/polynomial_arithmetic.hpp>

namespace waffle {

// In all the constructors below, the pippenger_runtime_state takes (n + 1) as the input
// as the degree of t_{high}(X) is (n + 1) for standard plonk. Refer to
// ./src/aztec/plonk/proof_system/prover/prover.cpp/ProverBase::compute_quotient_pre_commitment
// for more details on this.
//
// NOTE: If the number of roots cut out of the vanishing polynomial is increased beyond 4,
// the degree of t_{mid}, etc could also increase. Thus, the size of pippenger multi-scalar
// multiplications must be changed accordingly!
//
/**
 * proving_key constructor.
 *
 * Delegates to proving_key::init
 * */
proving_key::proving_key(const size_t num_gates,
                         const size_t num_inputs,
                         std::shared_ptr<ProverReferenceString> const& crs)
    : n(num_gates)
    , num_public_inputs(num_inputs)
    , small_domain(n, n)
    , mid_domain(2 * n, n > min_thread_block ? n : 2 * n)
    , large_domain(4 * n, n > min_thread_block ? n : 4 * n)
    , reference_string(crs)
    , pippenger_runtime_state(n + 1)
{
    init();
}

proving_key::proving_key(proving_key_data&& data, std::shared_ptr<ProverReferenceString> const& crs)
    : n(data.n)
    , num_public_inputs(data.num_public_inputs)
    , constraint_selectors(std::move(data.constraint_selectors))
    , constraint_selector_ffts(std::move(data.constraint_selector_ffts))
    , permutation_selectors(std::move(data.permutation_selectors))
    , permutation_selectors_lagrange_base(std::move(data.permutation_selectors_lagrange_base))
    , permutation_selector_ffts(std::move(data.permutation_selector_ffts))
    , small_domain(n, n)
    , mid_domain(2 * n, n > min_thread_block ? n : 2 * n)
    , large_domain(4 * n, n > min_thread_block ? n : 4 * n)
    , reference_string(crs)
    , pippenger_runtime_state(n + 1)
    , contains_recursive_proof(data.contains_recursive_proof)
    , recursive_proof_public_input_indices(std::move(data.recursive_proof_public_input_indices))
{
    init();
    // TODO: Currently only supporting TurboComposer in serialization!
    std::copy(turbo_polynomial_manifest, turbo_polynomial_manifest + 20, std::back_inserter(polynomial_manifest));
}
/**
 * Initialize proving key.
 *
 * 1. Compute lookup tables for small, mid and large domains
 * 2. Reset wire_ffts and opening_poly
 * 3. Construct L_1
 * 4. Initialize shited_opening_poly(n), linear_poly(n), quotient_mid(2*n), quotient_large(4*n) to zeroes.
 **/
void proving_key::init()
{
    if (n != 0) {
        small_domain.compute_lookup_table();
        mid_domain.compute_lookup_table();
        large_domain.compute_lookup_table();
    }

    reset();

    lagrange_1 = barretenberg::polynomial(4 * n, 4 * n + 8);
    barretenberg::polynomial_arithmetic::compute_lagrange_polynomial_fft(
        lagrange_1.get_coefficients(), small_domain, large_domain);
    lagrange_1.add_lagrange_base_coefficient(lagrange_1[0]);
    lagrange_1.add_lagrange_base_coefficient(lagrange_1[1]);
    lagrange_1.add_lagrange_base_coefficient(lagrange_1[2]);
    lagrange_1.add_lagrange_base_coefficient(lagrange_1[3]);
    lagrange_1.add_lagrange_base_coefficient(lagrange_1[4]);
    lagrange_1.add_lagrange_base_coefficient(lagrange_1[5]);
    lagrange_1.add_lagrange_base_coefficient(lagrange_1[6]);
    lagrange_1.add_lagrange_base_coefficient(lagrange_1[7]);

    // The opening polynomial W_{\script{z}}(X) in round 5 of prover's algorithm has degree n. However,
    // as explained in (./src/aztec/plonk/proof_system/prover/prover.cpp/ProverBase::compute_quotient_pre_commitment),
    // for standard plonk (program_width = 3) and number of roots cut out of the vanishing polynomial is 4,
    // the degree of the quotient polynomial t(X) is 3n. Thus, the number of coefficients in t_{high} is (n + 1).
    // But our prover algorithm assumes that each of t_{low}, t_{mid}, t_{high} is of degree (n - 1) (i.e. n
    // coefficients in each). 
    // Note that: deg(W_{\script{z}}) = max{ deg(t_{low}), deg(t_{mid}), deg(t_{high}), deg(a),
    // deg(b), ... }
    // => deg(W_{\script{z}}) = n + 1 when program_width is 3!
    // Therefore, when program_width is 3, we need to allow the degree of the opening polynomial to be (n + 1) and NOT
    // n.
    //
    // Transfer all of these to reset
    opening_poly =
        barretenberg::polynomial(n, n); // We already do this in reset(). / Ask Zac or Suyash if we can remove this.
    shifted_opening_poly = barretenberg::polynomial(n, n);
    linear_poly = barretenberg::polynomial(n, n);

    quotient_mid = barretenberg::polynomial(2 * n, 2 * n);
    quotient_large = barretenberg::polynomial(4 * n, 4 * n);

    memset((void*)&opening_poly[0], 0x00, sizeof(barretenberg::fr) * n);
    memset((void*)&shifted_opening_poly[0], 0x00, sizeof(barretenberg::fr) * n);
    memset((void*)&linear_poly[0], 0x00, sizeof(barretenberg::fr) * n);
    memset((void*)&quotient_mid[0], 0x00, sizeof(barretenberg::fr) * 2 * n);
    memset((void*)&quotient_large[0], 0x00, sizeof(barretenberg::fr) * 4 * n);
}

/**
 * Reset proving key
 *
 * Clear wire_ffts and fill it with new zeroed out polynomials of size (4 * n + 4) for each of:
 * (w_1_fft, w_2_fft, w_3_fft, w_4_fft, z_fft). Create opening_poly of size n.
 *
 **/
void proving_key::reset()
{
    wire_ffts.clear();

    opening_poly = barretenberg::polynomial(n, n);
    memset((void*)&opening_poly[0], 0x00, sizeof(barretenberg::fr) * n);

    barretenberg::polynomial w_1_fft = barretenberg::polynomial(4 * n + 4, 4 * n + 4);
    barretenberg::polynomial w_2_fft = barretenberg::polynomial(4 * n + 4, 4 * n + 4);
    barretenberg::polynomial w_3_fft = barretenberg::polynomial(4 * n + 4, 4 * n + 4);
    barretenberg::polynomial w_4_fft = barretenberg::polynomial(4 * n + 4, 4 * n + 4);
    barretenberg::polynomial z_fft = barretenberg::polynomial(4 * n + 4, 4 * n + 4);

    memset((void*)&w_1_fft[0], 0x00, sizeof(barretenberg::fr) * (4 * n + 4));
    memset((void*)&w_2_fft[0], 0x00, sizeof(barretenberg::fr) * (4 * n + 4));
    memset((void*)&w_3_fft[0], 0x00, sizeof(barretenberg::fr) * (4 * n + 4));
    memset((void*)&w_4_fft[0], 0x00, sizeof(barretenberg::fr) * (4 * n + 4));
    memset((void*)&z_fft[0], 0x00, sizeof(barretenberg::fr) * (4 * n + 4));

    wire_ffts.insert({ "w_1_fft", std::move(w_1_fft) });
    wire_ffts.insert({ "w_2_fft", std::move(w_2_fft) });
    wire_ffts.insert({ "w_3_fft", std::move(w_3_fft) });
    wire_ffts.insert({ "w_4_fft", std::move(w_4_fft) });
    wire_ffts.insert({ "z_fft", std::move(z_fft) });
}

proving_key::proving_key(const proving_key& other)
    : n(other.n)
    , num_public_inputs(other.num_public_inputs)
    , constraint_selectors(other.constraint_selectors)
    , constraint_selectors_lagrange_base(other.constraint_selectors_lagrange_base)
    , constraint_selector_ffts(other.constraint_selector_ffts)
    , permutation_selectors(other.permutation_selectors)
    , permutation_selectors_lagrange_base(other.permutation_selectors_lagrange_base)
    , permutation_selector_ffts(other.permutation_selector_ffts)
    , wire_ffts(other.wire_ffts)
    , small_domain(other.small_domain)
    , mid_domain(other.mid_domain)
    , large_domain(other.large_domain)
    , reference_string(other.reference_string)
    , lagrange_1(other.lagrange_1)
    , opening_poly(other.opening_poly)
    , shifted_opening_poly(other.shifted_opening_poly)
    , linear_poly(other.linear_poly)
    , quotient_mid(other.quotient_mid)
    , quotient_large(other.quotient_large)
    , pippenger_runtime_state(n + 1)
    , polynomial_manifest(other.polynomial_manifest)
    , contains_recursive_proof(other.contains_recursive_proof)
    , recursive_proof_public_input_indices(other.recursive_proof_public_input_indices)
{}

proving_key::proving_key(proving_key&& other)
    : n(other.n)
    , num_public_inputs(other.num_public_inputs)
    , constraint_selectors(other.constraint_selectors)
    , constraint_selectors_lagrange_base(other.constraint_selectors_lagrange_base)
    , constraint_selector_ffts(other.constraint_selector_ffts)
    , permutation_selectors(other.permutation_selectors)
    , permutation_selectors_lagrange_base(other.permutation_selectors_lagrange_base)
    , permutation_selector_ffts(other.permutation_selector_ffts)
    , wire_ffts(other.wire_ffts)
    , small_domain(std::move(other.small_domain))
    , mid_domain(std::move(other.mid_domain))
    , large_domain(std::move(other.large_domain))
    , reference_string(std::move(other.reference_string))
    , lagrange_1(std::move(other.lagrange_1))
    , opening_poly(std::move(other.opening_poly))
    , shifted_opening_poly(std::move(other.shifted_opening_poly))
    , linear_poly(std::move(other.linear_poly))
    , pippenger_runtime_state(std::move(other.pippenger_runtime_state))
    , polynomial_manifest(std::move(other.polynomial_manifest))
    , contains_recursive_proof(other.contains_recursive_proof)
    , recursive_proof_public_input_indices(std::move(other.recursive_proof_public_input_indices))
{}

proving_key& proving_key::operator=(proving_key&& other)
{
    n = other.n;
    num_public_inputs = other.num_public_inputs;
    constraint_selectors = std::move(other.constraint_selectors);
    constraint_selectors_lagrange_base = std::move(other.constraint_selectors_lagrange_base);
    constraint_selector_ffts = std::move(other.constraint_selector_ffts);
    permutation_selectors = std::move(other.permutation_selectors);
    permutation_selectors_lagrange_base = std::move(other.permutation_selectors_lagrange_base);
    permutation_selector_ffts = std::move(other.permutation_selector_ffts);
    wire_ffts = std::move(other.wire_ffts);
    small_domain = std::move(other.small_domain);
    mid_domain = std::move(other.mid_domain);
    large_domain = std::move(other.large_domain);
    reference_string = std::move(other.reference_string);
    lagrange_1 = std::move(other.lagrange_1);
    opening_poly = std::move(other.opening_poly);
    shifted_opening_poly = std::move(other.shifted_opening_poly);
    linear_poly = std::move(other.linear_poly);
    pippenger_runtime_state = std::move(other.pippenger_runtime_state);
    polynomial_manifest = std::move(other.polynomial_manifest);
    contains_recursive_proof = other.contains_recursive_proof;
    recursive_proof_public_input_indices = std::move(other.recursive_proof_public_input_indices);

    return *this;
}
} // namespace waffle
