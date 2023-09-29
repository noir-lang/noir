#pragma once

// TODO(@zac-wiliamson #2341 rename to pedersen.hpp once we migrate to new hash standard)

#include "../generators/fixed_base_scalar_mul.hpp"
#include "../generators/generator_data.hpp"
#include "../pedersen_commitment/pedersen_refactor.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <array>

namespace crypto {

/**
 * @brief Performs pedersen hashes!
 *
 * To hash to a size-n list of field elements `x`, we return the X-coordinate of:
 *
 *      Hash(x) = n.[h] + x_0. [g_0] + x_1 . [g_1] +... + x_n . [g_n]
 *
 * Where `g` is a list of generator points defined by `generator_data`
 * And `h` is a unique generator whose domain separator is the string `pedersen_hash_length`.
 *
 * The addition of `n.[h]` into the hash is to prevent length-extension attacks.
 * It also ensures that the hash output is never the point at infinity.
 *
 * It is neccessary that all generator points are linearly independent of one another,
 * so that finding collisions is equivalent to solving the discrete logarithm problem.
 * This is ensured via the generator derivation algorithm in `generator_data`
 */
template <typename Curve> class pedersen_hash_refactor {
  public:
    using AffineElement = typename Curve::AffineElement;
    using Element = typename Curve::Element;
    using Fr = typename Curve::ScalarField;
    using Fq = typename Curve::BaseField;
    using Group = typename Curve::Group;
    using generator_data = typename crypto::generator_data<Curve>;

    /**
     * @brief lhs_generator is an alias for the first element in `default_generators`.
     *        i.e. the 1st generator point in a size-2 pedersen hash
     *
     * @details Short story: don't make global static member variables publicly accessible.
     *          Ordering of global static variable initialization is not defined.
     *          Consider a scenario where this class has `inline static const AffineElement lhs_generator;`
     *          If another static variable's init function accesses `pedersen_hash_refactor::lhs_generator`,
     *          there is a chance that `lhs_generator` is not yet initialized due to undefined init order.
     *          This creates merry havoc due to assertions triggering during runtime initialization of global statics.
     *          So...don't do that. Wrap your statics.
     */
    inline static AffineElement get_lhs_generator() { return generator_data::get_default_generators()->get(0); }
    /**
     * @brief rhs_generator is an alias for the second element in `default_generators`.
     *        i.e. the 2nd generator point in a size-2 pedersen hash
     */
    inline static AffineElement get_rhs_generator() { return generator_data::get_default_generators()->get(1); }
    /**
     * @brief length_generator is used to ensure pedersen hash is not vulnerable to length-exstension attacks
     */
    inline static AffineElement get_length_generator()
    {
        static const AffineElement length_generator = Group::get_secure_generator_from_index(0, "pedersen_hash_length");
        return length_generator;
    }

    // TODO(@suyash67) as part of refactor project, can we remove this and replace with `hash`
    // (i.e. simplify the name as we no longer have a need for `hash_single`)
    static Fq hash_multiple(const std::vector<Fq>& inputs,
                            size_t hash_index = 0,
                            const generator_data* generator_context = generator_data::get_default_generators());

    static Fq hash(const std::vector<Fq>& inputs,
                   size_t hash_index = 0,
                   const generator_data* generator_context = generator_data::get_default_generators());
};

extern template class pedersen_hash_refactor<curve::Grumpkin>;
} // namespace crypto
