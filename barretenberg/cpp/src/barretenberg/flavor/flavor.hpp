/**
 * @file flavor.hpp
 * @brief Base class templates for structures that contain data parameterized by the fundamental polynomials of a Honk
 * variant (a "flavor").
 *
 * @details #Motivation
 * We choose the framework set out in these classes for several reasons.
 *
 * For one, it allows for a large amount of the information of a Honk flavor to be read at a glance in a single file.
 *
 * The primary motivation, however, is to reduce the sort loose of coupling that is a significant source of complexity
 * in the original plonk code. There, we find many similarly-named entities defined in many different places (to name
 * some: selector_properties; FooSelectors; PolynomialIndex; the labels given to the polynomial store; the commitment
 * label; inconsistent terminology and notation around these), and it can be difficult to discover or remember the
 * relationships between these. We aim for a more uniform treatment, to enfore identical and informative naming, and to
 * prevent the developer having to think very much about the ordering of protocol entities in disparate places.
 *
 * Another motivation is iterate on the polynomial manifest of plonk, which is nice in its compactness, but which feels
 * needlessly manual and low-level. In the past, this contained even more boolean parameters, making it quite hard to
 * parse. A typical construction is to loop over the polynomial manifest by extracting a globally-defined
 * "FOO_MANIFEST_SIZE" (the use of "manifest" here is distinct from the manifests in the transcript) to loop
 * over a C-style array, and then manually parsing the various tags of different types in the manifest entries. We
 * greatly enrich this structure by using basic C++ OOP functionality. Rather than recording the polynomial source in an
 * enum, we group polynomial handles using getter functions in our new class. We get code that is more compact,
 * more legible, and which is safer because it admits ranged `for` loops.
 *
 * Another motivation is proper and clear specification of Honk variants. The flavors are meant to be explicit and
 * easily comparable. In plonk, the various settings template parameters and objects like the CircuitType enum became
 * overloaded in time, and continue to be a point of accumulation for tech debt. We aim to remedy some of this by
 * putting proving system information in the flavor, and circuit construction information in the arithmetization (or
 * larger circuit constructor class).
 *
 * @details #Data model
 * All of the flavor classes derive from a single Entities_ template, which simply wraps a std::array (we would
 * inherit, but this is unsafe as std::array has a non-virtual destructor). The developer should think of every flavor
 * class as being:
 *  - A std::array<DataType, N> instance called _data.
 *  - An informative name for each entry of _data that is fixed at compile time.
 *  - Some classic metadata like we'd see in plonk (e.g., a circuit size, a reference string, an evaluation domain).
 *  - A collection of getters that record subsets of the array that are of interest in the Honk variant.
 *
 * Each getter returns a container of HandleType's, where a HandleType is a value type that is inexpensive to create and
 * that lets one view and mutate a DataType instance. The primary example here is that std::span is the handle type
 * chosen for barrtenberg::Polynomial.
 *
 * @details #Some Notes
 *
 * @note It would be ideal to codify more structure in these base class template and to have it imposed on the actual
 * flavors, but our inheritance model is complicated as it is, and we saw no reasonable way to fix this.
 *
 * @note One asymmetry to note is in the use of the term "key". It is worthwhile to distinguish betwen prover/verifier
 * circuit data, and "keys" that consist of such data augmented with witness data (whether, raw, blinded, or polynomial
 * commitments). Currently the proving key contains witness data, while the verification key does not.
 * TODO(Cody): It would be nice to resolve this but it's not essential.
 *
 * @note The VerifierCommitments classes are not 'tight' in the sense that that the underlying array contains(a few)
 * empty slots. This is a conscious choice to limit complexity. Note that there is very little memory cost here since
 * the DataType size in that case is small.
 *
 * @todo TODO(#395): Getters should return arrays?
 * @todo TODO(#396): Access specifiers?
 * @todo TODO(#397): Use more handle types?
 * @todo TODO(#398): Selectors should come from arithmetization.
 */

