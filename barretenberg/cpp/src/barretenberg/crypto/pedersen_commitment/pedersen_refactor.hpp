#pragma once

// TODO(@zac-wiliamson #2341 rename to pedersen.hpp once we migrate to new hash standard)

#include "../generators/fixed_base_scalar_mul.hpp"
#include "../generators/generator_data.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <array>

namespace crypto {

/**
 * @brief Contains a vector of precomputed generator points.
 *        Generators are defined via a domain separator.
 *        Number of generators in generator_data is fixed for a given object instance.
 *
 * @details generator_data is used to precompute short lists of commonly used generators,
 *          (e.g. static inline const default_generators = generator_data()).
 *          If an algorithm requires more than `_size_ generators,
 *          the `conditional_extend` method can be called to return a new `generator_data` object.
 *          N.B. we explicitly do not support mutating an existing `generator_data` object to increase the size of
 *          its `std::vector<affine_element> generators` member variable.
 *          This is because this class is intended to be used as a `static` member of other classes to provide lists
 * of precomputed generators. Mutating static member variables is *not* thread safe!
 */
template <typename Curve> class generator_data {
  public:
    using Group = typename Curve::Group;
    using AffineElement = typename Curve::AffineElement;
    static inline constexpr size_t DEFAULT_NUM_GENERATORS = 32;
    static inline const std::string DEFAULT_DOMAIN_SEPARATOR = "default_domain_separator";
    inline generator_data(const size_t num_generators = DEFAULT_NUM_GENERATORS,
                          const std::string& domain_separator = DEFAULT_DOMAIN_SEPARATOR)
        : _domain_separator(domain_separator)
        , _domain_separator_bytes(domain_separator.begin(), domain_separator.end())
        , _size(num_generators){};

    [[nodiscard]] inline std::string domain_separator() const { return _domain_separator; }
    [[nodiscard]] inline size_t size() const { return _size; }
    [[nodiscard]] inline AffineElement get(const size_t index, const size_t offset = 0) const
    {
        ASSERT(index + offset <= _size);
        return generators[index + offset];
    }

    /**
     * @brief If more generators than `_size` are required, this method will return a new `generator_data` object
     *        with the required generators.
     *
     * @note Question: is this a good pattern to support? Ideally downstream code would ensure their
     *       `generator_data` object is sufficiently large to cover potential needs.
     *       But if we did not support this pattern, it would make downstream code more complex as each method that
     *       uses `generator_data` would have to perform this accounting logic.
     *
     * @param target_num_generators
     * @return generator_data
     */
    [[nodiscard]] inline generator_data conditional_extend(const size_t target_num_generators) const
    {
        if (target_num_generators <= _size) {
            return *this;
        }
        return { target_num_generators, _domain_separator };
    }

  private:
    std::string _domain_separator;
    std::vector<uint8_t> _domain_separator_bytes;
    size_t _size;
    // ordering of static variable initialization is undefined, so we make `default_generators` private
    // and only accessible via `get_default_generators()`, which ensures var will be initialized at the cost of some
    // small runtime checks
    inline static const generator_data default_generators =
        generator_data(generator_data::DEFAULT_NUM_GENERATORS, generator_data::DEFAULT_DOMAIN_SEPARATOR);

  public:
    inline static const generator_data* get_default_generators() { return &default_generators; }
    const std::vector<AffineElement> generators = (Group::derive_generators_secure(_domain_separator_bytes, _size));
};

template class generator_data<curve::Grumpkin>;

/**
 * @brief Performs pedersen commitments!
 *
 * To commit to a size-n list of field elements `x`, a commitment is defined as:
 *
 *      Commit(x) = x[0].g[0] + x[1].g[1] + ... + x[n-1].g[n-1]
 *
 * Where `g` is a list of generator points defined by `generator_data`
 *
 */
template <typename Curve> class pedersen_commitment_refactor {
  public:
    using AffineElement = typename Curve::AffineElement;
    using Element = typename Curve::Element;
    using Fr = typename Curve::ScalarField;
    using Fq = typename Curve::BaseField;
    using Group = typename Curve::Group;

    static AffineElement commit_native(
        const std::vector<Fq>& inputs,
        size_t hash_index = 0,
        const generator_data<Curve>* generator_context = generator_data<Curve>::get_default_generators());
};

extern template class pedersen_commitment_refactor<curve::Grumpkin>;
} // namespace crypto
