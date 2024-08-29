#include "barretenberg/vm/avm/trace/alu_trace.hpp"
#include "barretenberg/vm/avm/trace/gadgets/range_check.hpp"

namespace bb::avm_trace {

/**************************************************************************************************
 *                              HELPERS IN ANONYMOUS NAMESPACE
 **************************************************************************************************/
namespace {

/**
 * Helper function to decompose a uint256_t into a b-lower bits and (256-b) upper bits
 * The outputs are cast to uint256_t so they are easier to use in checks
 */
std::tuple<uint256_t, uint256_t> decompose(uint256_t const& a, uint8_t const b)
{
    uint256_t upper_bitmask = (uint256_t(1) << uint256_t(b)) - 1;
    uint256_t a_lo = a & upper_bitmask;
    uint256_t a_hi = a >> b;
    return std::make_tuple(a_lo, a_hi);
}

// Returns the number of bits associated with a given memory tag
uint8_t mem_tag_bits(AvmMemoryTag in_tag)
{
    switch (in_tag) {
    case AvmMemoryTag::U8:
        return 8;
    case AvmMemoryTag::U16:
        return 16;
    case AvmMemoryTag::U32:
        return 32;
    case AvmMemoryTag::U64:
        return 64;
    case AvmMemoryTag::U128:
        return 128;
    case AvmMemoryTag::FF:
        return 254;
    case AvmMemoryTag::U0:
        return 0;
    }
    return 0;
}

// This is a helper that casts the input based on the AvmMemoryTag
// The input has to be uint256_t to handle larger inputs
FF cast_to_mem_tag(uint256_t input, AvmMemoryTag in_tag)
{
    switch (in_tag) {
    case AvmMemoryTag::U8:
        return FF{ static_cast<uint8_t>(input) };
    case AvmMemoryTag::U16:
        return FF{ static_cast<uint16_t>(input) };
    case AvmMemoryTag::U32:
        return FF{ static_cast<uint32_t>(input) };
    case AvmMemoryTag::U64:
        return FF{ static_cast<uint64_t>(input) };
    case AvmMemoryTag::U128:
        return FF{ uint256_t::from_uint128(uint128_t(input)) };
    case AvmMemoryTag::FF:
        return input;
    case AvmMemoryTag::U0:
        return FF{ 0 };
    }
    // Need this for gcc compilation even though we fully handle the switch cases
    // We should never reach this point
    __builtin_unreachable();
}

} // namespace

/**************************************************************************************************
 *                            RESET/FINALIZE
 **************************************************************************************************/
/**
 * @brief Resetting the internal state so that a new Alu trace can be rebuilt using the same object.
 *
 */
void AvmAluTraceBuilder::reset()
{
    alu_trace.clear();
    range_checked_required = false;
}

/**************************************************************************************************
 *                            COMPUTE - ARITHMETIC
 **************************************************************************************************/

/**
 * @brief Build Alu trace and compute the result of an addition of type defined by in_tag.
 *        Besides the addition calculation, for the types u8, u16, u32, u64, and u128, we
 *        have to store the result of the addition modulo 2^128 decomposed into 8-bit and
 *        16-bit registers, i.e.,
 *        a+b mod. 2^128 =  alu_u8_r0 + alu_u8_r1 * 2^8 + alu_u16_r0 * 2^16 ... +  alu_u16_r6 * 2^112
 *
 * @param a Left operand of the addition
 * @param b Right operand of the addition
 * @param in_tag Instruction tag defining the number of bits on which the addition applies.
 *               It is assumed that the caller never uses the type u0.
 * @param clk Clock referring to the operation in the main trace.
 *
 * @return FF The result of the addition casted in a finite field element
 */
FF AvmAluTraceBuilder::op_add(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    bool carry = false;
    uint256_t c_u256 = uint256_t(a) + uint256_t(b);
    FF c = cast_to_mem_tag(c_u256, in_tag);

    if (in_tag != AvmMemoryTag::FF) {
        // a_u128 + b_u128 >= 2^128  <==> c_u128 < a_u128
        if (uint128_t(c) < uint128_t(a)) {
            carry = true;
        }
        cmp_builder.range_check_builder.assert_range(uint128_t(c), mem_tag_bits(in_tag), EventEmitter::ALU, clk);
    }

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .opcode = OpCode::ADD,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_cf = carry,
        .range_check_input = c,
        .range_check_num_bits = in_tag != AvmMemoryTag::FF ? mem_tag_bits(in_tag) : 0,
        .range_check_sel = in_tag != AvmMemoryTag::FF,
    });
    return c;
}

