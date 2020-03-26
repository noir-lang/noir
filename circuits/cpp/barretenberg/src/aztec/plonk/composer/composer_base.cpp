#include "composer_base.hpp"
#include <plonk/proof_system/proving_key/proving_key.hpp>
#include <plonk/proof_system/utils/permutation.hpp>
#include <plonk/proof_system/verification_key/verification_key.hpp>

namespace waffle {
void ComposerBase::assert_equal(const uint32_t a_idx, const uint32_t b_idx)
{
    ASSERT((variables[a_idx] == variables[b_idx]));
    if ((variables[a_idx]) != (variables[b_idx])) {
        std::cout << "hey! variables not equal!" << std::endl;
    }
    for (size_t i = 0; i < wire_epicycles[b_idx].size(); ++i) {
        wire_epicycles[a_idx].emplace_back(wire_epicycles[b_idx][i]);
        if (wire_epicycles[b_idx][i].wire_type == WireType::LEFT) {
            w_l[wire_epicycles[b_idx][i].gate_index] = a_idx;
        } else if (wire_epicycles[b_idx][i].wire_type == WireType::RIGHT) {
            w_r[wire_epicycles[b_idx][i].gate_index] = a_idx;
        } else if (wire_epicycles[b_idx][i].wire_type == WireType::OUTPUT) {
            w_o[wire_epicycles[b_idx][i].gate_index] = a_idx;
        } else if (wire_epicycles[b_idx][i].wire_type == WireType::FOURTH) {
            w_4[wire_epicycles[b_idx][i].gate_index] = a_idx;
        }
    }
    wire_epicycles[b_idx] = std::vector<epicycle>();
}

template <size_t program_width> void ComposerBase::compute_sigma_permutations(proving_key* key)
{
    std::array<std::vector<uint32_t>, program_width> sigma_mappings;
    std::array<uint32_t, 4> wire_offsets{ 0U, 0x40000000, 0x80000000, 0xc0000000 };
    const uint32_t num_public_inputs = static_cast<uint32_t>(public_inputs.size());

    for (size_t i = 0; i < program_width; ++i) {
        sigma_mappings[i].reserve(key->n);
    }
    for (size_t i = 0; i < program_width; ++i) {
        for (size_t j = 0; j < key->n; ++j) {
            sigma_mappings[i].emplace_back(j + wire_offsets[i]);
        }
    }

    for (size_t i = 0; i < wire_epicycles.size(); ++i) {
        for (size_t j = 0; j < wire_epicycles[i].size(); ++j) {
            epicycle current_epicycle = wire_epicycles[i][j];
            size_t epicycle_index = j == wire_epicycles[i].size() - 1 ? 0 : j + 1;
            epicycle next_epicycle = wire_epicycles[i][epicycle_index];
            sigma_mappings[static_cast<uint32_t>(current_epicycle.wire_type) >>
                           30U][current_epicycle.gate_index + num_public_inputs] =
                next_epicycle.gate_index + static_cast<uint32_t>(next_epicycle.wire_type) + num_public_inputs;
        }
    }

    for (size_t i = 0; i < program_width; ++i) {
        std::string index = std::to_string(i + 1);
        barretenberg::polynomial sigma_polynomial(key->n);
        compute_permutation_lagrange_base_single<standard_settings>(
            sigma_polynomial, sigma_mappings[i], key->small_domain);

        if (i == 0) {
            barretenberg::fr work_root = barretenberg::fr::one();
            for (size_t j = 0; j < num_public_inputs; ++j) {
                sigma_polynomial[j] = work_root;
                work_root *= key->small_domain.root;
            }
        }

        barretenberg::polynomial sigma_polynomial_lagrange_base(sigma_polynomial);
        key->permutation_selectors_lagrange_base.insert(
            { "sigma_" + index, std::move(sigma_polynomial_lagrange_base) });
        sigma_polynomial.ifft(key->small_domain);
        barretenberg::polynomial sigma_fft(sigma_polynomial, key->large_domain.size);
        sigma_fft.coset_fft(key->large_domain);
        key->permutation_selectors.insert({ "sigma_" + index, std::move(sigma_polynomial) });
        key->permutation_selector_ffts.insert({ "sigma_" + index + "_fft", std::move(sigma_fft) });
    }
}

template void ComposerBase::compute_sigma_permutations<3>(proving_key* key);
template void ComposerBase::compute_sigma_permutations<4>(proving_key* key);

} // namespace waffle