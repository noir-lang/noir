
#include "barretenberg/vm/avm_trace/gadgets/avm_conversion_trace.hpp"

namespace bb::avm_trace {

AvmConversionTraceBuilder::AvmConversionTraceBuilder()
{
    conversion_trace.reserve(AVM_TRACE_SIZE);
}

std::vector<AvmConversionTraceBuilder::ConversionTraceEntry> AvmConversionTraceBuilder::finalize()
{
    return std::move(conversion_trace);
}

void AvmConversionTraceBuilder::reset()
{
    conversion_trace.clear();
}

/**
 * @brief Build conversion trace and compute the result of a TO_RADIX_LE operation.
 *        This operation is only valid for the FF instr_tag and always returns a byte array
 *
 * @param a First operand of the TO_RADIX_LE, the value to be converted
 * @param radix The upper bound for each limbm 0 <= limb < radix
 * @param num_limbs The number of limbs to the value into.
 * @param in_tag Instruction tag defining the number of bits for the LT.
 * @param clk Clock referring to the operation in the main trace.
 *
 * @return std::vector<uint8_t> The LE converted values stored as bytes.
 */
std::vector<uint8_t> AvmConversionTraceBuilder::op_to_radix_le(FF const& a,
                                                               uint32_t radix,
                                                               uint32_t num_limbs,
                                                               uint32_t clk)
{
    ASSERT(radix <= 256);

    auto a_uint256 = uint256_t(a);
    auto radix_uint256 = uint256_t(radix);

    std::vector<uint8_t> bytes{};
    for (uint32_t i = 0; i < num_limbs; i++) {
        bytes.emplace_back(static_cast<uint8_t>(a_uint256 % radix_uint256));
        a_uint256 /= radix_uint256;
    }

    conversion_trace.emplace_back(ConversionTraceEntry{
        .conversion_clk = clk,
        .to_radix_le_sel = true,
        .input = a,
        .radix = radix,
        .num_limbs = num_limbs,
        .limbs = bytes,
    });

    return bytes;
}

} // namespace bb::avm_trace