/**
 * @brief Build Alu trace and compute the result of a subtraction of type defined by in_tag.
 *        Besides the subtraction calculation, for the types u8, u16, u32, u64, and u128, we
 *        have to store the result of the subtraction modulo 2^128 decomposed into 8-bit and
 *        16-bit registers, i.e.,
 *        a-b mod. 2^128 = alu_u8_r0 + alu_u8_r1 * 2^8 + alu_u16_r0 * 2^16 ... +  alu_u16_r6 * 2^112
 *
 * @param a Left operand of the subtraction
 * @param b Right operand of the subtraction
 * @param in_tag Instruction tag defining the number of bits on which the subtracttion applies.
 *               It is assumed that the caller never uses the type u0.
 * @param clk Clock referring to the operation in the main trace.
 *
 * @return FF The result of the subtraction casted in a finite field element
 */
FF AvmAluTraceBuilder::op_sub(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    bool carry = false;
    uint256_t c_u256 = uint256_t(a) - uint256_t(b);
    FF c = cast_to_mem_tag(c_u256, in_tag);

    if (in_tag != AvmMemoryTag::FF) {
        // Underflow when a_u128 < b_u128
        if (uint128_t(a) < uint128_t(b)) {
            carry = true;
        }
        cmp_builder.range_check_builder.assert_range(uint128_t(c), mem_tag_bits(in_tag), EventEmitter::ALU, clk);
    }

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .opcode = OpCode::SUB,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_cf = carry,
        .range_check_input = c,
        .range_check_num_bits = in_tag != AvmMemoryTag::FF ? mem_tag_bits(in_tag) : 0,
        .range_check_sel = in_tag != AvmMemoryTag::FF,
    });
    return c;
}

/**
 * @brief Build Alu trace and compute the result of an multiplication of type defined by in_tag.
 *
 * @param a Left operand of the multiplication
 * @param b Right operand of the multiplication
 * @param in_tag Instruction tag defining the number of bits on which the multiplication applies.
 *               It is assumed that the caller never uses the type u0.
 * @param clk Clock referring to the operation in the main trace.
 *
 * @return FF The result of the multiplication casted in a finite field element
 */
FF AvmAluTraceBuilder::op_mul(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    uint256_t a_u256{ a };
    uint256_t b_u256{ b };
    uint256_t c_u256 = a_u256 * b_u256; // Multiplication over the integers (not mod. 2^128)

    FF c = cast_to_mem_tag(c_u256, in_tag);

    uint8_t limb_bits = mem_tag_bits(in_tag) / 2;
    uint8_t num_bits = mem_tag_bits(in_tag);

    // Decompose a
    auto [alu_a_lo, alu_a_hi] = decompose(a_u256, limb_bits);
    // Decompose b
    auto [alu_b_lo, alu_b_hi] = decompose(b_u256, limb_bits);

    uint256_t partial_prod = alu_a_lo * alu_b_hi + alu_a_hi * alu_b_lo;
    // Decompose the partial product
    auto [partial_prod_lo, partial_prod_hi] = decompose(partial_prod, limb_bits);

    auto c_hi = c_u256 >> num_bits;

    if (in_tag != AvmMemoryTag::FF) {
        cmp_builder.range_check_builder.assert_range(uint128_t(c), mem_tag_bits(in_tag), EventEmitter::ALU, clk);
    }

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .opcode = OpCode::MUL,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_a_lo = alu_a_lo,
        .alu_a_hi = alu_a_hi,
        .alu_b_lo = alu_b_lo,
        .alu_b_hi = alu_b_hi,
        .alu_c_lo = c,
        .alu_c_hi = c_hi,
        .partial_prod_lo = partial_prod_lo,
        .partial_prod_hi = partial_prod_hi,
        .range_check_input = c,
        .range_check_num_bits = in_tag != AvmMemoryTag::FF ? mem_tag_bits(in_tag) : 0,
        .range_check_sel = in_tag != AvmMemoryTag::FF,
    });
    return c;
}

