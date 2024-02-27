#pragma once
#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/proof_system/composer/permutation_lib.hpp"
#include "barretenberg/srs/global_crs.hpp"

namespace bb {

template <class Flavor> class ExecutionTrace_ {
    using Builder = typename Flavor::CircuitBuilder;
    using Polynomial = typename Flavor::Polynomial;
    using FF = typename Flavor::FF;
    using TrackBlocks = typename Builder::Arithmetization::TraceBlocks;
    using Wires = std::array<std::vector<uint32_t, bb::ContainerSlabAllocator<uint32_t>>, Builder::NUM_WIRES>;
    using ProvingKey = typename Flavor::ProvingKey;

  public:
    static constexpr size_t NUM_WIRES = Builder::NUM_WIRES;

    struct TraceData {
        std::array<Polynomial, NUM_WIRES> wires;
        std::array<Polynomial, Builder::Arithmetization::NUM_SELECTORS> selectors;
        // A vector of sets (vectors) of addresses into the wire polynomials whose values are copy constrained
        std::vector<CyclicPermutation> copy_cycles;

        TraceData(size_t dyadic_circuit_size, Builder& builder)
        {
            // Initializate the wire and selector polynomials
            for (auto& wire : wires) {
                wire = Polynomial(dyadic_circuit_size);
            }
            for (auto& selector : selectors) {
                selector = Polynomial(dyadic_circuit_size);
            }
            copy_cycles.resize(builder.variables.size());
        }
    };

    /**
     * @brief Given a circuit, populate a proving key with wire polys, selector polys, and sigma/id polys
     *
     * @param builder
     */
    static void populate(Builder& builder, const std::shared_ptr<ProvingKey>&);

  private:
    /**
     * @brief Add the wire and selector polynomials from the trace data to a honk or plonk proving key
     *
     * @param trace_data
     * @param builder
     * @param proving_key
     */
    static void add_wires_and_selectors_to_proving_key(TraceData& trace_data,
                                                       Builder& builder,
                                                       const std::shared_ptr<typename Flavor::ProvingKey>& proving_key);

    /**
     * @brief Construct wire polynomials, selector polynomials and copy cycles from raw circuit data
     *
     * @param builder
     * @param dyadic_circuit_size
     * @return TraceData
     */
    static TraceData construct_trace_data(Builder& builder, size_t dyadic_circuit_size);

    /**
     * @brief Populate the public inputs block
     * @details The first two wires are a copy of the public inputs and the other wires and all selectors are zero
     *
     * @param builder
     */
    static void populate_public_inputs_block(Builder& builder);

    /**
     * @brief Construct and add the goblin ecc op wires to the proving key
     * @details The ecc op wires vanish everywhere except on the ecc op block, where they contain a copy of the ecc op
     * data assumed already to be present in the corrresponding block of the conventional wires in the proving key.
     *
     * @param builder
     * @param proving_key
     */
    static void add_ecc_op_wires_to_proving_key(Builder& builder,
                                                const std::shared_ptr<typename Flavor::ProvingKey>& proving_key)
        requires IsGoblinFlavor<Flavor>;
};

} // namespace bb