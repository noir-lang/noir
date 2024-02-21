#include "execution_trace.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include "barretenberg/flavor/plonk_flavors.hpp"
#include "barretenberg/flavor/ultra.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
namespace bb {

template <class Flavor>
void ExecutionTrace_<Flavor>::generate(const Builder& builder,
                                       const std::shared_ptr<typename Flavor::ProvingKey>& proving_key)
{
    // Construct wire polynomials, selector polynomials, and copy cycles from raw circuit data
    auto trace_data = construct_trace_data(builder, proving_key->circuit_size);

    add_wires_and_selectors_to_proving_key(trace_data, builder, proving_key);

    // Compute the permutation argument polynomials (sigma/id) and add them to proving key
    compute_permutation_argument_polynomials<Flavor>(builder, proving_key.get(), trace_data.copy_cycles);
}

template <class Flavor>
void ExecutionTrace_<Flavor>::add_wires_and_selectors_to_proving_key(
    TraceData& trace_data, const Builder& builder, const std::shared_ptr<typename Flavor::ProvingKey>& proving_key)
{
    if constexpr (IsHonkFlavor<Flavor>) {
        for (auto [pkey_wire, trace_wire] : zip_view(proving_key->get_wires(), trace_data.wires)) {
            pkey_wire = std::move(trace_wire);
        }
        for (auto [pkey_selector, trace_selector] : zip_view(proving_key->get_selectors(), trace_data.selectors)) {
            pkey_selector = std::move(trace_selector);
        }
    } else if constexpr (IsPlonkFlavor<Flavor>) {
        for (size_t idx = 0; idx < trace_data.wires.size(); ++idx) {
            std::string wire_tag = "w_" + std::to_string(idx + 1) + "_lagrange";
            proving_key->polynomial_store.put(wire_tag, std::move(trace_data.wires[idx]));
        }
        for (size_t idx = 0; idx < trace_data.selectors.size(); ++idx) {
            proving_key->polynomial_store.put(builder.selector_names[idx] + "_lagrange",
                                              std::move(trace_data.selectors[idx]));
        }
    }
}

template <class Flavor>
typename ExecutionTrace_<Flavor>::TraceData ExecutionTrace_<Flavor>::construct_trace_data(const Builder& builder,
                                                                                          size_t dyadic_circuit_size)
{
    TraceData trace_data{ dyadic_circuit_size, builder };

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/862): Eventually trace_blocks will be constructed
    // directly in the builder, i.e. the gate addition methods will directly populate the wire/selectors in the
    // appropriate block. In the mean time we do some inefficient copying etc to construct it here post facto.
    auto trace_blocks = create_execution_trace_blocks(builder);

    uint32_t offset = 0; // Track offset at which to place each block in the trace polynomials
    // For each block in the trace, populate wire polys, copy cycles and selector polys
    for (auto& block : trace_blocks) {
        auto block_size = static_cast<uint32_t>(block.wires[0].size());
        info("block size = ", block_size);

        // Update wire polynomials and copy cycles
        // NB: The order of row/column loops is arbitrary but needs to be row/column to match old copy_cycle code
        for (uint32_t block_row_idx = 0; block_row_idx < block_size; ++block_row_idx) {
            for (uint32_t wire_idx = 0; wire_idx < NUM_WIRES; ++wire_idx) {
                uint32_t var_idx = block.wires[wire_idx][block_row_idx]; // an index into the variables array
                uint32_t real_var_idx = builder.real_variable_index[var_idx];
                uint32_t trace_row_idx = block_row_idx + offset;
                // Insert the real witness values from this block into the wire polys at the correct offset
                trace_data.wires[wire_idx][trace_row_idx] = builder.get_variable(var_idx);
                // Add the address of the witness value to its corresponding copy cycle
                // NB: Not adding cycles for wires 3 and 4 here is only needed in order to maintain consistency with old
                // version. We can remove this special case and the result is simply that all the zeros in wires 3 and 4
                // over the PI range are copy constrained together, but this changes sigma/id which changes the vkey.
                if (!(block.is_public_input && wire_idx > 1)) {
                    trace_data.copy_cycles[real_var_idx].emplace_back(cycle_node{ wire_idx, trace_row_idx });
                }
            }
        }

        // Insert the selector values for this block into the selector polynomials at the correct offset
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/398): implicit arithmetization/flavor consistency
        for (auto [selector_poly, selector] : zip_view(trace_data.selectors, block.selectors.get())) {
            for (size_t row_idx = 0; row_idx < block_size; ++row_idx) {
                size_t trace_row_idx = row_idx + offset;
                selector_poly[trace_row_idx] = selector[row_idx];
            }
        }

        offset += block_size;
    }
    return trace_data;
}

template <class Flavor>
std::vector<typename ExecutionTrace_<Flavor>::TraceBlock> ExecutionTrace_<Flavor>::create_execution_trace_blocks(
    const Builder& builder)
{
    std::vector<TraceBlock> trace_blocks;

    // Make a block for the zero row
    if constexpr (Flavor::has_zero_row) {
        TraceBlock zero_block;
        for (auto& wire : zero_block.wires) {
            wire.emplace_back(builder.zero_idx);
        }
        for (auto& selector : zero_block.selectors.get()) {
            selector.emplace_back(0);
        }
        trace_blocks.emplace_back(zero_block);
    }

    // Make a block for the ecc op wires
    if constexpr (IsGoblinFlavor<Flavor>) {
        trace_blocks.emplace_back(builder.ecc_op_block);
    }

    // Make a block for the public inputs
    TraceBlock public_block;
    for (auto& idx : builder.public_inputs) {
        for (size_t wire_idx = 0; wire_idx < NUM_WIRES; ++wire_idx) {
            if (wire_idx < 2) { // first two wires get a copy of the public inputs
                public_block.wires[wire_idx].emplace_back(idx);
            } else { // the remaining wires get zeros
                public_block.wires[wire_idx].emplace_back(builder.zero_idx);
            }
        }
        for (auto& selector : public_block.selectors.get()) {
            selector.emplace_back(0);
        }
    }

    public_block.is_public_input = true;
    trace_blocks.emplace_back(public_block);

    // Make a block for the basic wires and selectors
    TraceBlock conventional_block{ builder.wires, builder.selectors };
    trace_blocks.emplace_back(conventional_block);

    return trace_blocks;
}

template class ExecutionTrace_<UltraFlavor>;
template class ExecutionTrace_<GoblinUltraFlavor>;
template class ExecutionTrace_<plonk::flavor::Standard>;
template class ExecutionTrace_<plonk::flavor::Ultra>;

} // namespace bb