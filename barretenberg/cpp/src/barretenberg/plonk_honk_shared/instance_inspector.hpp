#pragma once

#include "barretenberg/common/log.hpp"

namespace bb::instance_inspector {

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
        info("\nInstance Inspector: All prover polynomials are non-zero.");
    } else {
        info("\nInstance Inspector: The following prover polynomials are identically zero: ");
        for (const std::string& label : zero_polys) {
            info("\t", label);
        }
    }
    info();
}

/**
 * @brief Print some useful info about polys related to the databus lookup relation
 *
 * @param prover_instance
 */
void print_databus_info(auto& prover_instance)
{
    info("\nInstance Inspector: Printing databus gate info.");
    auto& key = prover_instance->proving_key;
    for (size_t idx = 0; idx < prover_instance->proving_key.circuit_size; ++idx) {
        if (key->q_busread[idx] == 1) {
            info("idx = ", idx);
            info("q_busread = ", key->q_busread[idx]);
            info("w_l = ", key->w_l[idx]);
            info("w_r = ", key->w_r[idx]);
        }
        if (key->calldata_read_counts[idx] > 0) {
            info("idx = ", idx);
            info("read_counts = ", key->calldata_read_counts[idx]);
            info("calldata = ", key->calldata[idx]);
            info("databus_id = ", key->databus_id[idx]);
        }
    }
    info();
}

} // namespace bb::instance_inspector