FF AvmAluTraceBuilder::op_div(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk)
{
    ASSERT(in_tag != AvmMemoryTag::FF);

    uint256_t a_u256{ a };
    uint256_t b_u256{ b };
    uint256_t c_u256 = a_u256 / b_u256;
    uint256_t rem_u256 = a_u256 % b_u256;

    // If dividing by zero, don't add any rows in the ALU, the error will be handled in the main trace
    if (b_u256 == 0) {
        return 0;
    }
    uint8_t limb_bits = mem_tag_bits(in_tag) / 2;
    uint8_t num_bits = mem_tag_bits(in_tag);
    // Decompose a
    auto [alu_a_lo, alu_a_hi] = decompose(b_u256, limb_bits);
    // Decompose b
    auto [alu_b_lo, alu_b_hi] = decompose(c_u256, limb_bits);

    uint256_t partial_prod = alu_a_lo * alu_b_hi + alu_a_hi * alu_b_lo;
    // Decompose the partial product
    auto [partial_prod_lo, partial_prod_hi] = decompose(partial_prod, limb_bits);

    // We perform the range checks here
    if (in_tag != AvmMemoryTag::FF) {
        cmp_builder.range_check_builder.assert_range(uint128_t(c_u256), mem_tag_bits(in_tag), EventEmitter::ALU, clk);
    }
    // Also check the remainder < divisor (i.e. remainder < b)
    bool is_gt = cmp_builder.constrained_gt(b, rem_u256, clk, EventEmitter::ALU);

    AvmAluTraceBuilder::AluTraceEntry row{
        .alu_clk = clk,
        .opcode = OpCode::DIV,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = FF{ c_u256 },
        .alu_a_lo = alu_a_lo,
        .alu_a_hi = alu_a_hi,
        .alu_b_lo = alu_b_lo,
        .alu_b_hi = alu_b_hi,
        .alu_c_lo = a,
        .alu_c_hi = a_u256 >> num_bits,
        .partial_prod_lo = partial_prod_lo,
        .partial_prod_hi = partial_prod_hi,
        .remainder = rem_u256,
        .range_check_input = FF{ c_u256 },
        .range_check_num_bits = in_tag != AvmMemoryTag::FF ? mem_tag_bits(in_tag) : 0,
        .range_check_sel = in_tag != AvmMemoryTag::FF,
        .cmp_input_a = b,
        .cmp_input_b = rem_u256,
        .cmp_result = FF{ static_cast<uint8_t>(is_gt) },
        .cmp_op_is_gt = true,
    };
    alu_trace.push_back(row);
    return c_u256;
}

/**************************************************************************************************
 *                            COMPUTE - COMPARATORS
 **************************************************************************************************/

/**
 * @brief Build Alu trace and return a boolean based on equality of operands of type defined by in_tag.
 *
 * @param a Left operand of the equality
 * @param b Right operand of the equality
 * @param in_tag Instruction tag defining the number of bits for equality
 * @param clk Clock referring to the operation in the main trace.
 *
 * @return FF The boolean result of equality casted to a finite field element
 */
FF AvmAluTraceBuilder::op_eq(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{

    bool res = cmp_builder.constrained_eq(a, b, clk, EventEmitter::ALU);

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .opcode = OpCode::EQ,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = FF(static_cast<uint8_t>(res)),
        .cmp_input_a = a,
        .cmp_input_b = b,
        .cmp_result = FF{ static_cast<uint8_t>(res) },
        .cmp_op_is_eq = true,
    });

    return FF{ static_cast<uint8_t>(res) };
}

