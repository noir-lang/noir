#include "avm_alu_trace.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "barretenberg/relations/generated/avm/avm_alu.hpp"
#include <cstdint>
#include <sys/types.h>
#include <tuple>
#include <utility>

namespace bb::avm_trace {

/**
 * @brief Constructor of Alu trace builder of AVM. Only serves to set the capacity of the
 *        underlying trace.
 */
AvmAluTraceBuilder::AvmAluTraceBuilder()
{
    alu_trace.reserve(AVM_TRACE_SIZE);
}

/**
 * @brief Resetting the internal state so that a new Alu trace can be rebuilt using the same object.
 *
 */
void AvmAluTraceBuilder::reset()
{
    alu_trace.clear();
}

/**
 * @brief Prepare the Alu trace to be incorporated into the main trace.
 *
 * @return The Alu trace (which is moved).
 */
std::vector<AvmAluTraceBuilder::AluTraceEntry> AvmAluTraceBuilder::finalize()
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
FF AvmAluTraceBuilder::op_add(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    FF c = 0;
    bool carry = false;
    uint8_t alu_u8_r0 = 0;
    uint8_t alu_u8_r1 = 0;
    std::array<uint16_t, 15> alu_u16_reg{}; // Must be zero-initialized (FF tag case)

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

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
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
FF AvmAluTraceBuilder::op_sub(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    FF c = 0;
    bool carry = false;
    uint8_t alu_u8_r0 = 0;
    uint8_t alu_u8_r1 = 0;
    std::array<uint16_t, 15> alu_u16_reg{}; // Must be zero-initialized (FF tag case)
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

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
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
FF AvmAluTraceBuilder::op_mul(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t const clk)
{
    FF c = 0;
    bool carry = false;
    uint8_t alu_u8_r0 = 0;
    uint8_t alu_u8_r1 = 0;

    std::array<uint16_t, 15> alu_u16_reg{}; // Must be zero-initialized (FF tag case)

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
        std::array<uint16_t, 15> alu_u16_reg_a; // Will be initialized in for loop below.
        std::array<uint16_t, 15> alu_u16_reg_b; // Will be initialized in for loop below.
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

        // From PIL relation in avm_alu.pil, we need to determine the bit CF and 64-bit value R' in
        // a * b_l + a_l * b_h * 2^64 = (CF * 2^64 + R') * 2^128 + c
        // LHS is c_u256 - c_high

        // CF bit
        carry = ((c_u256 - c_high) >> 192) > 0;
        // R' value
        uint64_t alu_u64_r0 = static_cast<uint64_t>(((c_u256 - c_high) >> 128) & uint256_t(UINT64_MAX));

        c = FF{ uint256_t::from_uint128(c_u128) };

        alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
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

        alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
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
    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
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
    FF c = 0;
    uint128_t a_u128{ a };
    uint128_t c_u128 = ~a_u128;

    switch (in_tag) {
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
    case AvmMemoryTag::FF: // Unsupported as instruction tag {}
    case AvmMemoryTag::U0: // Unsupported as instruction tag {}
        return FF{ 0 };
    }

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .alu_op_not = true,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ic = c,
    });

    return c;
}

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
    FF c = a - b;
    // Don't invert 0 as it will throw
    FF inv_c = c != FF::zero() ? c.invert() : FF::zero();
    FF res = c == FF::zero() ? FF::one() : FF::zero();

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .alu_op_eq = true,
        .alu_ff_tag = in_tag == AvmMemoryTag::FF,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = res,
        .alu_op_eq_diff_inv = inv_c,
    });

    return res;
}

/**
 * @brief This is a helper function that decomposes the input into the various registers of the ALU.
 *        This additionally increments the counts for the corresponding range lookups entries.
 * @return A triplet of <alu_u8_r0, alu_u8_r1, alu_u16_reg>
 */
template <typename T>
std::tuple<uint8_t, uint8_t, std::vector<uint16_t>> AvmAluTraceBuilder::to_alu_slice_registers(T a)
{
    auto alu_u8_r0 = static_cast<uint8_t>(a);
    a >>= 8;
    auto alu_u8_r1 = static_cast<uint8_t>(a);
    a >>= 8;
    std::vector<uint16_t> alu_u16_reg{};
    for (size_t i = 0; i < 15; i++) {
        auto alu_u16 = static_cast<uint16_t>(a);
        u16_range_chk_counters[i][alu_u16]++;
        alu_u16_reg.push_back(alu_u16);
        a >>= 16;
    }
    u8_range_chk_counters[0][alu_u8_r0]++;
    u8_range_chk_counters[1][alu_u8_r1]++;
    return std::make_tuple(alu_u8_r0, alu_u8_r1, alu_u16_reg);
}