#pragma once
#include "barretenberg/common/ref_vector.hpp"
#include "barretenberg/common/std_array.hpp"
#include "barretenberg/common/std_vector.hpp"
#include "barretenberg/common/zip_view.hpp"
#include "barretenberg/polynomials/barycentric.hpp"
#include "barretenberg/polynomials/evaluation_domain.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include <array>
#include <concepts>
#include <vector>

namespace bb::honk::flavor {

/**
 * @brief Base class template containing circuit-specifying data.
 *
 */
class PrecomputedEntitiesBase {
  public:
    size_t circuit_size;
    size_t log_circuit_size;
    size_t num_public_inputs;
    CircuitType circuit_type; // TODO(#392)
};

/**
 * @brief Base proving key class.
 *
 * @tparam PrecomputedEntities An instance of PrecomputedEntities_ with polynomial data type and span handle type.
 * @tparam FF The scalar field on which we will encode our polynomial data. When instantiating, this may be extractable
 * from the other template paramter.
 */
template <typename PrecomputedPolynomials, typename WitnessPolynomials>
class ProvingKey_ : public PrecomputedPolynomials, public WitnessPolynomials {
  public:
    using Polynomial = typename PrecomputedPolynomials::DataType;
    using FF = typename Polynomial::FF;

    bool contains_recursive_proof;
    std::vector<uint32_t> recursive_proof_public_input_indices;
    bb::EvaluationDomain<FF> evaluation_domain;

