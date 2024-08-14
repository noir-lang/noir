#include "barretenberg/vm/avm/trace/helper.hpp"

#include <algorithm>
#include <cassert>

#include "barretenberg/vm/avm/trace/mem_trace.hpp"

namespace bb::avm_trace {

template <typename FF> std::string field_to_string(const FF& ff)
{
    std::ostringstream os;
    os << ff;
    std::string raw = os.str();
    auto first_not_zero = raw.find_first_not_of('0', 2);
    std::string result = "0x" + (first_not_zero != std::string::npos ? raw.substr(first_not_zero) : "0");
    return result;
}

void dump_trace_as_csv(std::vector<Row> const& trace, std::filesystem::path const& filename)
{
    std::ofstream file;
    file.open(filename);

    // Filter zero columns indices (ugly and slow).
    std::set<size_t> non_zero_columns;
    const size_t num_columns = Row::names().size();
    for (const Row& row : trace) {
        const auto row_vec = row.as_vector();
        for (size_t i = 0; i < num_columns; ++i) {
            if (row_vec[i] != 0) {
                non_zero_columns.insert(i);
            }
        }
    }
    std::vector<size_t> sorted_non_zero_columns(non_zero_columns.begin(), non_zero_columns.end());
    std::sort(sorted_non_zero_columns.begin(), sorted_non_zero_columns.end());

    const auto& names = Row::names();
    file << "ROW_NUMBER,";
    for (const auto& column_idx : sorted_non_zero_columns) {
        file << names[column_idx] << ",";
    }
    file << std::endl;

    for (size_t r = 0; r < trace.size(); ++r) {
        // Filter zero rows.
        const auto& row_vec = trace[r].as_vector();
        bool all_zero = true;
        for (const auto& column_idx : sorted_non_zero_columns) {
            if (row_vec[column_idx] != 0) {
                all_zero = false;
                break;
            }
        }
        if (!all_zero) {
            file << r << ",";
            for (const auto& column_idx : sorted_non_zero_columns) {
                file << field_to_string(row_vec[column_idx]) << ",";
            }
            file << std::endl;
        }
    }
}

bool is_operand_indirect(uint8_t ind_value, uint8_t operand_idx)
{
    if (operand_idx > 7) {
        return false;
    }

    return static_cast<bool>((ind_value & (1 << operand_idx)) >> operand_idx);
}

std::vector<std::vector<FF>> copy_public_inputs_columns(VmPublicInputs const& public_inputs,
                                                        std::vector<FF> const& calldata,
                                                        std::vector<FF> const& returndata)
{
    // We convert to a vector as the pil generated verifier is generic and unaware of the KERNEL_INPUTS_LENGTH
    // For each of the public input vectors
    std::vector<FF> public_inputs_kernel_inputs(std::get<KERNEL_INPUTS>(public_inputs).begin(),
                                                std::get<KERNEL_INPUTS>(public_inputs).end());
    std::vector<FF> public_inputs_kernel_value_outputs(std::get<KERNEL_OUTPUTS_VALUE>(public_inputs).begin(),
                                                       std::get<KERNEL_OUTPUTS_VALUE>(public_inputs).end());
    std::vector<FF> public_inputs_kernel_side_effect_outputs(
        std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(public_inputs).begin(),
        std::get<KERNEL_OUTPUTS_SIDE_EFFECT_COUNTER>(public_inputs).end());
    std::vector<FF> public_inputs_kernel_metadata_outputs(std::get<KERNEL_OUTPUTS_METADATA>(public_inputs).begin(),
                                                          std::get<KERNEL_OUTPUTS_METADATA>(public_inputs).end());

    assert(public_inputs_kernel_inputs.size() == KERNEL_INPUTS_LENGTH);
    assert(public_inputs_kernel_value_outputs.size() == KERNEL_OUTPUTS_LENGTH);
    assert(public_inputs_kernel_side_effect_outputs.size() == KERNEL_OUTPUTS_LENGTH);
    assert(public_inputs_kernel_metadata_outputs.size() == KERNEL_OUTPUTS_LENGTH);

    return {
        std::move(public_inputs_kernel_inputs),
        std::move(public_inputs_kernel_value_outputs),
        std::move(public_inputs_kernel_side_effect_outputs),
        std::move(public_inputs_kernel_metadata_outputs),
        calldata,
        returndata,
    };
}

} // namespace bb::avm_trace
