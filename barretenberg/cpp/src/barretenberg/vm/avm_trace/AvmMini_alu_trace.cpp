#include "AvmMini_alu_trace.hpp"

namespace avm_trace {

/**
 * @brief Constructor of Alu trace builder of AVM. Only serves to set the capacity of the
 *        underlying trace.
 */
AvmMiniAluTraceBuilder::AvmMiniAluTraceBuilder()
{
    alu_trace.reserve(AVM_TRACE_SIZE);
}

/**
 * @brief Resetting the internal state so that a new Alu trace can be rebuilt using the same object.
 *
 */
void AvmMiniAluTraceBuilder::reset()
{
    alu_trace.clear();
}

/**
 * @brief Prepare the Alu trace to be incorporated into the main trace.
 *
 * @return The Alu trace (which is moved).
 */
std::vector<AvmMiniAluTraceBuilder::AluTraceEntry> AvmMiniAluTraceBuilder::finalize()
{
    return std::move(alu_trace);
}

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
FF AvmMiniAluTraceBuilder::add(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    FF c{};
    bool carry = false;
    uint8_t alu_u8_r0{};
    uint8_t alu_u8_r1{};
    std::array<uint16_t, 8> alu_u16_reg{};

    uint128_t a_u128{ a };
    uint128_t b_u128{ b };
    uint128_t c_u128 = a_u128 + b_u128;

    switch (in_tag) {
    case AvmMemoryTag::FF:
        c = a + b;
        break;
    case AvmMemoryTag::U8:
        c = FF{ static_cast<uint8_t>(c_u128) };
        break;
    case AvmMemoryTag::U16:
        c = FF{ static_cast<uint16_t>(c_u128) };
        break;
    case AvmMemoryTag::U32:
        c = FF{ static_cast<uint32_t>(c_u128) };
        break;
    case AvmMemoryTag::U64:
        c = FF{ static_cast<uint64_t>(c_u128) };
        break;
    case AvmMemoryTag::U128:
        c = FF{ uint256_t::from_uint128(c_u128) };
        break;
    case AvmMemoryTag::U0: // Unsupported as instruction tag
        return FF{ 0 };
    }

    if (in_tag != AvmMemoryTag::FF) {
        // a_u128 + b_u128 >= 2^128  <==> c_u128 < a_u128
        if (c_u128 < a_u128) {
            carry = true;
        }

        uint128_t c_trunc_128 = c_u128;
        alu_u8_r0 = static_cast<uint8_t>(c_trunc_128);
        c_trunc_128 >>= 8;
        alu_u8_r1 = static_cast<uint8_t>(c_trunc_128);
        c_trunc_128 >>= 8;

        for (size_t i = 0; i < 7; i++) {
            alu_u16_reg.at(i) = static_cast<uint16_t>(c_trunc_128);
            c_trunc_128 >>= 16;
        }
    }

    alu_trace.push_back(AvmMiniAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .alu_op_add = true,
        .alu_ff_tag = in_tag == AvmMemoryTag::FF,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_cf = carry,
        .alu_u8_r0 = alu_u8_r0,
        .alu_u8_r1 = alu_u8_r1,
        .alu_u16_reg = alu_u16_reg,
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
FF AvmMiniAluTraceBuilder::sub(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    FF c{};
    bool carry = false;
    uint8_t alu_u8_r0{};
    uint8_t alu_u8_r1{};
    std::array<uint16_t, 8> alu_u16_reg{};
    uint128_t a_u128{ a };
    uint128_t b_u128{ b };
    uint128_t c_u128 = a_u128 - b_u128;

    switch (in_tag) {
    case AvmMemoryTag::FF:
        c = a - b;
        break;
    case AvmMemoryTag::U8:
        c = FF{ static_cast<uint8_t>(c_u128) };
        break;
    case AvmMemoryTag::U16:
        c = FF{ static_cast<uint16_t>(c_u128) };
        break;
    case AvmMemoryTag::U32:
        c = FF{ static_cast<uint32_t>(c_u128) };
        break;
    case AvmMemoryTag::U64:
        c = FF{ static_cast<uint64_t>(c_u128) };
        break;
    case AvmMemoryTag::U128:
        c = FF{ uint256_t::from_uint128(c_u128) };
        break;
    case AvmMemoryTag::U0: // Unsupported as instruction tag
        return FF{ 0 };
    }

    if (in_tag != AvmMemoryTag::FF) {
        // Underflow when a_u128 < b_u128
        if (a_u128 < b_u128) {
            carry = true;
        }

        uint128_t c_trunc_128 = c_u128;
        alu_u8_r0 = static_cast<uint8_t>(c_trunc_128);
        c_trunc_128 >>= 8;
        alu_u8_r1 = static_cast<uint8_t>(c_trunc_128);
        c_trunc_128 >>= 8;

        for (size_t i = 0; i < 7; i++) {
            alu_u16_reg.at(i) = static_cast<uint16_t>(c_trunc_128);
            c_trunc_128 >>= 16;
        }
    }

    alu_trace.push_back(AvmMiniAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .alu_op_sub = true,
        .alu_ff_tag = in_tag == AvmMemoryTag::FF,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_cf = carry,
        .alu_u8_r0 = alu_u8_r0,
        .alu_u8_r1 = alu_u8_r1,
        .alu_u16_reg = alu_u16_reg,
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
FF AvmMiniAluTraceBuilder::mul(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    FF c{};
    bool carry = false;
    uint8_t alu_u8_r0{};
    uint8_t alu_u8_r1{};

    std::array<uint16_t, 8> alu_u16_reg{};

    uint128_t a_u128{ a };
    uint128_t b_u128{ b };
    uint128_t c_u128 = a_u128 * b_u128; // Multiplication over the integers (not mod. 2^64)

    switch (in_tag) {
    case AvmMemoryTag::FF:
        c = a * b;
        break;
    case AvmMemoryTag::U8:
        c = FF{ static_cast<uint8_t>(c_u128) };
        break;
    case AvmMemoryTag::U16:
        c = FF{ static_cast<uint16_t>(c_u128) };
        break;
    case AvmMemoryTag::U32:
        c = FF{ static_cast<uint32_t>(c_u128) };
        break;
    case AvmMemoryTag::U64:
        c = FF{ static_cast<uint64_t>(c_u128) };
        break;
    case AvmMemoryTag::U128: {
        uint256_t a_u256{ a };
        uint256_t b_u256{ b };
        uint256_t c_u256 = a_u256 * b_u256; // Multiplication over the integers (not mod. 2^128)

        uint128_t a_u128{ a_u256 };
        uint128_t b_u128{ b_u256 };

        uint128_t c_u128 = a_u128 * b_u128;

        // Decompose a_u128 and b_u128 over 8 16-bit registers.
        std::array<uint16_t, 8> alu_u16_reg_a{};
        std::array<uint16_t, 8> alu_u16_reg_b{};
        uint128_t a_trunc_128 = a_u128;
        uint128_t b_trunc_128 = b_u128;

        for (size_t i = 0; i < 8; i++) {
            alu_u16_reg_a.at(i) = static_cast<uint16_t>(a_trunc_128);
            alu_u16_reg_b.at(i) = static_cast<uint16_t>(b_trunc_128);
            a_trunc_128 >>= 16;
            b_trunc_128 >>= 16;
        }

        // Represent a, b with 64-bit limbs: a = a_l + 2^64 * a_h, b = b_l + 2^64 * b_h,
        // c_high := 2^128 * a_h * b_h
        uint256_t c_high = ((a_u256 >> 64) * (b_u256 >> 64)) << 128;

        // From PIL relation in alu_chip.pil, we need to determine the bit CF and 64-bit value R' in
        // a * b_l + a_l * b_h * 2^64 = (CF * 2^64 + R') * 2^128 + c
        // LHS is c_u256 - c_high

        // CF bit
        carry = ((c_u256 - c_high) >> 192) > 0;
        // R' value
        uint64_t alu_u64_r0 = static_cast<uint64_t>(((c_u256 - c_high) >> 128) & uint256_t(UINT64_MAX));

        c = FF{ uint256_t::from_uint128(c_u128) };

        alu_trace.push_back(AvmMiniAluTraceBuilder::AluTraceEntry{
            .alu_clk = clk,
            .alu_op_mul = true,
            .alu_u128_tag = in_tag == AvmMemoryTag::U128,
            .alu_ia = a,
            .alu_ib = b,
            .alu_ic = c,
            .alu_cf = carry,
            .alu_u16_reg = alu_u16_reg_a,
            .alu_u64_r0 = alu_u64_r0,
        });

        alu_trace.push_back(AvmMiniAluTraceBuilder::AluTraceEntry{
            .alu_u16_reg = alu_u16_reg_b,
        });

        return c;
    }
    case AvmMemoryTag::U0: // Unsupported as instruction tag
        return FF{ 0 };
    }

    // Following code executed for: u8, u16, u32, u64 (u128 returned handled specifically)
    if (in_tag != AvmMemoryTag::FF) {
        // Decomposition of c_u128 into 8-bit and 16-bit registers as follows:
        // alu_u8_r0 + alu_u8_r1 * 2^8 + alu_u16_r0 * 2^16 ... +  alu_u16_r6 * 2^112
        uint128_t c_trunc_128 = c_u128;
        alu_u8_r0 = static_cast<uint8_t>(c_trunc_128);
        c_trunc_128 >>= 8;
        alu_u8_r1 = static_cast<uint8_t>(c_trunc_128);
        c_trunc_128 >>= 8;

        for (size_t i = 0; i < 7; i++) {
            alu_u16_reg.at(i) = static_cast<uint16_t>(c_trunc_128);
            c_trunc_128 >>= 16;
        }
    }

    // Following code executed for: ff, u8, u16, u32, u64 (u128 returned handled specifically)
    alu_trace.push_back(AvmMiniAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .alu_op_mul = true,
        .alu_ff_tag = in_tag == AvmMemoryTag::FF,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_cf = carry,
        .alu_u8_r0 = alu_u8_r0,
        .alu_u8_r1 = alu_u8_r1,
        .alu_u16_reg = alu_u16_reg,
    });

    return c;
}

} // namespace avm_trace