/**
 * @brief Build Alu trace and compute the result of a LT operation on two operands.
 *        The tag type in_tag does not change the result of the operation. But we
 *        do need it for a relation check in the alu.
 *
 * @param a Left operand of the LT
 * @param b Right operand of the LT
 * @param clk Clock referring to the operation in the main trace.
 * @param in_tag Instruction tag defining the number of bits for the LT.
 *
 * @return FF The boolean result of LT casted to a finite field element
 */

FF AvmAluTraceBuilder::op_lt(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    // Note: This is counter-intuitive, to show that a < b we use the GT gadget with the inputs swapped
    bool result = cmp_builder.constrained_gt(b, a, clk, EventEmitter::ALU);
    bool c = result;

    // The subtlety is here that the circuit is designed as a GT(x,y) circuit, therefore we swap the inputs a & b
    AvmAluTraceBuilder::AluTraceEntry row{
        .alu_clk = clk,
        .opcode = OpCode::LT,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = FF(static_cast<uint8_t>(c)),
        .cmp_input_a = b,
        .cmp_input_b = a,
        .cmp_result = FF{ static_cast<uint8_t>(result) },
        .cmp_op_is_gt = true,
    };
    alu_trace.push_back(row);
    return FF{ static_cast<int>(c) };
}

/**
 * @brief Build Alu trace and compute the result of a LTE operation on two operands.
 *        The tag type in_tag does not change the result of the operation. But we
 *        do need it for a relation check in the alu.
 *
 * @param a Left operand of the LTE
 * @param b Right operand of the LTE
 * @param clk Clock referring to the operation in the main trace.
 * @param in_tag Instruction tag defining the number of bits for the LT.
 *
 * @return FF The boolean result of LTE casted to a finite field element
 */
FF AvmAluTraceBuilder::op_lte(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    // Note: This is counter-intuitive, to show that a <= b we actually show that a > b and then invert the answer
    bool result = cmp_builder.constrained_gt(a, b, clk, EventEmitter::ALU);
    bool c = !result;

    // Construct the row that performs the lte check
    AvmAluTraceBuilder::AluTraceEntry row{
        .alu_clk = clk,
        .opcode = OpCode::LTE,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = FF(static_cast<uint8_t>(c)),
        .cmp_input_a = a,
        .cmp_input_b = b,
        .cmp_result = FF{ static_cast<uint8_t>(result) },
        .cmp_op_is_gt = true,
    };
    // Update the row and add new rows with the correct hi_lo limbs
    alu_trace.push_back(row);
    return FF{ static_cast<int>(c) };
}

/**************************************************************************************************
 *                            COMPUTE - BITWISE
 **************************************************************************************************/

/**
 * @brief Build Alu trace and compute the result of a Bitwise Not of type defined by in_tag.
 *
 * @param a Unary operand of Not
 * @param in_tag Instruction tag defining the number of bits on which the addition applies.
 *               It is assumed that the caller never uses the type u0.
 * @param clk Clock referring to the operation in the main trace.
 *
 * @return FF The result of the not casted in a finite field element
 */
FF AvmAluTraceBuilder::op_not(FF const& a, AvmMemoryTag in_tag, uint32_t const clk)
{
    ASSERT(in_tag != AvmMemoryTag::FF);

    uint128_t a_u128{ a };
    uint128_t c_u128 = ~a_u128;

    FF c = cast_to_mem_tag(uint256_t::from_uint128(c_u128), in_tag);

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .opcode = OpCode::NOT,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ic = c,
    });

    return c;
}

/**
 * @brief Build Alu trace and compute the result of a SHL operation on two operands of type defined by in_tag.
 *
 * @param a Left operand of the SHL
 * @param b Right operand of the SHL
 * @param clk Clock referring to the operation in the main trace.
 * @param in_tag Instruction tag defining the number of bits for the SHL.
 *
 * @return FF The boolean result of SHL casted to a finite field element
 */