    std::vector<std::string> get_labels() const
    {
        return concatenate(PrecomputedPolynomials::get_labels(), WitnessPolynomials::get_labels());
    }
    // This order matters! must match get_unshifted in entity classes
    RefVector<Polynomial> get_all() { return concatenate(get_precomputed_polynomials(), get_witness_polynomials()); }
    RefVector<Polynomial> get_witness_polynomials() { return WitnessPolynomials::get_all(); }
    RefVector<Polynomial> get_precomputed_polynomials() { return PrecomputedPolynomials::get_all(); }
    ProvingKey_() = default;
    ProvingKey_(const size_t circuit_size, const size_t num_public_inputs)
    {
        this->evaluation_domain = bb::EvaluationDomain<FF>(circuit_size, circuit_size);
        PrecomputedPolynomials::circuit_size = circuit_size;
        this->log_circuit_size = numeric::get_msb(circuit_size);
        this->num_public_inputs = num_public_inputs;
        // Allocate memory for precomputed polynomials
        for (auto& poly : PrecomputedPolynomials::get_all()) {
            poly = Polynomial(circuit_size);
        }
        // Allocate memory for witness polynomials
        for (auto& poly : WitnessPolynomials::get_all()) {
            poly = Polynomial(circuit_size);
        }
    };
};

/**
 * @brief Base verification key class.
 *
 * @tparam PrecomputedEntities An instance of PrecomputedEntities_ with affine_element data type and handle type.
 */
template <typename PrecomputedCommitments> class VerificationKey_ : public PrecomputedCommitments {
  public:
    VerificationKey_() = default;
    VerificationKey_(const size_t circuit_size, const size_t num_public_inputs)
    {
        this->circuit_size = circuit_size;
        this->log_circuit_size = numeric::get_msb(circuit_size);
        this->num_public_inputs = num_public_inputs;
    };
};

// Because of how Gemini is written, is importat to put the polynomials out in this order.
auto get_unshifted_then_shifted(const auto& all_entities)
{
    return concatenate(all_entities.get_unshifted(), all_entities.get_shifted());
};

/**
 * @brief Recursive utility function to find max PARTIAL_RELATION_LENGTH tuples of Relations.
 * @details The "partial length" of a relation is 1 + the degree of the relation, where any challenges used in the
 * relation are as constants, not as variables..
 *
 */
template <typename Tuple, std::size_t Index = 0> static constexpr size_t compute_max_partial_relation_length()
{
    if constexpr (Index >= std::tuple_size<Tuple>::value) {
        return 0; // Return 0 when reach end of the tuple
    } else {
        constexpr size_t current_length = std::tuple_element<Index, Tuple>::type::RELATION_LENGTH;
        constexpr size_t next_length = compute_max_partial_relation_length<Tuple, Index + 1>();
        return (current_length > next_length) ? current_length : next_length;
    }
}

/**
 * @brief Recursive utility function to find max TOTAL_RELATION_LENGTH among tuples of Relations.
 * @details The "total length" of a relation is 1 + the degree of the relation, where any challenges used in the
 * relation are regarded as variables.
 *
 */
template <typename Tuple, std::size_t Index = 0> static constexpr size_t compute_max_total_relation_length()
{
    if constexpr (Index >= std::tuple_size<Tuple>::value) {
        return 0; // Return 0 when reach end of the tuple
    } else {
        constexpr size_t current_length = std::tuple_element<Index, Tuple>::type::TOTAL_RELATION_LENGTH;
        constexpr size_t next_length = compute_max_total_relation_length<Tuple, Index + 1>();
        return (current_length > next_length) ? current_length : next_length;
    }
}

/**
 * @brief Recursive utility function to find the number of subrelations.
 *
 */
template <typename Tuple, std::size_t Index = 0> static constexpr size_t compute_number_of_subrelations()
{
    if constexpr (Index >= std::tuple_size<Tuple>::value) {
        return 0;
    } else {
        constexpr size_t subrelations_in_relation =
            std::tuple_element_t<Index, Tuple>::SUBRELATION_PARTIAL_LENGTHS.size();
        return subrelations_in_relation + compute_number_of_subrelations<Tuple, Index + 1>();
    }
}
/**
 * @brief Recursive utility function to construct a container for the subrelation accumulators of Protogalaxy folding.
 * @details The size of the outer tuple is equal to the number of relations. Each relation contributes an inner tuple of
 * univariates whose size is equal to the number of subrelations of the relation. The length of a univariate in an inner
 * tuple is determined by the corresponding subrelation length and the number of instances to be folded.
 */
template <typename Tuple, size_t NUM_INSTANCES, size_t Index = 0>
static constexpr auto create_protogalaxy_tuple_of_tuples_of_univariates()
{
    if constexpr (Index >= std::tuple_size<Tuple>::value) {
        return std::tuple<>{}; // Return empty when reach end of the tuple
    } else {
        using UnivariateTuple =
            typename std::tuple_element_t<Index,
                                          Tuple>::template ProtogalaxyTupleOfUnivariatesOverSubrelations<NUM_INSTANCES>;
        return std::tuple_cat(std::tuple<UnivariateTuple>{},
                              create_protogalaxy_tuple_of_tuples_of_univariates<Tuple, NUM_INSTANCES, Index + 1>());
    }
}

/**
 * @brief Recursive utility function to construct a container for the subrelation accumulators of sumcheck proving.
 * @details The size of the outer tuple is equal to the number of relations. Each relation contributes an inner tuple of
 * univariates whose size is equal to the number of subrelations of the relation. The length of a univariate in an inner
 * tuple is determined by the corresponding subrelation length.
 */
template <typename Tuple, std::size_t Index = 0> static constexpr auto create_sumcheck_tuple_of_tuples_of_univariates()
{
    if constexpr (Index >= std::tuple_size<Tuple>::value) {
        return std::tuple<>{}; // Return empty when reach end of the tuple
    } else {
        using UnivariateTuple = typename std::tuple_element_t<Index, Tuple>::SumcheckTupleOfUnivariatesOverSubrelations;
        return std::tuple_cat(std::tuple<UnivariateTuple>{},
                              create_sumcheck_tuple_of_tuples_of_univariates<Tuple, Index + 1>());
    }
}

/**
 * @brief Recursive utility function to construct tuple of arrays
 * @details Container for storing value of each identity in each relation. Each Relation contributes an array of
 * length num-identities.
 */
template <typename Tuple, std::size_t Index = 0> static constexpr auto create_tuple_of_arrays_of_values()
{
    if constexpr (Index >= std::tuple_size<Tuple>::value) {
        return std::tuple<>{}; // Return empty when reach end of the tuple
    } else {
        using Values = typename std::tuple_element_t<Index, Tuple>::SumcheckArrayOfValuesOverSubrelations;
        return std::tuple_cat(std::tuple<Values>{}, create_tuple_of_arrays_of_values<Tuple, Index + 1>());
    }
}

} // namespace bb::honk::flavor

