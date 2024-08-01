#include "barretenberg/plonk_honk_shared/arithmetization/mega_arithmetization.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"

namespace bb {

/**
 * @brief A debugging utility for tracking the max size of each block over all circuits in the IVC
 *
 */
struct MaxBlockSizeTracker {
    using Builder = MegaCircuitBuilder;
    using MegaTraceBlocks = MegaArithmetization::MegaTraceBlocks<size_t>;
    MegaTraceBlocks max_sizes;

    MaxBlockSizeTracker()
    {
        for (auto& size : max_sizes.get()) {
            size = 0; // init max sizes to zero
        }
    }

    // Update the max block sizes based on the block sizes of a provided circuit
    void update(Builder& circuit)
    {
        for (auto [block, max_size] : zip_view(circuit.blocks.get(), max_sizes.get())) {
            max_size = std::max(block.size(), max_size);
        }
    }

    // For printing only. Must match the order of the members in the arithmetization
    std::vector<std::string> block_labels{ "ecc_op",           "pub_inputs", "arithmetic",
                                           "delta_range",      "elliptic",   "aux",
                                           "lookup",           "busread",    "poseidon_external",
                                           "poseidon_internal" };

    void print()
    {
        info("Minimum required block sizes for structured trace: ");
        for (auto [label, max_size] : zip_view(block_labels, max_sizes.get())) {
            std::cout << std::left << std::setw(20) << (label + ":") << max_size << std::endl;
        }
        info("");
    }
};
} // namespace bb