#include "prover_library.hpp"
#include "barretenberg/honk/flavor/goblin_ultra.hpp"
#include "barretenberg/honk/flavor/standard.hpp"
#include "barretenberg/honk/flavor/standard_grumpkin.hpp"
#include "barretenberg/honk/flavor/ultra.hpp"
#include "barretenberg/honk/flavor/ultra_grumpkin.hpp"
#include <span>
#include <string>

namespace proof_system::honk::prover_library {

/**
 * @brief Construct sorted list accumulator polynomial 's'.
 *
 * @details Compute s = s_1 + η*s_2 + η²*s_3 + η³*s_4 (via Horner) where s_i are the
 * sorted concatenated witness/table polynomials
 *
 * @param key proving key
 * @param sorted_list_polynomials sorted concatenated witness/table polynomials
 * @param eta random challenge
 * @return Polynomial
 */
template <typename Flavor>
typename Flavor::Polynomial compute_sorted_list_accumulator(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                                            typename Flavor::FF eta)
{
    using FF = typename Flavor::FF;
    using Polynomial = typename Flavor::Polynomial;

    const size_t circuit_size = key->circuit_size;

    auto sorted_list_accumulator = Polynomial{ circuit_size };

    auto sorted_polynomials = key->get_sorted_polynomials();

    // Construct s via Horner, i.e. s = s_1 + η(s_2 + η(s_3 + η*s_4))
    for (size_t i = 0; i < circuit_size; ++i) {
        FF T0 = sorted_polynomials[3][i];
        T0 *= eta;
        T0 += sorted_polynomials[2][i];
        T0 *= eta;
        T0 += sorted_polynomials[1][i];
        T0 *= eta;
        T0 += sorted_polynomials[0][i];
        sorted_list_accumulator[i] = T0;
    }

    return sorted_list_accumulator;
}

/**
 * @brief Add plookup memory records to the fourth wire polynomial
 *
 * @details This operation must be performed after the first three wires have been committed to, hence the dependence on
 * the `eta` challenge.
 *
 * @tparam Flavor
 * @param eta challenge produced after commitment to first three wire polynomials
 */
template <typename Flavor>
void add_plookup_memory_records_to_wire_4(std::shared_ptr<typename Flavor::ProvingKey>& key, typename Flavor::FF eta)
{
    // The plookup memory record values are computed at the indicated indices as
    // w4 = w3 * eta^3 + w2 * eta^2 + w1 * eta + read_write_flag;
    // (See plookup_auxiliary_widget.hpp for details)
    auto wires = key->get_wires();

    // Compute read record values
    for (const auto& gate_idx : key->memory_read_records) {
        wires[3][gate_idx] += wires[2][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += wires[1][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += wires[0][gate_idx];
        wires[3][gate_idx] *= eta;
    }

    // Compute write record values
    for (const auto& gate_idx : key->memory_write_records) {
        wires[3][gate_idx] += wires[2][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += wires[1][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += wires[0][gate_idx];
        wires[3][gate_idx] *= eta;
        wires[3][gate_idx] += 1;
    }
}

// flavor::Ultra

template typename honk::flavor::Ultra::Polynomial compute_sorted_list_accumulator<honk::flavor::Ultra>(
    std::shared_ptr<typename honk::flavor::Ultra::ProvingKey>& key, typename honk::flavor::Ultra::FF eta);

template void add_plookup_memory_records_to_wire_4<honk::flavor::Ultra>(
    std::shared_ptr<typename honk::flavor::Ultra::ProvingKey>& key, typename honk::flavor::Ultra::FF eta);

// flavor::UltraGrumpkin

template typename honk::flavor::UltraGrumpkin::Polynomial compute_sorted_list_accumulator<honk::flavor::UltraGrumpkin>(
    std::shared_ptr<typename honk::flavor::UltraGrumpkin::ProvingKey>& key,
    typename honk::flavor::UltraGrumpkin::FF eta);

template void add_plookup_memory_records_to_wire_4<honk::flavor::UltraGrumpkin>(
    std::shared_ptr<typename honk::flavor::UltraGrumpkin::ProvingKey>& key,
    typename honk::flavor::UltraGrumpkin::FF eta);

// flavor::GoblinUltra

template typename honk::flavor::GoblinUltra::Polynomial compute_sorted_list_accumulator<honk::flavor::GoblinUltra>(
    std::shared_ptr<typename honk::flavor::GoblinUltra::ProvingKey>& key, typename honk::flavor::GoblinUltra::FF eta);

template void add_plookup_memory_records_to_wire_4<honk::flavor::GoblinUltra>(
    std::shared_ptr<typename honk::flavor::GoblinUltra::ProvingKey>& key, typename honk::flavor::GoblinUltra::FF eta);

} // namespace proof_system::honk::prover_library