// Forward declare honk flavors
namespace bb::honk::flavor {
class Ultra;
class ECCVM;
class GoblinUltra;
template <typename BuilderType> class UltraRecursive_;
template <typename BuilderType> class GoblinUltraRecursive_;
} // namespace bb::honk::flavor

// Forward declare plonk flavors
namespace bb::plonk::flavor {
class Standard;
class Ultra;
} // namespace bb::plonk::flavor

// Establish concepts for testing flavor attributes
namespace bb {
/**
 * @brief Test whether a type T lies in a list of types ...U.
 *
 * @tparam T The type being tested
 * @tparam U A parameter pack of types being checked against T.
 */
// clang-format off

template <typename T>
concept IsPlonkFlavor = IsAnyOf<T, plonk::flavor::Standard, plonk::flavor::Ultra>;

template <typename T> 
concept IsHonkFlavor = IsAnyOf<T, honk::flavor::Ultra, honk::flavor::GoblinUltra>;

template <typename T> 
concept IsUltraFlavor = IsAnyOf<T, honk::flavor::Ultra, honk::flavor::GoblinUltra>;

template <typename T> 
concept IsGoblinFlavor = IsAnyOf<T, honk::flavor::GoblinUltra,
                                    honk::flavor::GoblinUltraRecursive_<UltraCircuitBuilder>,
                                    honk::flavor::GoblinUltraRecursive_<GoblinUltraCircuitBuilder>>;

template <typename T> 
concept IsRecursiveFlavor = IsAnyOf<T, honk::flavor::UltraRecursive_<UltraCircuitBuilder>, 
                                       honk::flavor::UltraRecursive_<GoblinUltraCircuitBuilder>, 
                                       honk::flavor::GoblinUltraRecursive_<UltraCircuitBuilder>,
                                       honk::flavor::GoblinUltraRecursive_<GoblinUltraCircuitBuilder>>;

template <typename T> concept IsGrumpkinFlavor = IsAnyOf<T, honk::flavor::ECCVM>;

template <typename T> concept IsFoldingFlavor = IsAnyOf<T, honk::flavor::Ultra, 
                                                           honk::flavor::GoblinUltra, 
                                                           honk::flavor::UltraRecursive_<UltraCircuitBuilder>, 
                                                           honk::flavor::UltraRecursive_<GoblinUltraCircuitBuilder>, 
                                                           honk::flavor::GoblinUltraRecursive_<UltraCircuitBuilder>, 
                                                           honk::flavor::GoblinUltraRecursive_<GoblinUltraCircuitBuilder>>;

template <typename T> concept UltraFlavor = IsAnyOf<T, honk::flavor::Ultra, honk::flavor::GoblinUltra>;

template <typename T> concept ECCVMFlavor = IsAnyOf<T, honk::flavor::ECCVM>;

template <typename Container, typename Element>
inline std::string flavor_get_label(Container&& container, const Element& element) {
    for (auto [label, data] : zip_view(container.get_labels(), container.get_all())) {
        if (&data == &element) {
            return label;
        }
    }
    return "(unknown label)";
}

// clang-format on
} // namespace bb