FF AvmAluTraceBuilder::op_shl(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk)
{
    ASSERT(in_tag != AvmMemoryTag::FF);
    // Check that the shifted amount is an 8-bit integer
    ASSERT(uint256_t(b) < 256);

    // Perform the shift operation over 256-bit integers
    uint256_t a_u256{ a };
    uint8_t b_u8 = uint8_t(b);
    uint256_t c_u256 = a_u256 << b_u8;
    FF c = cast_to_mem_tag(c_u256, in_tag);

    uint8_t num_bits = mem_tag_bits(in_tag);
    // We decompose the input into two limbs partitioned at the n - b bit
    auto [a_lo, a_hi] = decompose(a, num_bits - b_u8);

    // Check if this is a trivial shift - i.e. we shift more than the max bits of our input
    bool zero_shift = cmp_builder.constrained_gt(b, num_bits - 1, clk, EventEmitter::ALU);
    if (!zero_shift) {
        u8_pow_2_counters[0][b_u8]++;
        u8_pow_2_counters[1][num_bits - b_u8]++;
    }

    // Non Trivial shifts need to be range checked
    if (!zero_shift) {
        cmp_builder.range_check_builder.assert_range(
            uint128_t(a_lo), static_cast<uint8_t>(num_bits - b_u8), EventEmitter::ALU, clk);
    }
    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .opcode = OpCode::SHL,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_a_lo = a_lo,
        .alu_a_hi = a_hi,
        .mem_tag_bits = num_bits,
        .mem_tag_sub_shift = static_cast<uint8_t>(num_bits - b_u8),
        .zero_shift = zero_shift,
        .range_check_input = !zero_shift ? a_lo : 0,
        .range_check_num_bits = !zero_shift ? static_cast<uint8_t>(num_bits - b_u8) : 0,
        .range_check_sel = !zero_shift && in_tag != AvmMemoryTag::FF,
        .cmp_input_a = b,
        .cmp_input_b = FF{ static_cast<uint8_t>(num_bits - 1) },
        .cmp_result = FF{ static_cast<uint8_t>(zero_shift) },
        .cmp_op_is_gt = true,
    });

    return c;
}

/**
 * @brief Build Alu trace and compute the result of a SHR operation on two operands of type defined by in_tag.
 *
 * @param a Left operand of the SHR
 * @param b Right operand of the SHR
 * @param clk Clock referring to the operation in the main trace.
 * @param in_tag Instruction tag defining the number of bits for the SHR.
 *
 * @return FF The boolean result of SHR casted to a finite field element
 */
FF AvmAluTraceBuilder::op_shr(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk)
{
    ASSERT(in_tag != AvmMemoryTag::FF);
    // Check that the shifted amount is an 8-bit integer
    ASSERT(uint256_t(b) < 256);

    // Perform the shift operation over 256-bit integers
    uint256_t a_u256{ a };

    uint8_t b_u8 = static_cast<uint8_t>(uint256_t(b));
    uint256_t c_u256 = a_u256 >> b_u8;
    FF c = cast_to_mem_tag(c_u256, in_tag);

    uint8_t num_bits = mem_tag_bits(in_tag);
    bool zero_shift = cmp_builder.constrained_gt(b, num_bits - 1, clk, EventEmitter::ALU);
    if (!zero_shift) {
        // Add counters for the pow of two lookups
        u8_pow_2_counters[0][b_u8]++;
        u8_pow_2_counters[1][num_bits - b_u8]++;
    }

    // We decompose the input into two limbs partitioned at the b-th bit
    auto [a_lo, a_hi] = decompose(a, b_u8);

    if (!zero_shift) {
        cmp_builder.range_check_builder.assert_range(
            uint128_t(a_hi), static_cast<uint8_t>(num_bits - b_u8), EventEmitter::ALU, clk);
    }

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .opcode = OpCode::SHR,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_a_lo = a_lo,
        .alu_a_hi = a_hi,
        .mem_tag_bits = num_bits,
        .mem_tag_sub_shift = static_cast<uint8_t>(num_bits - b_u8),
        .zero_shift = zero_shift,
        .range_check_input = !zero_shift ? a_hi : 0,
        .range_check_num_bits = !zero_shift ? static_cast<uint8_t>(num_bits - b_u8) : 0,
        .range_check_sel = !zero_shift && in_tag != AvmMemoryTag::FF,
        .cmp_input_a = b,
        .cmp_input_b = FF{ static_cast<uint8_t>(num_bits - 1) },
        .cmp_result = FF{ static_cast<uint8_t>(zero_shift) },
        .cmp_op_is_gt = true,

    });
    return c_u256;
}

