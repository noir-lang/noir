#include "barretenberg/vm/avm_trace/avm_alu_trace.hpp"

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

// This creates a witness exclusively for the relation a > b
// This is useful when we want to enforce in certain checks that a must be greater than b
std::tuple<uint256_t, uint256_t, bool> gt_witness(uint256_t const& a, uint256_t const& b)
{
    uint256_t two_pow_128 = uint256_t(1) << uint256_t(128);
    auto [a_lo, a_hi] = decompose(a, 128);
    auto [b_lo, b_hi] = decompose(b, 128);
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
    uint256_t two_pow_126 = uint256_t(1) << uint256_t(128);
    auto [a_lo, a_hi] = decompose(a, 128);
    auto [b_lo, b_hi] = decompose(b, 128);
    bool isGT = a > b;
    if (isGT) {
        return gt_witness(a, b);
    }
    bool borrow = b_lo < a_lo;
    auto borrow_u256 = uint256_t(static_cast<uint64_t>(borrow));
    uint256_t r_lo = b_lo - a_lo + borrow_u256 * two_pow_126;
    uint256_t r_hi = b_hi - a_hi - borrow_u256;
    return std::make_tuple(r_lo, r_hi, borrow);
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
} // Anonymous namespace

/**************************************************************************************************
 *                                 PRIVATE HELPERS
 **************************************************************************************************/

/**
 * @brief This is a helper function that decomposes the input into the various registers of the ALU.
 *        This additionally increments the counts for the corresponding range lookups entries.
 * @return A triplet of <alu_u8_r0, alu_u8_r1, alu_u16_reg>
 */
template <typename T>
std::tuple<uint8_t, uint8_t, std::array<uint16_t, 15>> AvmAluTraceBuilder::to_alu_slice_registers(T a)
{
    range_checked_required = true;
    auto alu_u8_r0 = static_cast<uint8_t>(a);
    a >>= 8;
    auto alu_u8_r1 = static_cast<uint8_t>(a);
    a >>= 8;
    std::array<uint16_t, 15> alu_u16_reg;
    for (size_t i = 0; i < 15; i++) {
        auto alu_u16 = static_cast<uint16_t>(a);
        u16_range_chk_counters[i][alu_u16]++;
        alu_u16_reg.at(i) = alu_u16;
        a >>= 16;
    }
    u8_range_chk_counters[0][alu_u8_r0]++;
    u8_range_chk_counters[1][alu_u8_r1]++;
    return std::make_tuple(alu_u8_r0, alu_u8_r1, alu_u16_reg);
}

/**
 * @brief This is a helper function that is used to generate the range check entries for operations that require
 * multi-row range checks This additionally increments the counts for the corresponding range lookups entries.
 * @param row The initial row where the comparison operation was performed
 * @param hi_lo_limbs The vector of 128-bit limbs hi and lo pairs of limbs that will be range checked.
 * @return A vector of AluTraceEntry rows for the range checks for the operation.
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
        r.alu_u16_reg = alu_u16_reg;

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

/**
 * @brief Prepare the Alu trace to be incorporated into the main trace.
 *
 * @return The Alu trace (which is moved).
 */
