#include "./generator_data.hpp"

// TODO(@zac-williamson #2341 delete this file once we migrate to new pedersen hash standard)

namespace crypto {
namespace generators {
namespace {

// The number of unique base points with default main index with precomputed ladders
constexpr size_t num_default_generators = 200;

/**
 * @brief Contains number of hash indices all of which support a fixed number of generators per index.
 */
struct HashIndexParams {
    size_t num_indices;
    size_t num_generators_per_index;

    /**
     * @brief Computes the total number of generators for a given HashIndexParams.
     *
     * @return Number of generators.
     */
    constexpr size_t total_generators() const { return (num_indices * num_generators_per_index); }
};

constexpr HashIndexParams LOW = { 32, 8 };
constexpr HashIndexParams MID = { 8, 16 };
constexpr HashIndexParams HIGH = { 4, 48 };

constexpr size_t num_hash_indices = (LOW.num_indices + MID.num_indices + HIGH.num_indices);
constexpr size_t num_indexed_generators = LOW.total_generators() + MID.total_generators() + HIGH.total_generators();

constexpr size_t size_of_generator_data_array = num_default_generators + num_indexed_generators;
constexpr size_t num_generator_types = 3;

ladder_t g1_ladder;
bool inited = false;

template <size_t ladder_length, size_t ladder_max_length>
void compute_fixed_base_ladder(const grumpkin::g1::affine_element& generator,
                               std::array<fixed_base_ladder, ladder_max_length>& ladder)
{
    ASSERT(ladder_length <= ladder_max_length);
    grumpkin::g1::element* ladder_temp =
        static_cast<grumpkin::g1::element*>(aligned_alloc(64, sizeof(grumpkin::g1::element) * (ladder_length * 2)));

    grumpkin::g1::element accumulator;
    accumulator = grumpkin::g1::element(generator);
    for (size_t i = 0; i < ladder_length; ++i) {
        ladder_temp[i] = accumulator;
        accumulator.self_dbl();
        ladder_temp[ladder_length + i] = ladder_temp[i] + accumulator;
        accumulator.self_dbl();
    }
    grumpkin::g1::element::batch_normalize(&ladder_temp[0], ladder_length * 2);
    for (size_t i = 0; i < ladder_length; ++i) {
        grumpkin::fq::__copy(ladder_temp[i].x, ladder[ladder_length - 1 - i].one.x);
        grumpkin::fq::__copy(ladder_temp[i].y, ladder[ladder_length - 1 - i].one.y);
        grumpkin::fq::__copy(ladder_temp[ladder_length + i].x, ladder[ladder_length - 1 - i].three.x);
        grumpkin::fq::__copy(ladder_temp[ladder_length + i].y, ladder[ladder_length - 1 - i].three.y);
    }

    constexpr grumpkin::fq eight_inverse = grumpkin::fq{ 8, 0, 0, 0 }.to_montgomery_form().invert();
    std::array<grumpkin::fq, ladder_length> y_denominators;
    for (size_t i = 0; i < ladder_length; ++i) {

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
    grumpkin::fq::batch_invert(&y_denominators[0], ladder_length);
    for (size_t i = 0; i < ladder_length; ++i) {
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

    compute_fixed_base_ladder<quad_length>(generator, gen_data->ladder);
    std::array<fixed_base_ladder, aux_length> aux_ladder_temp;
    compute_fixed_base_ladder<aux_length>(aux_generator, aux_ladder_temp);

    // Fill in the aux_generator multiples in the last two indices of the ladder.
    for (size_t j = 0; j < aux_length; ++j) {
        gen_data->ladder[j + quad_length] = aux_ladder_temp[j];
    }

    return gen_data;
}

const fixed_base_ladder* get_ladder_internal(ladder_t const& ladder, const size_t num_bits, const size_t offset = 0)
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
    const fixed_base_ladder* result = &ladder[quad_length + offset - n - 1];
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

    for (size_t i = 0; i < num_default_generators; i++) {
        global_generator_data[i] = compute_generator_data(generators[i], aux_generators[i], skew_generators[i]);
    }

    for (size_t i = num_default_generators; i < size_of_generator_data_array; i++) {
        global_generator_data[i] = compute_generator_data(generators[i], aux_generators[i], skew_generators[i]);
    }

    compute_fixed_base_ladder<quad_length>(grumpkin::g1::one, g1_ladder);

    inited = true;
    return global_generator_data;
};

const fixed_base_ladder* get_g1_ladder(const size_t num_bits)
{
    init_generator_data();
    return get_ladder_internal(g1_ladder, num_bits);
}

/**
 * @brief Returns a reference to the generator data for the specified generator index.
 * The generator index is composed of an index and sub-index. The index specifies
 * which hash index the generator belongs to, and the sub-index specifies the
 * position of the generator within the hash index.
 *
 * The generator data is stored in a global array of generator_data objects, which
 * is initialized lazily when the function is called for the first time. The global
 * array includes both default generators and user-defined generators.
 *
 * If the specified index is 0, the sub-index is used to look up the corresponding
 * default generator in the global array. Otherwise, the global index of the generator
 * is calculated based on the index and sub-index, and used to look up the corresponding
 * user-defined generator in the global array.
 *
 * The function throws an exception if the specified index is invalid.
 *
 * @param index The generator index, consisting of an index and sub-index.
 * @return A reference to the generator data for the specified generator index.
 * @throws An exception if the specified index is invalid.
 *
 * @note TODO: Write a generator indexing example
 */
generator_data const& get_generator_data(generator_index_t index)
{
    // Initialize the global array of generator data
    auto& global_generator_data = init_generator_data();

    // Handle default generators
    if (index.index == 0) {
        ASSERT(index.sub_index < num_default_generators);
        return *global_generator_data[index.sub_index];
    }

    // Handle user-defined generators
    ASSERT(index.index <= num_hash_indices);
    size_t global_index_offset = 0;
    if (0 < index.index && index.index <= LOW.num_indices) {
        // Calculate the global index of the generator for the LOW hash index
        ASSERT(index.sub_index < LOW.num_generators_per_index);
        const size_t local_index_offset = 0;
        const size_t generator_count_offset = 0;
        global_index_offset =
            generator_count_offset + (index.index - local_index_offset - 1) * LOW.num_generators_per_index;

    } else if (index.index <= (LOW.num_indices + MID.num_indices)) {
        // Calculate the global index of the generator for the MID hash index
        ASSERT(index.sub_index < MID.num_generators_per_index);
        const size_t local_index_offset = LOW.num_indices;
        const size_t generator_count_offset = LOW.total_generators();
        global_index_offset =
            generator_count_offset + (index.index - local_index_offset - 1) * MID.num_generators_per_index;

    } else if (index.index <= (LOW.num_indices + MID.num_indices + HIGH.num_indices)) {
        // Calculate the global index of the generator for the HIGH hash index
        const size_t local_index_offset = LOW.num_indices + MID.num_indices;
        const size_t generator_count_offset = LOW.total_generators() + MID.total_generators();
        ASSERT(index.sub_index < HIGH.num_generators_per_index);
        global_index_offset =
            generator_count_offset + (index.index - local_index_offset - 1) * HIGH.num_generators_per_index;

    } else {
        // Throw an exception for invalid index values
        throw_or_abort(format("invalid hash index: ", index.index));
    }

    // Return a reference to the user-defined generator with the specified index and sub-index
    return *global_generator_data[num_default_generators + global_index_offset + index.sub_index];
}

const fixed_base_ladder* generator_data::get_ladder(size_t num_bits) const
{
    init_generator_data();
    return get_ladder_internal(ladder, num_bits);
}

const fixed_base_ladder* generator_data::get_hash_ladder(size_t num_bits) const
{
    init_generator_data();
    return get_ladder_internal(ladder, num_bits, aux_length);
}

} // namespace generators
} // namespace crypto