/**************************************************************************************************
 *                            COMPUTE - TYPE CONVERSIONS
 **************************************************************************************************/

/**
 * @brief Build ALU trace for the CAST opcode.
 *
 * @param a Input value to be casted. Tag of the input is not taken into account.
 * @param in_tag Tag specifying the type for the input to be casted into.
 * @param clk Clock referring to the operation in the main trace.
 * @return The casted value as a finite field element.
 */
FF AvmAluTraceBuilder::op_cast(FF const& a, AvmMemoryTag in_tag, uint32_t clk)
{
    FF c = cast_to_mem_tag(a, in_tag);

    uint8_t num_bits = mem_tag_bits(in_tag);

    // Get the decomposition of a
    auto [a_lo, a_hi] = decompose(uint256_t(a), num_bits);

    if (in_tag != AvmMemoryTag::FF) {
        cmp_builder.range_check_builder.assert_range(uint128_t(c), mem_tag_bits(in_tag), EventEmitter::ALU, clk);
    }
    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .opcode = OpCode::CAST,
        .tag = in_tag,
        .alu_ia = a,
        .alu_ic = c,
        .alu_a_lo = a_lo,
        .alu_a_hi = a_hi,
        .range_check_input = c,
        .range_check_num_bits = in_tag != AvmMemoryTag::FF ? mem_tag_bits(in_tag) : 0,
        .range_check_sel = in_tag != AvmMemoryTag::FF,
    });

    return c;
}

/**
 * @brief Helper routine telling whether range check is required.
 *
 * @return A boolean telling whether range check is required.
 */
bool AvmAluTraceBuilder::is_range_check_required() const
{
    return range_checked_required;
}

/**
 * @brief Helper function that returns a boolean if this entry is an alu operation.
 *        This is helpful to filter out range check rows or the second row in the 128-bit multiply.
 *
 * @return A boolean telling whether the ALU is enabled for the row.
 */
bool AvmAluTraceBuilder::is_alu_row_enabled(const AvmAluTraceBuilder::AluTraceEntry& r)
{
    return (r.opcode == OpCode::ADD || r.opcode == OpCode::SUB || r.opcode == OpCode::MUL || r.opcode == OpCode::EQ ||
            r.opcode == OpCode::NOT || r.opcode == OpCode::LT || r.opcode == OpCode::LTE || r.opcode == OpCode::SHR ||
            r.opcode == OpCode::SHL || r.opcode == OpCode::CAST || r.opcode == OpCode::DIV);
}

/**
 * @brief Incorporates the ALU trace in the main trace.
 */
