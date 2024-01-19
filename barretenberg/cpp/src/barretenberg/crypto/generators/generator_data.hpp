#pragma once

#include "barretenberg/common/container.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <array>
#include <map>
#include <optional>

namespace bb::crypto {
/**
 * @brief class that stores precomputed generators used for Pedersen commitments and Pedersen hashes
 *
 * @details We create distinct sets of generators via the use of a domain separator.
 *          This enables the use of context-specific commitments and hashes.
 *          For example, a circuit that generates commitments `foo = commit({ a, b })` and `bar = commit({c, d})` where
 *          `foo` and `bar` should not collide.
 *
 *          The goal of `generator_data` is twofold:
 *          1. Prevent redundant computation of the same generators at runtime (i.e. store in a singleton object)
 *          2. Compute a small number of default generators at compile-time, so that short processes that require a
 *             small number of generators do not have to execute the expensive `g1::derive_generators` method
 *
 *          We store generators in a key:value map, where the key is the domain separator and the value is the vector of
 *          associated generators. Pedersen methods take in a pointer to a `generator_data` object.
 *
 *          `generator_data` contains a static instantiation of the class: `default_data`.
 *          The intention is for `default_data` to be used as a singleton class.
 *          All Pedersen methods that require a `*generator_data` parameter (from now on referred to as "generator
 *          context") should default to using `default_data`.
 *
 *          Q: Why make the generator context an input parameter when it defaults to `default_data`?
 *          A: This is not thread-safe. Each process that uses a `generator_data` object may extend `generator_data` if
 *             more generators are required.
 *             i.e. either each process must use an independent `generator_data` object or the author must KNOW that
 *             `generator_data` will not be extended by any process
 *
 * @tparam Curve
 */
template <typename Curve> class generator_data {
  public:
    using Group = typename Curve::Group;
    using AffineElement = typename Curve::AffineElement;
    using GeneratorList = std::vector<AffineElement>;
    using GeneratorView = std::span<AffineElement const>;
    static inline constexpr size_t DEFAULT_NUM_GENERATORS = 8;
    static inline constexpr std::string_view DEFAULT_DOMAIN_SEPARATOR = "DEFAULT_DOMAIN_SEPARATOR";
    inline constexpr generator_data() = default;

    static inline constexpr std::array<AffineElement, DEFAULT_NUM_GENERATORS> make_precomputed_generators()
    {
        std::array<AffineElement, DEFAULT_NUM_GENERATORS> output;
        std::vector<AffineElement> res = Group::derive_generators(DEFAULT_DOMAIN_SEPARATOR, DEFAULT_NUM_GENERATORS, 0);
        std::copy(res.begin(), res.end(), output.begin());
        return output;
    }

    /**
     * @brief Precompute a small number of generators at compile time. For small pedersen commitments + pedersen hashes,
     * this prevents us from having to derive generators at runtime
     */
    static inline constexpr std::array<AffineElement, DEFAULT_NUM_GENERATORS> precomputed_generators =
        make_precomputed_generators();

    [[nodiscard]] inline GeneratorView get(const size_t num_generators,
                                           const size_t generator_offset = 0,
                                           const std::string_view domain_separator = DEFAULT_DOMAIN_SEPARATOR) const
    {
        const bool is_default_domain = domain_separator == DEFAULT_DOMAIN_SEPARATOR;
        if (is_default_domain && (num_generators + generator_offset) < DEFAULT_NUM_GENERATORS) {
            return GeneratorView{ precomputed_generators.data() + generator_offset, num_generators };
        }

        if (!generator_map.has_value()) {
            generator_map = std::map<std::string, GeneratorList>();
        }
        std::map<std::string, GeneratorList>& map = generator_map.value();

        // Case 2: we want default generators, but more than we precomputed at compile time. If we have not yet copied
        // the default generators into the map, do so.
        if (is_default_domain && !initialized_precomputed_generators) {
            map.insert({ std::string(DEFAULT_DOMAIN_SEPARATOR),
                         GeneratorList(precomputed_generators.begin(), precomputed_generators.end()) });
            initialized_precomputed_generators = true;
        }

        // if the generator map does not contain our desired generators, add entry into map
        if (!map.contains(std::string(domain_separator))) {
            map.insert({
                std::string(domain_separator),
                Group::derive_generators(domain_separator, num_generators + generator_offset, 0),
            });
        }

        GeneratorList& generators = map.at(std::string(domain_separator));

        // If the current GeneratorList does not contain enough generators, extend it
        if (num_generators + generator_offset > generators.size()) {
            const size_t num_extra_generators = num_generators + generator_offset - generators.size();
            GeneratorList extended_generators =
                Group::derive_generators(domain_separator, num_extra_generators, generators.size());
            generators.reserve(num_generators + generator_offset);
            std::copy(extended_generators.begin(), extended_generators.end(), std::back_inserter(generators));
        }

        return GeneratorView{ generators.data() + generator_offset, num_generators };
    }

    // getter method for `default_data`. Object exists as a singleton so we don't need a smart pointer.
    // Don't call `delete` on this pointer.
    static inline generator_data* get_default_generators() { return &default_data; }

  private:
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
    static inline constinit generator_data default_data = generator_data();

    // We mark the following two params as `mutable` so that our `get` method can be marked `const`.
    // A non-const getter creates downstream issues as all const methods that use a non-const `get`
    // would need to be marked const.
    // Rationale is that it's ok for `get` to be `const` because all changes are internal to the class and don't change
    // the external functionality of `generator_data`.
    // i.e. `generator_data.get` will return the same output regardless of the internal state of `generator_data`.

    // bool that describes whether we've copied the precomputed enerators into `generator_map`. This cannot be done at
    // compile-time because std::map is a dynamically sized object.
    mutable bool initialized_precomputed_generators = false;

    // We wrap the std::map in a `std::optional` so that we can construct `generator_data` at compile time.
    // This allows us to mark `default_data` as `constinit`, which prevents static initialization ordering fiasco
    mutable std::optional<std::map<std::string, GeneratorList>> generator_map = {};
};

template <typename Curve> struct GeneratorContext {
    size_t offset = 0;
    std::string domain_separator = std::string(generator_data<Curve>::DEFAULT_DOMAIN_SEPARATOR);
    generator_data<Curve>* generators = generator_data<Curve>::get_default_generators();

    GeneratorContext() = default;
    GeneratorContext(size_t hash_index)
        : offset(hash_index){};
    GeneratorContext(size_t _offset, std::string_view _domain_separator)
        : offset(_offset)
        , domain_separator(_domain_separator)
    {}
};
} // namespace bb::crypto