#include "proving_key.hpp"
#include <polynomials/polynomial_arithmetic.hpp>

namespace waffle {
proving_key::proving_key(const size_t num_gates,
                         const size_t num_inputs,
                         std::shared_ptr<ProverReferenceString> const& crs)
    : n(num_gates)
    , num_public_inputs(num_inputs)
    , small_domain(n, n)
    , mid_domain(2 * n, n > min_thread_block ? n : 2 * n)
    , large_domain(4 * n, n > min_thread_block ? n : 4 * n)
    , reference_string(crs)
    , pippenger_runtime_state(n)
{
    if (n != 0) {
        small_domain.compute_lookup_table();
        mid_domain.compute_lookup_table();
        large_domain.compute_lookup_table();
    }

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
    // memset((void*)&z[0], 0x00, sizeof(barretenberg::fr) * n);

    wire_ffts.insert({ "w_1_fft", std::move(w_1_fft) });
    wire_ffts.insert({ "w_2_fft", std::move(w_2_fft) });
    wire_ffts.insert({ "w_3_fft", std::move(w_3_fft) });
    wire_ffts.insert({ "w_4_fft", std::move(w_4_fft) });
    wire_ffts.insert({ "z_fft", std::move(z_fft) });

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

    opening_poly = barretenberg::polynomial(n, n);
    shifted_opening_poly = barretenberg::polynomial(n, n);
    linear_poly = barretenberg::polynomial(n, n);

    quotient_mid = barretenberg::polynomial(2 * n, 2 * n);
    quotient_large = barretenberg::polynomial(4 * n, 4 * n);

    memset((void*)&opening_poly[0], 0x00, sizeof(barretenberg::fr) * n);
    memset((void*)&shifted_opening_poly[0], 0x00, sizeof(barretenberg::fr) * n);
    memset((void*)&linear_poly[0], 0x00, sizeof(barretenberg::fr) * n);
    memset((void*)&quotient_mid[0], 0x00, sizeof(barretenberg::fr) * 2 * n);
    memset((void*)&quotient_large[0], 0x00, sizeof(barretenberg::fr) * 4 * n);

    // size_t memory = opening_poly.get_max_size() * 32;
    // memory += (linear_poly.get_max_size() * 32);
    // memory += (shifted_opening_poly.get_max_size() * 32);
    // memory += (opening_poly.get_max_size() * 32);
    // memory += (lagrange_1.get_max_size() * 32);
    // memory += (w_1_fft.get_max_size() * 32);
    // memory += (w_2_fft.get_max_size() * 32);
    // memory += (w_3_fft.get_max_size() * 32);
    // memory += (w_4_fft.get_max_size() * 32);
    // memory += (z_fft.get_max_size() * 32);
    // memory += (z.get_max_size() * 32);
    // memory += (small_domain.size * 2 * 32);
    // memory += (mid_domain.size * 2 * 32);
    // memory += (large_domain.size * 2 * 32);
    // memory += (quotient_mid.get_max_size() * 32);
    // memory += (quotient_large.get_max_size() * 32);

    // printf("proving key allocated memory = %lu \n", memory / (1024UL * 1024UL));
}

void proving_key::reset()
{
    wire_ffts.clear();

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
    , pippenger_runtime_state(n)
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
    return *this;
}
} // namespace waffle