/**
 * @brief This is a helper function that is used to generate the range check entries for the comparison operation
 * (LT/LTE opcodes). This additionally increments the counts for the corresponding range lookups entries.
 * @param row The initial row where the comparison operation was performed
 * @param hi_lo_limbs The vector of 128-bit limbs hi and lo pairs of limbs that will be range checked.
 * @return A vector of AluTraceEntry rows for the range checks for the comparison operation.
 */
std::vector<AvmAluTraceBuilder::AluTraceEntry> AvmAluTraceBuilder::cmp_range_check_helper(
    AvmAluTraceBuilder::AluTraceEntry row, std::vector<uint256_t> hi_lo_limbs)
{
    // Assume each limb is 128 bits and since we can perform 256-bit range check per rows
    // we need to have (limbs.size() / 2) range checks rows
    size_t num_rows = hi_lo_limbs.size() / 2;
    // The first row is the original comparison instruction (LT/LTE)
    std::vector<AvmAluTraceBuilder::AluTraceEntry> rows{ std::move(row) };
    rows.resize(num_rows, {});

    // We need to ensure that the number of rows is even
    ASSERT(hi_lo_limbs.size() % 2 == 0);
    // Now for each row, we need to unpack a pair from the hi_lo_limb array into the ALUs 8-bit and 16-bit registers
    // The first row unpacks a_lo and a_hi, the second row unpacks b_lo and b_hi, and so on.
    for (size_t j = 0; j < num_rows; j++) {
        auto& r = rows.at(j);
        uint256_t lo_limb = hi_lo_limbs.at(2 * j);
        uint256_t hi_limb = hi_lo_limbs.at(2 * j + 1);
        uint256_t limb = lo_limb + (hi_limb << 128);
        // Unpack lo limb and handle in the 8-bit registers
        auto [alu_u8_r0, alu_u8_r1, alu_u16_reg] = AvmAluTraceBuilder::to_alu_slice_registers(limb);
        r.alu_u8_r0 = alu_u8_r0;
        r.alu_u8_r1 = alu_u8_r1;
        std::copy(alu_u16_reg.begin(), alu_u16_reg.end(), r.alu_u16_reg.begin());

        r.cmp_rng_ctr = j > 0 ? static_cast<uint8_t>(num_rows - j) : 0;
        r.rng_chk_sel = j > 0;
        r.alu_op_eq_diff_inv = j > 0 ? FF(num_rows - j).invert() : 0;

        std::vector<FF> limb_arr = { hi_lo_limbs.begin() + static_cast<int>(2 * j), hi_lo_limbs.end() };
        // Resizing here is probably suboptimal for performance, we can probably handle the shorter vectors and
        // pad with zero during the finalise
        limb_arr.resize(10, FF::zero());
        r.hi_lo_limbs = limb_arr;
    }
    return rows;
}

/**
 * Helper function to decompose a uint256_t into upper 128-bit and lower 128-bit tuple.
 * The outputs are cast to uint256_t so they are easier to use in checks
 */

std::tuple<uint256_t, uint256_t> decompose(uint256_t const& a)
{
    uint256_t upper_bitmask = (uint256_t(1) << uint256_t(128)) - 1;
    uint256_t a_lo = a & upper_bitmask;
    uint256_t a_hi = a >> 128;
    return std::make_tuple(a_lo, a_hi);
}

// This creates a witness exclusively for the relation a > b
// This is useful when we want to enforce in certain checks that a must be greater than b
std::tuple<uint256_t, uint256_t, bool> gt_witness(uint256_t const& a, uint256_t const& b)
{
    uint256_t two_pow_128 = uint256_t(1) << uint256_t(128);
    auto [a_lo, a_hi] = decompose(a);
    auto [b_lo, b_hi] = decompose(b);
    bool borrow = a_lo <= b_lo;
    auto borrow_u256 = uint256_t(static_cast<uint64_t>(borrow));
    uint256_t r_lo = a_lo - b_lo - 1 + borrow_u256 * two_pow_128;
    uint256_t r_hi = a_hi - b_hi - borrow_u256;
    return std::make_tuple(r_lo, r_hi, borrow);
}