std::vector<AvmAluTraceBuilder::AluTraceEntry> AvmAluTraceBuilder::finalize()
{
    return std::move(alu_trace);
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
    }

    // The range checks are activated for all tags and therefore we need to call the slice register
    // routines also for tag FF with input 0.
    auto [u8_r0, u8_r1, u16_reg] = to_alu_slice_registers(in_tag == AvmMemoryTag::FF ? 0 : c_u128);
    alu_u8_r0 = u8_r0;
    alu_u8_r1 = u8_r1;
    alu_u16_reg = u16_reg;

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
    }

    // The range checks are activated for all tags and therefore we need to call the slice register
    // routines also for tag FF with input 0.
    auto [u8_r0, u8_r1, u16_reg] = to_alu_slice_registers(in_tag == AvmMemoryTag::FF ? 0 : c_u128);
    alu_u8_r0 = u8_r0;
    alu_u8_r1 = u8_r1;
    alu_u16_reg = u16_reg;

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

        // Decompose a_u128 and b_u128 over 8/16-bit registers.
        auto [a_u8_r0, a_u8_r1, a_u16_reg] = to_alu_slice_registers(a_u128);
        auto [b_u8_r0, b_u8_r1, b_u16_reg] = to_alu_slice_registers(b_u128);

        // Represent a, b with 64-bit limbs: a = a_l + 2^64 * a_h, b = b_l + 2^64 * b_h,
        // c_high := 2^128 * a_h * b_h
        uint256_t c_high = ((a_u256 >> 64) * (b_u256 >> 64)) << 128;

        // From PIL relation in avm_alu.pil, we need to determine the bit CF and 64-bit value R_64 in
        // a * b_l + a_l * b_h * 2^64 = (CF * 2^64 + R_64) * 2^128 + c
        // LHS is c_u256 - c_high

        // CF bit
        carry = ((c_u256 - c_high) >> 192) > 0;
        // R_64 value
        uint64_t r_64 = static_cast<uint64_t>(((c_u256 - c_high) >> 128) & uint256_t(UINT64_MAX));

        // Decompose R_64 over 16-bit registers u16_r7, u16_r8, u16_r9, u_16_r10
        for (size_t i = 0; i < 4; i++) {
            auto const slice = static_cast<uint16_t>(r_64);
            assert(a_u16_reg.at(7 + i) == 0);
            u16_range_chk_counters[7 + i][0]--;
            a_u16_reg.at(7 + i) = slice;
            u16_range_chk_counters[7 + i][slice]++;
            r_64 >>= 16;
        }

        c = FF{ uint256_t::from_uint128(c_u128) };

        alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
            .alu_clk = clk,
            .alu_op_mul = true,
            .alu_u128_tag = in_tag == AvmMemoryTag::U128,
            .alu_ia = a,
            .alu_ib = b,
            .alu_ic = c,
            .alu_cf = carry,
            .alu_u8_r0 = a_u8_r0,
            .alu_u8_r1 = a_u8_r1,
            .alu_u16_reg = a_u16_reg,
        });

        alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
            .alu_u8_r0 = b_u8_r0,
            .alu_u8_r1 = b_u8_r1,
            .alu_u16_reg = b_u16_reg,
        });

        return c;
    }
    case AvmMemoryTag::U0: // Unsupported as instruction tag
        return FF{ 0 };
    }

    // Following code executed for: u8, u16, u32, u64 (u128 returned handled specifically).
    // The range checks are activated for all tags and therefore we need to call the slice register
    // routines also for tag FF with input 0.
    auto [u8_r0, u8_r1, u16_reg] = to_alu_slice_registers(in_tag == AvmMemoryTag::FF ? 0 : c_u128);
    alu_u8_r0 = u8_r0;
    alu_u8_r1 = u8_r1;
    alu_u16_reg = u16_reg;

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