void AvmAluTraceBuilder::finalize(std::vector<AvmFullRow<FF>>& main_trace)
{
    // This embeds the ALU information into the main trace.
    for (size_t i = 0; i < alu_trace.size(); i++) {
        auto const& src = alu_trace.at(i);
        auto& dest = main_trace.at(i);

        dest.alu_clk = FF(static_cast<uint32_t>(src.alu_clk));
        dest.alu_sel_alu = FF(1);

        if (src.opcode.has_value()) {
            dest.alu_op_add = FF(src.opcode == OpCode::ADD ? 1 : 0);
            dest.alu_op_sub = FF(src.opcode == OpCode::SUB ? 1 : 0);
            dest.alu_op_mul = FF(src.opcode == OpCode::MUL ? 1 : 0);
            dest.alu_op_not = FF(src.opcode == OpCode::NOT ? 1 : 0);
            dest.alu_op_eq = FF(src.opcode == OpCode::EQ ? 1 : 0);
            dest.alu_op_lt = FF(src.opcode == OpCode::LT ? 1 : 0);
            dest.alu_op_lte = FF(src.opcode == OpCode::LTE ? 1 : 0);
            dest.alu_op_cast = FF(src.opcode == OpCode::CAST ? 1 : 0);
            dest.alu_op_shr = FF(src.opcode == OpCode::SHR ? 1 : 0);
            dest.alu_op_shl = FF(src.opcode == OpCode::SHL ? 1 : 0);
            dest.alu_op_div = FF(src.opcode == OpCode::DIV ? 1 : 0);
        }

        if (src.tag.has_value()) {
            dest.alu_ff_tag = FF(src.tag == AvmMemoryTag::FF ? 1 : 0);
            dest.alu_u8_tag = FF(src.tag == AvmMemoryTag::U8 ? 1 : 0);
            dest.alu_u16_tag = FF(src.tag == AvmMemoryTag::U16 ? 1 : 0);
            dest.alu_u32_tag = FF(src.tag == AvmMemoryTag::U32 ? 1 : 0);
            dest.alu_u64_tag = FF(src.tag == AvmMemoryTag::U64 ? 1 : 0);
            dest.alu_u128_tag = FF(src.tag == AvmMemoryTag::U128 ? 1 : 0);
            dest.alu_in_tag = FF(static_cast<uint32_t>(src.tag.value()));
        }

        // General ALU fields
        dest.alu_ia = src.alu_ia;
        dest.alu_ib = src.alu_ib;
        dest.alu_ic = src.alu_ic;
        dest.alu_a_lo = src.alu_a_lo;
        dest.alu_a_hi = src.alu_a_hi;
        dest.alu_b_lo = src.alu_b_lo;
        dest.alu_b_hi = src.alu_b_hi;
        dest.alu_c_lo = src.alu_c_lo;
        dest.alu_c_hi = src.alu_c_hi;

        // Helpful Multiply and Divide fields
        dest.alu_partial_prod_lo = src.partial_prod_lo;
        dest.alu_partial_prod_hi = src.partial_prod_hi;

        // Additions and Subtraction specific field
        dest.alu_cf = FF(static_cast<uint32_t>(src.alu_cf));

        // Division specific fields
        dest.alu_remainder = src.remainder;

        // LT and LTE specific fields
        dest.alu_sel_cmp = dest.alu_op_lt + dest.alu_op_lte;

        // Shift specific fields
        dest.alu_zero_shift = FF(static_cast<uint8_t>(src.zero_shift));
        dest.alu_sel_shift_which = (dest.alu_op_shl + dest.alu_op_shr) * (FF::one() - dest.alu_zero_shift);
        dest.alu_max_bits_sub_b_bits = FF(src.mem_tag_sub_shift);
        dest.alu_b_pow = FF(uint256_t(1) << dest.alu_ib);
        dest.alu_max_bits_sub_b_pow = FF(uint256_t(1) << uint256_t(dest.alu_max_bits_sub_b_bits));

        // Range Check Fields
        dest.alu_range_check_sel = FF(static_cast<uint8_t>(src.range_check_sel));
        dest.alu_range_check_input_value = src.range_check_input;
        dest.alu_range_check_num_bits = src.range_check_num_bits;

        // Cmp Gadget Fields
        dest.alu_cmp_gadget_input_a = src.cmp_input_a;
        dest.alu_cmp_gadget_input_b = src.cmp_input_b;
        dest.alu_cmp_gadget_result = src.cmp_result;
        dest.alu_cmp_gadget_gt = FF(static_cast<uint8_t>(src.cmp_op_is_gt));
        dest.alu_cmp_gadget_sel = dest.alu_cmp_gadget_gt + dest.alu_op_eq;
    }
    reset();
}

} // namespace bb::avm_trace