// This check is more flexible than gt_witness and is used when we want to generate the witness
// to the relation (a - b - 1) * q + (b - a) * (1 - q)
// where q = 1 if a > b and q = 0 if a <= b
std::tuple<uint256_t, uint256_t, bool> gt_or_lte_witness(uint256_t const& a, uint256_t const& b)
{
    uint256_t two_pow_128 = uint256_t(1) << uint256_t(128);
    auto [a_lo, a_hi] = decompose(a);
    auto [b_lo, b_hi] = decompose(b);
    bool isGT = a > b;
    if (isGT) {
        return gt_witness(a, b);
    }
    bool borrow = b_lo < a_lo;
    auto borrow_u256 = uint256_t(static_cast<uint64_t>(borrow));
    uint256_t r_lo = b_lo - a_lo + borrow_u256 * two_pow_128;
    uint256_t r_hi = b_hi - a_hi - borrow_u256;
    return std::make_tuple(r_lo, r_hi, borrow);
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
    bool c = uint256_t(a) < uint256_t(b);

    // Note: This is counter-intuitive, to show that a < b we actually show that b > a
    // The subtlety is here that the circuit is designed as a GT(x,y) circuit, therefore we swap the inputs a & b
    // Get the decomposition of b
    auto [a_lo, a_hi] = decompose(b);
    // Get the decomposition of a
    auto [b_lo, b_hi] = decompose(a);
    // Get the decomposition of p - a and p - b **remember that we swap the inputs**
    // Note that a valid witness here is ONLY that p > a and p > b
    auto [p_sub_a_lo, p_sub_a_hi, p_a_borrow] = gt_witness(FF::modulus, b);
    auto [p_sub_b_lo, p_sub_b_hi, p_b_borrow] = gt_witness(FF::modulus, a);
    // We either generate a witness that a <= b or a > b (its validity depends on the value of c)
    auto [r_lo, r_hi, borrow] = gt_or_lte_witness(b, a);

    // The vector of limbs that are used in the GT circuit and that are range checked
    std::vector<uint256_t> hi_lo_limbs = { a_lo,       a_hi,       b_lo,       b_hi, p_sub_a_lo,
                                           p_sub_a_hi, p_sub_b_lo, p_sub_b_hi, r_lo, r_hi };

    AvmAluTraceBuilder::AluTraceEntry row{
        .alu_clk = clk,
        .alu_op_lt = true,
        .alu_ff_tag = in_tag == AvmMemoryTag::FF,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = FF(static_cast<uint8_t>(c)),
        .borrow = borrow,
        .p_a_borrow = p_a_borrow,
        .p_b_borrow = p_b_borrow,
    };
    // Update the row and add new rows with the correct hi_lo limbs
    std::vector<AvmAluTraceBuilder::AluTraceEntry> rows = cmp_range_check_helper(row, hi_lo_limbs);
    // Append the rows to the alu_trace
    alu_trace.insert(alu_trace.end(), rows.begin(), rows.end());
    return { static_cast<int>(c) };
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
    bool c = uint256_t(a) <= uint256_t(b);

    // Get the decomposition of a
    auto [a_lo, a_hi] = decompose(a);
    // Get the decomposition of b
    auto [b_lo, b_hi] = decompose(b);
    // Get the decomposition of p - a and p - b
    // Note that a valid witness here is that p > a and p > b
    auto [p_sub_a_lo, p_sub_a_hi, p_a_borrow] = gt_witness(FF::modulus, a);
    auto [p_sub_b_lo, p_sub_b_hi, p_b_borrow] = gt_witness(FF::modulus, b);
    // We either generate a witness that a <= b or a > b (its validity depends on the value of c)
    auto [r_lo, r_hi, borrow] = gt_or_lte_witness(a, b);

    // The vector of limbs that are used in the GT circuit and that are range checked
    std::vector<uint256_t> hi_lo_limbs = { a_lo,       a_hi,       b_lo,       b_hi, p_sub_a_lo,
                                           p_sub_a_hi, p_sub_b_lo, p_sub_b_hi, r_lo, r_hi };

    // Construct the row that performs the lte check
    AvmAluTraceBuilder::AluTraceEntry row{
        .alu_clk = clk,
        .alu_op_lte = true,
        .alu_ff_tag = in_tag == AvmMemoryTag::FF,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = FF(static_cast<uint8_t>(c)),
        .borrow = borrow,
        .p_a_borrow = p_a_borrow,
        .p_b_borrow = p_b_borrow,
    };
    // Update the row and add new rows with the correct hi_lo limbs
    std::vector<AvmAluTraceBuilder::AluTraceEntry> rows = cmp_range_check_helper(row, hi_lo_limbs);
    alu_trace.insert(alu_trace.end(), rows.begin(), rows.end());
    return { static_cast<int>(c) };
}
} // namespace bb::avm_trace