FF AvmAluTraceBuilder::op_div(FF const& a, FF const& b, AvmMemoryTag in_tag, uint32_t clk)
{
    uint256_t a_u256{ a };
    uint256_t b_u256{ b };
    uint256_t c_u256 = a_u256 / b_u256;
    uint256_t rem_u256 = a_u256 % b_u256;

    // If dividing by zero, don't add any rows in the ALU, the error will be handled in the main trace
    if (b_u256 == 0) {
        return 0;
    }

    if (a_u256 < b_u256) {
        // If a < b, the result is trivially 0
        uint256_t rng_chk_lo = b_u256 - a_u256 - 1;
        auto [u8_r0, u8_r1, u16_reg] = to_alu_slice_registers(rng_chk_lo);
        alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry({
            .alu_clk = clk,
            .alu_op_div = true,
            .alu_u8_tag = in_tag == AvmMemoryTag::U8,
            .alu_u16_tag = in_tag == AvmMemoryTag::U16,
            .alu_u32_tag = in_tag == AvmMemoryTag::U32,
            .alu_u64_tag = in_tag == AvmMemoryTag::U64,
            .alu_u128_tag = in_tag == AvmMemoryTag::U128,
            .alu_ia = a,
            .alu_ib = b,
            .alu_ic = 0,
            .alu_u8_r0 = u8_r0,
            .alu_u8_r1 = u8_r1,
            .alu_u16_reg = u16_reg,
            .hi_lo_limbs = { rng_chk_lo, 0, 0, 0, 0, 0 },
            .remainder = a,

        }));
        return 0;
    }
    // Decompose a and primality check that b*c < p when a is a 256-bit integer
    auto [a_lo, a_hi] = decompose(b_u256 * c_u256, 128);
    auto [p_sub_a_lo, p_sub_a_hi, p_a_borrow] = gt_witness(FF::modulus, b_u256 * c_u256);
    // Decompose the divisor
    auto [divisor_lo, divisor_hi] = decompose(b_u256, 64);
    // Decompose the quotient
    auto [quotient_lo, quotient_hi] = decompose(c_u256, 64);
    uint256_t partial_prod = divisor_lo * quotient_hi + divisor_hi * quotient_lo;
    // Decompose the partial product
    auto [partial_prod_lo, partial_prod_hi] = decompose(partial_prod, 64);

    FF b_hi = b_u256 - rem_u256 - 1;

    // 64 bit range checks for the divisor and quotient limbs
    // Spread over two rows
    std::array<uint16_t, 8> div_u64_rng_chk;
    std::array<uint16_t, 8> div_u64_rng_chk_shifted;
    for (size_t i = 0; i < 4; i++) {
        div_u64_rng_chk.at(i) = uint16_t(divisor_lo >> (16 * i));
        div_u64_rng_chk.at(i + 4) = uint16_t(divisor_hi >> (16 * i));
        div_u64_range_chk_counters[i][uint16_t(divisor_lo >> (16 * i))]++;
        div_u64_range_chk_counters[i + 4][uint16_t(divisor_hi >> (16 * i))]++;

        div_u64_rng_chk_shifted.at(i) = uint16_t(quotient_lo >> (16 * i));
        div_u64_rng_chk_shifted.at(i + 4) = uint16_t(quotient_hi >> (16 * i));
        div_u64_range_chk_counters[i][uint16_t(quotient_lo >> (16 * i))]++;
        div_u64_range_chk_counters[i + 4][uint16_t(quotient_hi >> (16 * i))]++;
    }

    // Each hi and lo limb is range checked over 128 bits
    // Load the range check values into the ALU registers
    auto hi_lo_limbs = std::vector<uint256_t>{ a_lo, a_hi, partial_prod, b_hi, p_sub_a_lo, p_sub_a_hi };
    AvmAluTraceBuilder::AluTraceEntry row{
        .alu_clk = clk,
        .alu_op_div = true,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = FF{ c_u256 },
        .remainder = rem_u256,
        .divisor_lo = divisor_lo,
        .divisor_hi = divisor_hi,
        .quotient_lo = quotient_lo,
        .quotient_hi = quotient_hi,
        .partial_prod_lo = partial_prod_lo,
        .partial_prod_hi = partial_prod_hi,
        .div_u64_range_chk_sel = true,
        .div_u64_range_chk = div_u64_rng_chk,

    };
    // We perform the range checks here
    std::vector<AvmAluTraceBuilder::AluTraceEntry> rows = cmp_range_check_helper(row, hi_lo_limbs);
    // Add the range checks for the quotient limbs in the row after the division operation
    rows.at(1).div_u64_range_chk = div_u64_rng_chk_shifted;
    rows.at(1).div_u64_range_chk_sel = true;
    alu_trace.insert(alu_trace.end(), rows.begin(), rows.end());
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
    auto [a_lo, a_hi] = decompose(b, 128);
    // Get the decomposition of a
    auto [b_lo, b_hi] = decompose(a, 128);
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
    bool c = uint256_t(a) <= uint256_t(b);

    // Get the decomposition of a
    auto [a_lo, a_hi] = decompose(a, 128);
    // Get the decomposition of b
    auto [b_lo, b_hi] = decompose(b, 128);
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
    // Perform the shift operation over 256-bit integers
    uint256_t a_u256{ a };
    // Check that the shift amount is an 8-bit integer
    ASSERT(uint256_t(b) < 256);
    ASSERT(in_tag != AvmMemoryTag::U0 || in_tag != AvmMemoryTag::FF);

    uint8_t b_u8 = static_cast<uint8_t>(uint256_t(b));

    uint256_t c_u256 = a_u256 << b_u8;

    uint8_t num_bits = mem_tag_bits(in_tag);
    u8_pow_2_counters[0][b_u8]++;
    // If we are shifting more than the number of bits, the result is trivially 0
    if (b_u8 >= num_bits) {
        u8_pow_2_counters[1][b_u8 - num_bits]++;
        // Even though the registers are trivially zero, we call this function to increment the lookup counters
        // Future workaround would be to decouple the range_check toggle and the counter from this function
        [[maybe_unused]] auto [alu_u8_r0, alu_u8_r1, alu_u16_reg] = AvmAluTraceBuilder::to_alu_slice_registers(0);
        alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
            .alu_clk = clk,
            .alu_op_shl = true,
            .alu_ff_tag = in_tag == AvmMemoryTag::FF,
            .alu_u8_tag = in_tag == AvmMemoryTag::U8,
            .alu_u16_tag = in_tag == AvmMemoryTag::U16,
            .alu_u32_tag = in_tag == AvmMemoryTag::U32,
            .alu_u64_tag = in_tag == AvmMemoryTag::U64,
            .alu_u128_tag = in_tag == AvmMemoryTag::U128,
            .alu_ia = a,
            .alu_ib = b,
            .alu_ic = 0,
            .hi_lo_limbs = { 0, 0, 0, 0 },
            .mem_tag_bits = num_bits,
            .mem_tag_sub_shift = static_cast<uint8_t>(b_u8 - num_bits),
            .shift_lt_bit_len = false,
        });
        return 0;
    }
    // We decompose the input into two limbs partitioned at the b-th bit, we use x_lo and x_hi
    // to avoid any confusion with the a_lo and a_hi that form part of the range check
    auto [x_lo, x_hi] = decompose(a, num_bits - b_u8);

    u8_pow_2_counters[1][num_bits - b_u8]++;
    // We can modify the dynamic range check by performing an additional static one
    // rng_chk_lo = 2^(num_bits - b) - x_lo - 1 && rng_chk_hi = 2^b - x_hi - 1
    uint256_t rng_chk_lo = uint256_t(uint256_t(1) << (num_bits - b_u8)) - x_lo - 1;
    uint256_t rng_chk_hi = uint256_t(uint256_t(1) << b_u8) - x_hi - 1;

    // Each hi and lo limb is range checked over 128 bits
    uint256_t limb = rng_chk_lo + (rng_chk_hi << 128);
    // Load the range check values into the ALU registers
    auto [alu_u8_r0, alu_u8_r1, alu_u16_reg] = AvmAluTraceBuilder::to_alu_slice_registers(limb);

    FF c = 0;
    switch (in_tag) {
    case AvmMemoryTag::U8:
        c = FF{ uint8_t(c_u256) };
        break;
    case AvmMemoryTag::U16:
        c = FF{ uint16_t(c_u256) };
        break;
    case AvmMemoryTag::U32:
        c = FF{ uint32_t(c_u256) };
        break;
    case AvmMemoryTag::U64:
        c = FF{ uint64_t(c_u256) };
        break;
    case AvmMemoryTag::U128:
        c = FF{ uint256_t::from_uint128(uint128_t(c_u256)) };
        break;
    // Unsupported instruction tags, asserted earlier in function
    case AvmMemoryTag::U0:
    case AvmMemoryTag::FF:
        __builtin_unreachable();
    }

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .alu_op_shl = true,
        .alu_ff_tag = in_tag == AvmMemoryTag::FF,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ib = b,
        .alu_ic = c,
        .alu_u8_r0 = alu_u8_r0,
        .alu_u8_r1 = alu_u8_r1,
        .alu_u16_reg = alu_u16_reg,
        .hi_lo_limbs{ rng_chk_lo, rng_chk_hi, x_lo, x_hi },
        .mem_tag_bits = num_bits,
        .mem_tag_sub_shift = static_cast<uint8_t>(num_bits - b_u8),
        .shift_lt_bit_len = true,
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
    // Perform the shift operation over 256-bit integers
    uint256_t a_u256{ a };
    // Check that the shifted amount is an 8-bit integer
    ASSERT(uint256_t(b) < 256);
    ASSERT(in_tag != AvmMemoryTag::U0 || in_tag != AvmMemoryTag::FF);

    uint8_t b_u8 = static_cast<uint8_t>(uint256_t(b));
    uint256_t c_u256 = a_u256 >> b_u8;

    uint8_t num_bits = mem_tag_bits(in_tag);
    u8_pow_2_counters[0][b_u8]++;

    // If we are shifting more than the number of bits, the result is trivially 0
    if (b_u8 >= num_bits) {
        u8_pow_2_counters[1][b_u8 - num_bits]++;
        // Even though the registers are trivially zero, we call this function to increment the lookup counters
        // Future workaround would be to decouple the range_check toggle and the counter from this function
        [[maybe_unused]] auto [alu_u8_r0, alu_u8_r1, alu_u16_reg] = AvmAluTraceBuilder::to_alu_slice_registers(0);
        alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
            .alu_clk = clk,
            .alu_op_shr = true,
            .alu_ff_tag = in_tag == AvmMemoryTag::FF,
            .alu_u8_tag = in_tag == AvmMemoryTag::U8,
            .alu_u16_tag = in_tag == AvmMemoryTag::U16,
            .alu_u32_tag = in_tag == AvmMemoryTag::U32,
            .alu_u64_tag = in_tag == AvmMemoryTag::U64,
            .alu_u128_tag = in_tag == AvmMemoryTag::U128,
            .alu_ia = a,
            .alu_ib = b,
            .alu_ic = 0,
            .hi_lo_limbs = { 0, 0, 0, 0 },
            .mem_tag_bits = num_bits,
            .mem_tag_sub_shift = static_cast<uint8_t>(b_u8 - num_bits),
            .shift_lt_bit_len = false,
        });
        return 0;
    }
    // We decompose the input into two limbs partitioned at the b-th bit, we use x_lo and x_hi
    // to avoid any confusion with the a_lo and a_hi that form part of the range check
    auto [x_lo, x_hi] = decompose(a, b_u8);
    // We can modify the dynamic range check by performing an additional static one
    // rng_chk_lo = 2^b - x_lo - 1 && rng_chk_hi = 2^(num_bits - b) - x_hi - 1
    uint256_t rng_chk_lo = (uint256_t(1) << b_u8) - x_lo - 1;
    uint256_t rng_chk_hi = (uint256_t(1) << (num_bits - b_u8)) - x_hi - 1;

    // Each hi and lo limb is range checked over 128 bits
    uint256_t limb = rng_chk_lo + (rng_chk_hi << uint256_t(128));
    // Load the range check values into the ALU registers
    auto [alu_u8_r0, alu_u8_r1, alu_u16_reg] = AvmAluTraceBuilder::to_alu_slice_registers(limb);

    // Add counters for the pow of two lookups
    u8_pow_2_counters[1][num_bits - b_u8]++;

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .alu_op_shr = true,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ib = b,
        // Could be replaced with x_hi but nice to have 2 ways of calculating the result
        .alu_ic = FF(c_u256),
        .alu_u8_r0 = alu_u8_r0,
        .alu_u8_r1 = alu_u8_r1,
        .alu_u16_reg = alu_u16_reg,
        .hi_lo_limbs{ rng_chk_lo, rng_chk_hi, x_lo, x_hi },
        .mem_tag_bits = num_bits,
        .mem_tag_sub_shift = static_cast<uint8_t>(num_bits - b_u8),
        .shift_lt_bit_len = true,

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
    FF c;

    switch (in_tag) {
    case AvmMemoryTag::U8:
        c = FF(uint8_t(a));
        break;
    case AvmMemoryTag::U16:
        c = FF(uint16_t(a));
        break;
    case AvmMemoryTag::U32:
        c = FF(uint32_t(a));
        break;
    case AvmMemoryTag::U64:
        c = FF(uint64_t(a));
        break;
    case AvmMemoryTag::U128:
        c = FF(uint256_t::from_uint128(uint128_t(a)));
        break;
    case AvmMemoryTag::FF:
        c = a;
        break;
    default:
        c = 0;
        break;
    }

    // Get the decomposition of a
    auto [a_lo, a_hi] = decompose(uint256_t(a), 128);
    // Decomposition of p-a
    auto [p_sub_a_lo, p_sub_a_hi, p_a_borrow] = gt_witness(FF::modulus, uint256_t(a));
    auto [u8_r0, u8_r1, u16_reg] = to_alu_slice_registers(uint256_t(a));

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_clk = clk,
        .alu_op_cast = true,
        .alu_ff_tag = in_tag == AvmMemoryTag::FF,
        .alu_u8_tag = in_tag == AvmMemoryTag::U8,
        .alu_u16_tag = in_tag == AvmMemoryTag::U16,
        .alu_u32_tag = in_tag == AvmMemoryTag::U32,
        .alu_u64_tag = in_tag == AvmMemoryTag::U64,
        .alu_u128_tag = in_tag == AvmMemoryTag::U128,
        .alu_ia = a,
        .alu_ic = c,
        .alu_u8_r0 = u8_r0,
        .alu_u8_r1 = u8_r1,
        .alu_u16_reg = u16_reg,
        .hi_lo_limbs = { a_lo, a_hi, p_sub_a_lo, p_sub_a_hi },
        .p_a_borrow = p_a_borrow,
    });

    uint256_t sub = (p_sub_a_hi << 128) + p_sub_a_lo;
    auto [sub_u8_r0, sub_u8_r1, sub_u16_reg] = to_alu_slice_registers(sub);

    alu_trace.push_back(AvmAluTraceBuilder::AluTraceEntry{
        .alu_op_cast_prev = true,
        .alu_u8_r0 = sub_u8_r0,
        .alu_u8_r1 = sub_u8_r1,
        .alu_u16_reg = sub_u16_reg,
        .hi_lo_limbs = { p_sub_a_lo, p_sub_a_hi },
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
 * @return A boolean telling whether range check is required.
 */
bool AvmAluTraceBuilder::is_alu_row_enabled(AvmAluTraceBuilder::AluTraceEntry const& r)
{
    return (r.alu_op_add || r.alu_op_sub || r.alu_op_mul || r.alu_op_eq || r.alu_op_not || r.alu_op_lt ||
            r.alu_op_lte || r.alu_op_shr || r.alu_op_shl || r.alu_op_cast || r.alu_op_div);
}

} // namespace bb::avm_trace
