#include "./generator_data.hpp"

namespace crypto {
namespace generators {
namespace {

// Parameters for generator table construction
struct GeneratorParameters {
    size_t num_default_generators; // Number of unique base points with default main index with precomputed ladders
    size_t num_hash_indices;       // Number of unique hash indices
    size_t num_generators_per_hash_index; // Number of generators per hash index
    size_t hash_indices_generator_offset; // Offset for hash index generators
};

// Define BARRETENBERG_CRYPTO_GENERATOR_PARAMETERS_HACK to use custom values for generator parameters
// This hack is to avoid breakage due to generators in aztec circuits while maintaining compatibility
// with the barretenberg master.
#ifdef BARRETENBERG_CRYPTO_GENERATOR_PARAMETERS_HACK
constexpr GeneratorParameters GEN_PARAMS = { BARRETENBERG_CRYPTO_GENERATOR_PARAMETERS_HACK };
#else
#ifdef __wasm__
constexpr GeneratorParameters GEN_PARAMS = { 32, 16, 8, 2048 };
// TODO need to resolve memory out of bounds when these are too high
#else
constexpr GeneratorParameters GEN_PARAMS = { 2048, 16, 8, 2048 };
#endif
#endif

constexpr size_t num_indexed_generators = GEN_PARAMS.num_hash_indices * GEN_PARAMS.num_generators_per_hash_index;
constexpr size_t size_of_generator_data_array = GEN_PARAMS.hash_indices_generator_offset + num_indexed_generators;
constexpr size_t num_generator_types = 3;

ladder_t g1_ladder;
bool inited = false;

void compute_fixed_base_ladder(const grumpkin::g1::affine_element& generator, ladder_t& ladder)
{
    grumpkin::g1::element* ladder_temp =
        static_cast<grumpkin::g1::element*>(aligned_alloc(64, sizeof(grumpkin::g1::element) * (quad_length * 2)));

    grumpkin::g1::element accumulator;
    accumulator = grumpkin::g1::element(generator);
    for (size_t i = 0; i < quad_length; ++i) {
        ladder_temp[i] = accumulator;
        accumulator.self_dbl();
        ladder_temp[quad_length + i] = ladder_temp[i] + accumulator;
        accumulator.self_dbl();
    }
    grumpkin::g1::element::batch_normalize(&ladder_temp[0], quad_length * 2);
    for (size_t i = 0; i < quad_length; ++i) {
        grumpkin::fq::__copy(ladder_temp[i].x, ladder[quad_length - 1 - i].one.x);
        grumpkin::fq::__copy(ladder_temp[i].y, ladder[quad_length - 1 - i].one.y);
        grumpkin::fq::__copy(ladder_temp[quad_length + i].x, ladder[quad_length - 1 - i].three.x);
        grumpkin::fq::__copy(ladder_temp[quad_length + i].y, ladder[quad_length - 1 - i].three.y);
    }

    constexpr grumpkin::fq eight_inverse = grumpkin::fq{ 8, 0, 0, 0 }.to_montgomery_form().invert();
    std::array<grumpkin::fq, quad_length> y_denominators;
    for (size_t i = 0; i < quad_length; ++i) {

        grumpkin::fq x_beta = ladder[i].one.x;
        grumpkin::fq x_gamma = ladder[i].three.x;

        grumpkin::fq y_beta = ladder[i].one.y;
        grumpkin::fq y_gamma = ladder[i].three.y;
        grumpkin::fq x_beta_times_nine = x_beta + x_beta;
        x_beta_times_nine = x_beta_times_nine + x_beta_times_nine;
        x_beta_times_nine = x_beta_times_nine + x_beta_times_nine;
        x_beta_times_nine = x_beta_times_nine + x_beta;

        grumpkin::fq x_alpha_1 = ((x_gamma - x_beta) * eight_inverse);
        grumpkin::fq x_alpha_2 = ((x_beta_times_nine - x_gamma) * eight_inverse);

        grumpkin::fq T0 = x_beta - x_gamma;
        y_denominators[i] = (((T0 + T0) + T0));

        grumpkin::fq y_alpha_1 = ((y_beta + y_beta) + y_beta) - y_gamma;
        grumpkin::fq T1 = x_gamma * y_beta;
        T1 = ((T1 + T1) + T1);
        grumpkin::fq y_alpha_2 = ((x_beta * y_gamma) - T1);

        ladder[i].q_x_1 = x_alpha_1;
        ladder[i].q_x_2 = x_alpha_2;
        ladder[i].q_y_1 = y_alpha_1;
        ladder[i].q_y_2 = y_alpha_2;
    }
    grumpkin::fq::batch_invert(&y_denominators[0], quad_length);
    for (size_t i = 0; i < quad_length; ++i) {
        ladder[i].q_y_1 *= y_denominators[i];
        ladder[i].q_y_2 *= y_denominators[i];
    }
    free(ladder_temp);
}

/**
 * We need to derive three kinds of generators:
 *    1. generators (P[])
 *    2. aux_generators (P_aux[])
 *    3. skew_generators (P_skew[])
 * We use three generators to hash a single field element in the hash_single method:
 * H(f) = lambda * P[i]  +  gamma * P_aux[i]  -  skew * P_skew[i]
 */
template <size_t N> inline auto derive_generators()
{
    ASSERT((N % num_generator_types) == 0);
    std::vector<grumpkin::g1::affine_element> generators;
    std::vector<grumpkin::g1::affine_element> aux_generators;
    std::vector<grumpkin::g1::affine_element> skew_generators;
    auto res = grumpkin::g1::derive_generators<N>();
    for (size_t i = 0; i < N; i += num_generator_types) {
        generators.push_back(res[i]);
        aux_generators.push_back(res[i + 1]);
        skew_generators.push_back(res[i + 2]);
    }

    return std::make_tuple(generators, aux_generators, skew_generators);
}

auto compute_generator_data(grumpkin::g1::affine_element const& generator,
                            grumpkin::g1::affine_element const& aux_generator,
                            grumpkin::g1::affine_element const& skew_generator)
{
    auto gen_data = std::make_unique<generator_data>();
    gen_data->generator = generator;
    gen_data->aux_generator = aux_generator;
    gen_data->skew_generator = skew_generator;

    compute_fixed_base_ladder(generator, gen_data->ladder);
    compute_fixed_base_ladder(aux_generator, gen_data->aux_ladder);

    constexpr size_t first_generator_segment = quad_length - 2;
    constexpr size_t second_generator_segment = 2;

    for (size_t j = 0; j < first_generator_segment; ++j) {
        gen_data->hash_ladder[j] = gen_data->ladder[j + (quad_length - first_generator_segment)];
    }
    for (size_t j = 0; j < second_generator_segment; ++j) {
        gen_data->hash_ladder[j + first_generator_segment] =
            gen_data->aux_ladder[j + (quad_length - second_generator_segment)];
    }

    return gen_data;
}

const fixed_base_ladder* get_ladder_internal(std::array<fixed_base_ladder, quad_length> const& ladder,
                                             const size_t num_bits)
{
    // find n, such that 2n + 1 >= num_bits
    size_t n;
    if (num_bits == 0) {
        n = 0;
    } else {
        n = (num_bits - 1) >> 1;
        if (((n << 1) + 1) < num_bits) {
            ++n;
        }
    }
    const fixed_base_ladder* result = &ladder[quad_length - n - 1];
    return result;
}

} // namespace

/**
 * Precompute ladders and hash ladders
 *
 * `ladders` contains precomputed multiples of a base point
 *
 * Each entry in `ladders` is a `fixed_base_ladder` struct, which contains a pair of points,
 * `one` and `three`
 *
 * e.g. a size-4 `ladder` over a base point `P`, will have the following structure:
 *
 *    ladder[3].one = [P]
 *    ladder[3].three = 3[P]
 *    ladder[2].one = 4[P]
 *    ladder[2].three = 12[P]
 *    ladder[1].one = 16[P]
 *    ladder[1].three = 3*16[P]
 *    ladder[0].one = 64[P] + [P]
 *    ladder[0].three = 3*64[P]
 *
 * i.e. for a ladder size of `n`, we have the following:
 *
 *                        n - 1 - i
 *    ladder[i].one   = (4           ).[P]
 *                          n - 1 - i
 *    ladder[i].three = (3*4           ).[P]
 *
 * When a fixed-base scalar multiplier is decomposed into a size-2 WNAF, each ladder entry represents
 * the positive half of a WNAF table
 *
 * `hash_ladders` are stitched together from two `ladders` objects to preserve the uniqueness of a pedersen
 *hash. If a pedersen hash input is a 256-bit scalar, using a single generator point would mean that multiple
 *inputs would hash to the same output.
 *
 * e.g. if the grumpkin curve order is `n`, then hash(x) = hash(x + n) if we use a single generator
 *
 * For this reason, a hash ladder is built in a way that enables hashing the 252 higher bits of a 256 bit scalar
 * according to one generator and the four lower bits according to a second.
 *
 * Specifically,
 *
 *  1. For j=0,...,126, hash_ladders[i][j]=ladders[i][j] (i.e. generator  i)
 *  2. For j=127,128  hash_ladders[i][j]=aux_ladders[i][j] (i.e. auxiliary generator i)
 *
 * This is sufficient to create an injective hash for 256 bit strings
 * The reason we need 127 elements to hash 252 bits, or equivalently 126 quads, is that the first element of the
 *ladder is used simply to add the  "normalization factor" 4^{127}*[P] (so ladder[0].three is never used); this
 *addition makes all resultant scalars positive. When wanting to hash e.g. 254 instead of 256 bits, we will
 *start the ladder one step forward - this happends in `get_ladder_internal`
 **/
std::vector<std::unique_ptr<generator_data>> const& init_generator_data()
{
    static std::vector<std::unique_ptr<generator_data>> global_generator_data;
    if (inited) {
        return global_generator_data;
    }
    std::vector<grumpkin::g1::affine_element> generators;
    std::vector<grumpkin::g1::affine_element> aux_generators;
    std::vector<grumpkin::g1::affine_element> skew_generators;
    std::tie(generators, aux_generators, skew_generators) =
        derive_generators<size_of_generator_data_array * num_generator_types>();

    global_generator_data.resize(size_of_generator_data_array);

    for (size_t i = 0; i < GEN_PARAMS.num_default_generators; i++) {
        global_generator_data[i] = compute_generator_data(generators[i], aux_generators[i], skew_generators[i]);
    }

    for (size_t i = GEN_PARAMS.hash_indices_generator_offset; i < size_of_generator_data_array; i++) {
        global_generator_data[i] = compute_generator_data(generators[i], aux_generators[i], skew_generators[i]);
    }

    compute_fixed_base_ladder(grumpkin::g1::one, g1_ladder);

    inited = true;
    return global_generator_data;
};

const fixed_base_ladder* get_g1_ladder(const size_t num_bits)
{
    init_generator_data();
    return get_ladder_internal(g1_ladder, num_bits);
}

/**
 * Generator indexing:
 *
 * Number of default generators (index = 0): N = 2048
 * Number of hash indices: H = 32
 * Number of sub indices for a given hash index: h = 64.
 * Number of types of generators needed per hash index: t = 3
 *
 * Default generators:
 * 0: P_0  P_1  P_2  ...  P_{N'-1}
 *
 * Hash-index dependent generators: (let N' = t * N)
 * 1:  P_{N' + 0*h*t}   P_{N' + 0*h*t + 1*t}  ...  P_{N' + 0*h*t + (h-1)*t}
 * 2:  P_{N' + 1*h*t}   P_{N' + 1*h*t + 1*t}  ...  P_{N' + 1*h*t + (h-1)*t}
 * 2:  P_{N' + 2*h*t}   P_{N' + 2*h*t + 1*t}  ...  P_{N' + 2*h*t + (h-1)*t}
 * 4:
 * .
 * .
 * .
 * H-1:  P_{N' + (H-2)*h*t}   P_{N' + (H-2)*h*t + 1*t}  ...  P_{N' + (H-2)*h*t + (h-1)*t}
 * H  :  P_{N' + (H-1)*h*t}   P_{N' + (H-1)*h*t + 1*t}  ...  P_{N' + (H-1)*h*t + (h-1)*t}
 *
 * Total generators = (N + H * h) * t = 2304
 */
generator_data const& get_generator_data(generator_index_t index)
{
    auto& global_generator_data = init_generator_data();
    if (index.index == 0) {
        ASSERT(index.sub_index < GEN_PARAMS.num_default_generators);
        return *global_generator_data[index.sub_index];
    }
    ASSERT(index.index <= GEN_PARAMS.num_hash_indices);
    ASSERT(index.sub_index < GEN_PARAMS.num_generators_per_hash_index);
    return *global_generator_data[GEN_PARAMS.hash_indices_generator_offset +
                                  ((index.index - 1) * GEN_PARAMS.num_generators_per_hash_index) + index.sub_index];
}

const fixed_base_ladder* generator_data::get_ladder(size_t num_bits) const
{
    init_generator_data();
    return get_ladder_internal(ladder, num_bits);
}

const fixed_base_ladder* generator_data::get_hash_ladder(size_t num_bits) const
{
    init_generator_data();
    return get_ladder_internal(hash_ladder, num_bits);
}

} // namespace generators
} // namespace crypto