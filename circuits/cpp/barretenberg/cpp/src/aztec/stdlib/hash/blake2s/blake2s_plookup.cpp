#include "blake2s_plookup.hpp"
#include "blake_util.hpp"

#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <plonk/composer/plookup_tables/sha256.hpp>
#include <plonk/composer/ultra_composer.hpp>
#include <stdlib/primitives/bit_array/bit_array.hpp>
#include <stdlib/primitives/field/field.hpp>
#include <stdlib/primitives/uint/uint.hpp>
#include <stdlib/primitives/plookup/plookup.hpp>

/**
 * Optimizations:
 *
 * 1. use lookup tables for basic XOR operations
 * 2. replace use of uint32 with basic field_t type
 *
 **/
namespace plonk {
namespace stdlib {

namespace blake2s_plookup {

using plookup::ColumnIdx;

constexpr uint32_t blake2s_IV[8] = { 0x6A09E667UL, 0xBB67AE85UL, 0x3C6EF372UL, 0xA54FF53AUL,
                                     0x510E527FUL, 0x9B05688CUL, 0x1F83D9ABUL, 0x5BE0CD19UL };

constexpr uint32_t initial_H[8] = {
    0x6b08e647, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
};

enum blake2s_constant {
    BLAKE2S_BLOCKBYTES = 64,
    BLAKE2S_OUTBYTES = 32,
    BLAKE2S_KEYBYTES = 32,
    BLAKE2S_SALTBYTES = 8,
    BLAKE2S_PERSONALBYTES = 8
};

/**
 * The blake2s_state consists of the following components:
 * h: A 64-byte chain value denoted decomposed as (h_0, h_1, ..., h_7), each h_i is a 32-bit number.
 *    It form the first two rows on the internal state matrix v of the compression function G.
 *
 * t: It is a counter (t_0 lsb and t_1 msb) used in the the initialization of the internal state v.
 *
 * f: f_0 and f_1 are finalization flags used in the initialization of the internal state v.
 *           /  0xfff...ff   if the block processed is the last
 *    f_0 = |
 *           \  0x000...00   otherwise
 *           /  0xfff...ff   if the last node is processed in merkle-tree hashing
 *    f_1 = |
 *           \  0x000...00   otherwise
 *
 * Further, the internal state 4x4 matrix used by the compression function is denoted by v.
 * The input data is stored in the 16-word message m.
 */
template <typename Composer> struct blake2s_state {
    field_t<Composer> h[8];
    field_t<Composer> t[2];
    field_t<Composer> f[2];
};

template <typename Composer> void blake2s_increment_counter(blake2s_state<Composer>& S, const uint32_t inc)
{
    typedef field_t<Composer> field_pt;
    field_pt inc_scalar = field_pt(uint256_t(inc));
    S.t[0] = S.t[0] + inc_scalar;
    // TODO: Secure!? Think so as inc is known at "compile" time as it's derived from the msg length.
    const bool to_inc = uint32_t(uint256_t(S.t[0].get_value())) < inc;
    S.t[1] = S.t[1] + (to_inc ? field_pt(1) : field_pt(0));
}

template <typename Composer> void blake2s_compress(blake2s_state<Composer>& S, byte_array<Composer> const& in)
{
    typedef field_t<Composer> field_pt;
    field_pt m[16];
    field_pt v[16];

    for (size_t i = 0; i < 16; ++i) {
        m[i] = field_pt(in.slice(i * 4, 4).reverse());
    }

    for (size_t i = 0; i < 8; ++i) {
        v[i] = S.h[i];
    }

    v[8] = field_pt(uint256_t(blake2s_IV[0]));
    v[9] = field_pt(uint256_t(blake2s_IV[1]));
    v[10] = field_pt(uint256_t(blake2s_IV[2]));
    v[11] = field_pt(uint256_t(blake2s_IV[3]));

    // Use the lookup tables to perform XORs
    const auto lookup_1 =
        plookup_read::get_lookup_accumulators(BLAKE_XOR, S.t[0], field_pt(uint256_t(blake2s_IV[4])), true);
    v[12] = lookup_1[ColumnIdx::C3][0];
    const auto lookup_2 =
        plookup_read::get_lookup_accumulators(BLAKE_XOR, S.t[1], field_pt(uint256_t(blake2s_IV[5])), true);
    v[13] = lookup_2[ColumnIdx::C3][0];
    const auto lookup_3 =
        plookup_read::get_lookup_accumulators(BLAKE_XOR, S.f[0], field_pt(uint256_t(blake2s_IV[6])), true);
    v[14] = lookup_3[ColumnIdx::C3][0];
    const auto lookup_4 =
        plookup_read::get_lookup_accumulators(BLAKE_XOR, S.f[1], field_pt(uint256_t(blake2s_IV[7])), true);
    v[15] = lookup_4[ColumnIdx::C3][0];

    blake_util::round_fn_lookup<Composer>(v, m, 0);
    blake_util::round_fn_lookup<Composer>(v, m, 1);
    blake_util::round_fn_lookup<Composer>(v, m, 2);
    blake_util::round_fn_lookup<Composer>(v, m, 3);
    blake_util::round_fn_lookup<Composer>(v, m, 4);
    blake_util::round_fn_lookup<Composer>(v, m, 5);
    blake_util::round_fn_lookup<Composer>(v, m, 6);
    blake_util::round_fn_lookup<Composer>(v, m, 7);
    blake_util::round_fn_lookup<Composer>(v, m, 8);
    blake_util::round_fn_lookup<Composer>(v, m, 9);

    // At this point in the algorithm, the elements (v0, v1, v2, v3) and (v8, v9, v10, v11) in the state matrix 'v' can
    // be 'overflowed' i.e. contain values > 2^{32}. However we do NOT need to normalize them to be < 2^{32}, the
    // following `read_sequence_from_table` calls correctly constrain the output to be 32-bits
    for (size_t i = 0; i < 8; ++i) {
        const auto lookup_a = plookup_read::get_lookup_accumulators(BLAKE_XOR, S.h[i], v[i], true);
        const auto lookup_b =
            plookup_read::get_lookup_accumulators(BLAKE_XOR, lookup_a[ColumnIdx::C3][0], v[i + 8], true);
        S.h[i] = lookup_b[ColumnIdx::C3][0];
    }
}

template <typename Composer> void blake2s(blake2s_state<Composer>& S, byte_array<Composer> const& in)
{
    size_t offset = 0;
    size_t size = in.size();

    while (size > BLAKE2S_BLOCKBYTES) {
        blake2s_increment_counter(S, BLAKE2S_BLOCKBYTES);
        blake2s_compress<Composer>(S, in.slice(offset, BLAKE2S_BLOCKBYTES));
        offset += BLAKE2S_BLOCKBYTES;
        size -= BLAKE2S_BLOCKBYTES;
    }

    // Set last block.
    S.f[0] = field_t<Composer>(uint256_t((uint32_t)-1));

    byte_array<Composer> final(in.get_context());
    final.write(in.slice(offset)).write(byte_array<Composer>(in.get_context(), BLAKE2S_BLOCKBYTES - size));
    blake2s_increment_counter(S, (uint32_t)size);
    blake2s_compress<Composer>(S, final);
}

template <typename Composer> byte_array<Composer> blake2s(const byte_array<Composer>& input)
{
    blake2s_state<Composer> S;

    for (size_t i = 0; i < 8; i++) {
        S.h[i] = field_t<Composer>(uint256_t(initial_H[i]));
    }

    blake2s(S, input);

    byte_array<Composer> result(input.get_context());
    for (auto h : S.h) {
        byte_array<Composer> v(h, 4);
        result.write(v.reverse());
    }
    return result;
}

template byte_array<waffle::UltraComposer> blake2s(const byte_array<waffle::UltraComposer>& input);

} // namespace blake2s_plookup

} // namespace stdlib
} // namespace plonk
