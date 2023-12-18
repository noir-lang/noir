#pragma once

#include "barretenberg/common/log.hpp"

namespace instance_inspector {

// Determine whether a polynomial has at least one non-zero coefficient
bool is_non_zero(auto& polynomial)
{
    for (auto& coeff : polynomial) {
        if (!coeff.is_zero()) {
            return true;
        }
    }
    return false;
}

/**
 * @brief Utility for indicating which polynomials in a prover instance are identically zero
 *
 * @param prover_instance
 */
void inspect_instance(auto& prover_instance)
{
    auto& prover_polys = prover_instance->prover_polynomials;
    std::vector<std::string> zero_polys;
    for (auto [label, poly] : zip_view(prover_polys.get_labels(), prover_polys.get_all())) {
        if (!is_non_zero(poly)) {
            zero_polys.emplace_back(label);
        }
    }
    if (zero_polys.empty()) {
        info("\nDebug Utility: All prover polynomials are non-zero.");
    } else {
        info("\nDebug Utility: The following prover polynomials are identically zero: ");
        for (const std::string& label : zero_polys) {
            info("\t", label);
        }
    }
    info();
}

} // namespace instance_inspector