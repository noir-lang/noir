#pragma once
#include "barretenberg/execution_trace/execution_trace.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/plonk_honk_shared/composer/composer_lib.hpp"
#include "barretenberg/plonk_honk_shared/composer/permutation_lib.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib_circuit_builders/goblin_ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"

namespace bb {
/**
 * @brief  A ProverInstance is normally constructed from a finalized circuit and it contains all the information
 * required by an Ultra Goblin Honk prover to create a proof. A ProverInstance is also the result of running the
 * Protogalaxy prover, in which case it becomes a relaxed counterpart with the folding parameters (target sum and gate
 * challenges set to non-zero values).
 *
 * @details This is the equivalent of ω in the paper.
 */

template <class Flavor> class ProverInstance_ {
    using Circuit = typename Flavor::CircuitBuilder;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using FF = typename Flavor::FF;
    using ProverPolynomials = typename Flavor::ProverPolynomials;
    using Polynomial = typename Flavor::Polynomial;
    using RelationSeparator = typename Flavor::RelationSeparator;

    using Trace = ExecutionTrace_<Flavor>;

  public:
    ProvingKey proving_key;
    ProverPolynomials prover_polynomials;

    RelationSeparator alphas;
    bb::RelationParameters<FF> relation_parameters;

    bool is_accumulator = false;

    // The folding parameters (\vec{β}, e) which are set for accumulators (i.e. relaxed instances).
    std::vector<FF> gate_challenges;
    FF target_sum;

    ProverInstance_(Circuit& circuit)
    {
        BB_OP_COUNT_TIME_NAME("ProverInstance(Circuit&)");
        circuit.add_gates_to_ensure_all_polys_are_non_zero();
        circuit.finalize_circuit();
        if constexpr (IsGoblinFlavor<Flavor>) {
            circuit.op_queue->append_nonzero_ops();
        }

        dyadic_circuit_size = compute_dyadic_size(circuit);

        proving_key = std::move(ProvingKey(dyadic_circuit_size, circuit.public_inputs.size()));

        // Construct and add to proving key the wire, selector and copy constraint polynomials
        Trace::populate(circuit, proving_key);

        // If Goblin, construct the databus polynomials
        if constexpr (IsGoblinFlavor<Flavor>) {
            construct_databus_polynomials(circuit);
        }

        // First and last lagrange polynomials (in the full circuit size)
        const auto [lagrange_first, lagrange_last] =
            compute_first_and_last_lagrange_polynomials<FF>(dyadic_circuit_size);
        proving_key.lagrange_first = lagrange_first;
        proving_key.lagrange_last = lagrange_last;

        construct_table_polynomials(circuit, dyadic_circuit_size);

        proving_key.sorted_polynomials = construct_sorted_list_polynomials<Flavor>(circuit, dyadic_circuit_size);

        std::span<FF> public_wires_source = proving_key.w_r;

        // Construct the public inputs array
        for (size_t i = 0; i < proving_key.num_public_inputs; ++i) {
            size_t idx = i + proving_key.pub_inputs_offset;
            proving_key.public_inputs.emplace_back(public_wires_source[idx]);
        }
    }

    ProverInstance_() = default;
    ~ProverInstance_() = default;

    void compute_databus_id()
        requires IsGoblinFlavor<Flavor>;

  private:
    static constexpr size_t num_zero_rows = Flavor::has_zero_row ? 1 : 0;
    static constexpr size_t NUM_WIRES = Circuit::NUM_WIRES;
    size_t dyadic_circuit_size = 0; // final power-of-2 circuit size

    size_t compute_dyadic_size(Circuit&);

    void construct_databus_polynomials(Circuit&)
        requires IsGoblinFlavor<Flavor>;

    void construct_table_polynomials(Circuit&, size_t);
};

} // namespace bb
