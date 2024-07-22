#pragma once
#include "barretenberg/execution_trace/execution_trace.hpp"
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/mega_arithmetization.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/ultra_arithmetization.hpp"
#include "barretenberg/plonk_honk_shared/composer/composer_lib.hpp"
#include "barretenberg/plonk_honk_shared/composer/permutation_lib.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_keccak.hpp"

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

    RelationSeparator alphas;
    bb::RelationParameters<FF> relation_parameters;

    bool is_accumulator = false;

    // The folding parameters (\vec{β}, e) which are set for accumulators (i.e. relaxed instances).
    std::vector<FF> gate_challenges;
    FF target_sum;

    ProverInstance_(Circuit& circuit, TraceStructure trace_structure = TraceStructure::NONE)
    {
        BB_OP_COUNT_TIME_NAME("ProverInstance(Circuit&)");
        circuit.add_gates_to_ensure_all_polys_are_non_zero();
        circuit.finalize_circuit();

        // Set flag indicating whether the polynomials will be constructed with fixed block sizes for each gate type
        const bool is_structured = (trace_structure != TraceStructure::NONE);

        // If using a structured trace, set fixed block sizes, check their validity, and set the dyadic circuit size
        if (is_structured) {
            circuit.blocks.set_fixed_block_sizes(trace_structure); // set the fixed sizes for each block
            circuit.blocks.check_within_fixed_sizes();             // ensure that no block exceeds its fixed size
            dyadic_circuit_size = compute_structured_dyadic_size(circuit); // set the dyadic size accordingly
        } else {
            dyadic_circuit_size = compute_dyadic_size(circuit); // set dyadic size directly from circuit block sizes
        }

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/905): This is adding ops to the op queue but NOT to
        // the circuit, meaning the ECCVM/Translator will use different ops than the main circuit. This will lead to
        // failure once https://github.com/AztecProtocol/barretenberg/issues/746 is resolved.
        if constexpr (IsGoblinFlavor<Flavor>) {
            circuit.op_queue->append_nonzero_ops();
        }

        proving_key = ProvingKey(dyadic_circuit_size, circuit.public_inputs.size());

        // Construct and add to proving key the wire, selector and copy constraint polynomials
        Trace::populate(circuit, proving_key, is_structured);

        // If Goblin, construct the databus polynomials
        if constexpr (IsGoblinFlavor<Flavor>) {
            construct_databus_polynomials(circuit);
        }

        // First and last lagrange polynomials (in the full circuit size)
        proving_key.polynomials.lagrange_first[0] = 1;
        proving_key.polynomials.lagrange_last[dyadic_circuit_size - 1] = 1;

        construct_lookup_table_polynomials<Flavor>(proving_key.polynomials.get_tables(), circuit, dyadic_circuit_size);

        construct_lookup_read_counts<Flavor>(proving_key.polynomials.lookup_read_counts,
                                             proving_key.polynomials.lookup_read_tags,
                                             circuit,
                                             dyadic_circuit_size);

        std::span<FF> public_wires_source = proving_key.polynomials.w_r;

        // Construct the public inputs array
        for (size_t i = 0; i < proving_key.num_public_inputs; ++i) {
            size_t idx = i + proving_key.pub_inputs_offset;
            proving_key.public_inputs.emplace_back(public_wires_source[idx]);
        }
    }

    ProverInstance_() = default;
    ~ProverInstance_() = default;

  private:
    static constexpr size_t num_zero_rows = Flavor::has_zero_row ? 1 : 0;
    static constexpr size_t NUM_WIRES = Circuit::NUM_WIRES;
    size_t dyadic_circuit_size = 0; // final power-of-2 circuit size

    size_t compute_dyadic_size(Circuit&);

    /**
     * @brief Compute dyadic size based on a structured trace with fixed block size
     *
     */
    size_t compute_structured_dyadic_size(Circuit& builder)
    {
        size_t minimum_size = builder.blocks.get_total_structured_size();
        return builder.get_circuit_subgroup_size(minimum_size);
    }

    void construct_databus_polynomials(Circuit&)
        requires IsGoblinFlavor<Flavor>;
};

} // namespace bb
