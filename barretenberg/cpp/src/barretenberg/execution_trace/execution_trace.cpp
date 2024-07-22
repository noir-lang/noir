#include "execution_trace.hpp"
#include "barretenberg/flavor/plonk_flavors.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_keccak.hpp"
namespace bb {

template <class Flavor>
void ExecutionTrace_<Flavor>::populate(Builder& builder, typename Flavor::ProvingKey& proving_key, bool is_structured)
{
    // Construct wire polynomials, selector polynomials, and copy cycles from raw circuit data
    auto trace_data = construct_trace_data(builder, proving_key.circuit_size, is_structured);

    add_wires_and_selectors_to_proving_key(trace_data, builder, proving_key);

    if constexpr (IsUltraPlonkOrHonk<Flavor>) {
        add_memory_records_to_proving_key(trace_data, builder, proving_key);
    }

    if constexpr (IsGoblinFlavor<Flavor>) {
        add_ecc_op_wires_to_proving_key(builder, proving_key);
    }

    // Compute the permutation argument polynomials (sigma/id) and add them to proving key
    compute_permutation_argument_polynomials<Flavor>(builder, &proving_key, trace_data.copy_cycles);
}

template <class Flavor>
void ExecutionTrace_<Flavor>::add_wires_and_selectors_to_proving_key(TraceData& trace_data,
                                                                     Builder& builder,
                                                                     typename Flavor::ProvingKey& proving_key)
{
    if constexpr (IsHonkFlavor<Flavor>) {
        for (auto [pkey_wire, trace_wire] : zip_view(proving_key.polynomials.get_wires(), trace_data.wires)) {
            pkey_wire = trace_wire.share();
        }
        proving_key.polynomials.set_shifted(); // Ensure shifted wires are set correctly
        for (auto [pkey_selector, trace_selector] :
             zip_view(proving_key.polynomials.get_selectors(), trace_data.selectors)) {
            pkey_selector = trace_selector.share();
        }
        proving_key.pub_inputs_offset = trace_data.pub_inputs_offset;
    } else if constexpr (IsPlonkFlavor<Flavor>) {
        for (size_t idx = 0; idx < trace_data.wires.size(); ++idx) {
            std::string wire_tag = "w_" + std::to_string(idx + 1) + "_lagrange";
            proving_key.polynomial_store.put(wire_tag, std::move(trace_data.wires[idx]));
        }
        for (size_t idx = 0; idx < trace_data.selectors.size(); ++idx) {
            proving_key.polynomial_store.put(builder.selector_names[idx] + "_lagrange",
                                             std::move(trace_data.selectors[idx]));
        }
    }
}

template <class Flavor>
void ExecutionTrace_<Flavor>::add_memory_records_to_proving_key(TraceData& trace_data,
                                                                Builder& builder,
                                                                typename Flavor::ProvingKey& proving_key)
    requires IsUltraPlonkOrHonk<Flavor>
{
    ASSERT(proving_key.memory_read_records.empty() && proving_key.memory_write_records.empty());

    // Update indices of RAM/ROM reads/writes based on where block containing these gates sits in the trace
    for (auto& index : builder.memory_read_records) {
        proving_key.memory_read_records.emplace_back(index + trace_data.ram_rom_offset);
    }
    for (auto& index : builder.memory_write_records) {
        proving_key.memory_write_records.emplace_back(index + trace_data.ram_rom_offset);
    }
}

template <class Flavor>
typename ExecutionTrace_<Flavor>::TraceData ExecutionTrace_<Flavor>::construct_trace_data(Builder& builder,
                                                                                          size_t dyadic_circuit_size,
                                                                                          bool is_structured)
{
    TraceData trace_data{ dyadic_circuit_size, builder };

    // Complete the public inputs execution trace block from builder.public_inputs
    populate_public_inputs_block(builder);

    uint32_t offset = Flavor::has_zero_row ? 1 : 0; // Offset at which to place each block in the trace polynomials
    // For each block in the trace, populate wire polys, copy cycles and selector polys
    for (auto& block : builder.blocks.get()) {
        auto block_size = static_cast<uint32_t>(block.size());

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
                trace_data.copy_cycles[real_var_idx].emplace_back(cycle_node{ wire_idx, trace_row_idx });
            }
        }

        // Insert the selector values for this block into the selector polynomials at the correct offset
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/398): implicit arithmetization/flavor consistency
        for (auto [selector_poly, selector] : zip_view(trace_data.selectors, block.selectors)) {
            for (size_t row_idx = 0; row_idx < block_size; ++row_idx) {
                size_t trace_row_idx = row_idx + offset;
                selector_poly[trace_row_idx] = selector[row_idx];
            }
        }

        // Store the offset of the block containing RAM/ROM read/write gates for use in updating memory records
        if (block.has_ram_rom) {
            trace_data.ram_rom_offset = offset;
        }
        // Store offset of public inputs block for use in the pub input mechanism of the permutation argument
        if (block.is_pub_inputs) {
            trace_data.pub_inputs_offset = offset;
        }

        // If the trace is structured, we populate the data from the next block at a fixed block size offset
        if (is_structured) {
            offset += block.get_fixed_size();
        } else { // otherwise, the next block starts immediately following the previous one
            offset += block_size;
        }
    }
    return trace_data;
}

template <class Flavor> void ExecutionTrace_<Flavor>::populate_public_inputs_block(Builder& builder)
{
    // Update the public inputs block
    for (auto& idx : builder.public_inputs) {
        for (size_t wire_idx = 0; wire_idx < NUM_WIRES; ++wire_idx) {
            if (wire_idx < 2) { // first two wires get a copy of the public inputs
                builder.blocks.pub_inputs.wires[wire_idx].emplace_back(idx);
            } else { // the remaining wires get zeros
                builder.blocks.pub_inputs.wires[wire_idx].emplace_back(builder.zero_idx);
            }
        }
        for (auto& selector : builder.blocks.pub_inputs.selectors) {
            selector.emplace_back(0);
        }
    }
}

template <class Flavor>
void ExecutionTrace_<Flavor>::add_ecc_op_wires_to_proving_key(Builder& builder,
                                                              typename Flavor::ProvingKey& proving_key)
    requires IsGoblinFlavor<Flavor>
{
    // Copy the ecc op data from the conventional wires into the op wires over the range of ecc op gates
    auto& ecc_op_selector = proving_key.polynomials.lagrange_ecc_op;
    const size_t op_wire_offset = Flavor::has_zero_row ? 1 : 0;
    for (auto [ecc_op_wire, wire] :
         zip_view(proving_key.polynomials.get_ecc_op_wires(), proving_key.polynomials.get_wires())) {
        for (size_t i = 0; i < builder.blocks.ecc_op.size(); ++i) {
            size_t idx = i + op_wire_offset;
            ecc_op_wire[idx] = wire[idx];
            ecc_op_selector[idx] = 1; // construct selector as the indicator on the ecc op block
        }
    }
}

template class ExecutionTrace_<UltraFlavor>;
template class ExecutionTrace_<UltraKeccakFlavor>;
template class ExecutionTrace_<MegaFlavor>;
template class ExecutionTrace_<plonk::flavor::Standard>;
template class ExecutionTrace_<plonk::flavor::Ultra>;

} // namespace bb