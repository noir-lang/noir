/**
 * @file relation_definer.hpp
 * @author Rumata888
 * @brief This file contains settings for the General Permutation Relation implementations and (in the future) Lookup
 * implementations
 *
 */
#pragma once
#include <cstddef>
#include <tuple>
namespace proof_system::honk::sumcheck {

/**
 * @brief This class contains an example of how to set PermutationSettings classes used by the
 * GenericPermutationRelationImpl class to specify a concrete permutation
 *
 * @details To create your own permutation:
 * 1) Create a copy of this class and rename it
 * 2) Update all the values with the ones needed for your permutation
 * 3) Update "DECLARE_IMPLEMENTATIONS_FOR_ALL_SETTINGS" and "DEFINE_IMPLEMENTATIONS_FOR_ALL_SETTINGS" to include the new
 * settings
 * 4) Add the relation with the chosen settings to Relations in the flavor (for example,"`
 *   using Relations = std::tuple<sumcheck::GenericPermutationRelation<sumcheck::ExamplePermutationSettings, FF>>;)`
 *
 */
class ExampleTuplePermutationSettings {
  public:
    // This constant defines how many columns are bundled together to form each set. For example, in this case we are
    // bundling tuples of (permutation_set_column_1, permutation_set_column_2) to be a permutation of
    // (permutation_set_column_3,permutation_set_column_4). As the tuple has 2 elements, set the value to 2
    constexpr static size_t COLUMNS_PER_SET = 2;

    /**
     * @brief If this method returns true on a row of values, then the inverse polynomial at this index. Otherwise the
     * value needs to be set to zero.
     *
     * @details If this is true then permutation takes place in this row
     *
     */
    template <typename AllEntities> static inline bool inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.enable_tuple_set_permutation == 1);
    }

    /**
     * @brief Get all the entities for the permutation when we don't need to update them
     *
     * @details The entities are returned as a tuple of references in the following order:
     * - The entity/polynomial used to store the product of the inverse values
     * - The entity/polynomial that switches on the subrelation of the permutation relation that ensures correctness of
     * the inverse polynomial
     * - The entity/polynomial that enables adding a tuple-generated value from the first set to the logderivative sum
     * subrelation
     * - The entity/polynomial that enables adding a tuple-generated value from the second set to the logderivative sum
     * subrelation
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the first set (N.B. ORDER IS IMPORTANT!)
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the second set (N.B. ORDER IS IMPORTANT!)
     *
     * @return All the entities needed for the permutation
     */
    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {

        return std::forward_as_tuple(
            in.tuple_permutation_inverses,   /* The polynomial containing the inverse product*/
            in.enable_tuple_set_permutation, /* The polynomial enabling the product check subrelation */
            in.enable_tuple_set_permutation, /* Enables adding first set to the sum */
            in.enable_tuple_set_permutation, /* Enables adding second set to the sum */
            in.permutation_set_column_3,     /* The first entry in the first set tuple */
            in.permutation_set_column_4,     /* The second entry in the first set tuple */
            in.permutation_set_column_1,     /* The first entry in the second set tuple */
            in.permutation_set_column_2);    /* The second entry in the second set tuple */
    }

    /**
     * @brief Get all the entities for the permutation when need to update them
     *
     * @details The entities are returned as a tuple of references in the following order:
     * - The entity/polynomial used to store the product of the inverse values
     * - The entity/polynomial that switches on the subrelation of the permutation relation that ensures correctness of
     * the inverse polynomial
     * - The entity/polynomial that enables adding a tuple-generated value from the first set to the logderivative sum
     * subrelation
     * - The entity/polynomial that enables adding a tuple-generated value from the second set to the logderivative sum
     * subrelation
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the first set (N.B. ORDER IS IMPORTANT!)
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the second set (N.B. ORDER IS IMPORTANT!)
     *
     * @return All the entities needed for the permutation
     */
    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(
            in.tuple_permutation_inverses,   /* The polynomial containing the inverse product*/
            in.enable_tuple_set_permutation, /* The polynomial enabling the product check subrelation */
            in.enable_tuple_set_permutation, /* Enables adding first set to the sum */
            in.enable_tuple_set_permutation, /* Enables adding second set to the sum */
            in.permutation_set_column_3,     /* The first entry in the first set tuple */
            in.permutation_set_column_4,     /* The second entry in the first set tuple */
            in.permutation_set_column_1,     /* The first entry in the second set tuple */
            in.permutation_set_column_2);    /* The second entry in the second set tuple */
    }
};

/**
 * @brief This class contains an example of how to set PermutationSettings classes used by the
 * GenericPermutationRelationImpl class to specify a concrete permutation
 *
 * @details To create your own permutation:
 * 1) Create a copy of this class and rename it
 * 2) Update all the values with the ones needed for your permutation
 * 3) Update "DECLARE_IMPLEMENTATIONS_FOR_ALL_SETTINGS" and "DEFINE_IMPLEMENTATIONS_FOR_ALL_SETTINGS" to include the new
 * settings
 * 4) Add the relation with the chosen settings to Relations in the flavor (for example,"`
 *   using Relations = std::tuple<sumcheck::GenericPermutationRelation<sumcheck::ExamplePermutationSettings, FF>>;)`
 *
 */
class ExampleSameWirePermutationSettings {
  public:
    // This constant defines how many columns are bundled together to form each set. For example, in this case we are
    // permuting entries in the column with itself (self_permutation_column), so we choose just one
    constexpr static size_t COLUMNS_PER_SET = 1;

    /**
     * @brief If this method returns true on a row of values, then the inverse polynomial at this index. Otherwise the
     * value needs to be set to zero.
     *
     * @details If this is true then permutation takes place in this row
     *
     */
    template <typename AllEntities> static inline bool inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.enable_single_column_permutation == 1);
    }

    /**
     * @brief Get all the entities for the permutation when we don't need to update them
     *
     * @details The entities are returned as a tuple of references in the following order:
     * - The entity/polynomial used to store the product of the inverse values
     * - The entity/polynomial that switches on the subrelation of the permutation relation that ensures correctness of
     * the inverse polynomial
     * - The entity/polynomial that enables adding a tuple-generated value from the first set to the logderivative sum
     * subrelation
     * - The entity/polynomial that enables adding a tuple-generated value from the second set to the logderivative sum
     * subrelation
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the first set (N.B. ORDER IS IMPORTANT!)
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the second set (N.B. ORDER IS IMPORTANT!)
     *
     * @return All the entities needed for the permutation
     */
    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {

        return std::forward_as_tuple(
            in.single_permutation_inverses,      /* The polynomial containing the inverse product*/
            in.enable_single_column_permutation, /* The polynomial enabling the product check subrelation */
            in.enable_first_set_permutation,     /* Enables adding first set to the sum */
            in.enable_second_set_permutation,    /* Enables adding second set to the sum */
            in.self_permutation_column,          /* The first set column */
            in.self_permutation_column /* The second set column which in this case is the same as the first set column
                                        */
        );
    }

    /**
     * @brief Get all the entities for the permutation when need to update them
     *
     * @details The entities are returned as a tuple of references in the following order:
     * - The entity/polynomial used to store the product of the inverse values
     * - The entity/polynomial that switches on the subrelation of the permutation relation that ensures correctness of
     * the inverse polynomial
     * - The entity/polynomial that enables adding a tuple-generated value from the first set to the logderivative sum
     * subrelation
     * - The entity/polynomial that enables adding a tuple-generated value from the second set to the logderivative sum
     * subrelation
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the first set (N.B. ORDER IS IMPORTANT!)
     * - A sequence of COLUMNS_PER_SET entities/polynomials that represent the second set (N.B. ORDER IS IMPORTANT!)
     *
     * @return All the entities needed for the permutation
     */
    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(
            in.single_permutation_inverses,      /* The polynomial containing the inverse product*/
            in.enable_single_column_permutation, /* The polynomial enabling the product check subrelation */
            in.enable_first_set_permutation,     /* Enables adding first set to the sum */
            in.enable_second_set_permutation,    /* Enables adding second set to the sum */
            in.self_permutation_column,          /* The first set column */
            in.self_permutation_column /* The second set column which in this case is the same as the first set column
                                        */
        );
    }
};

#define DEFINE_IMPLEMENTATIONS_FOR_SETTINGS(RelationImplementation, flavor, Settings)                                  \
    template class RelationImplementation<Settings, flavor::FF>;                                                       \
    template <typename FF_> using RelationImplementation##Settings = RelationImplementation<Settings, FF_>;            \
    DEFINE_SUMCHECK_RELATION_CLASS(RelationImplementation##Settings, flavor);

#define DEFINE_IMPLEMENTATIONS_FOR_ALL_SETTINGS(RelationImplementation, flavor)                                        \
    DEFINE_IMPLEMENTATIONS_FOR_SETTINGS(RelationImplementation, flavor, ExampleTuplePermutationSettings);              \
    DEFINE_IMPLEMENTATIONS_FOR_SETTINGS(RelationImplementation, flavor, ExampleSameWirePermutationSettings);

#define DECLARE_IMPLEMENTATIONS_FOR_SETTINGS(RelationImplementation, flavor, Settings)                                 \
    extern template class RelationImplementation<Settings, flavor::FF>;                                                \
    template <typename FF_> using RelationImplementation##Settings = RelationImplementation<Settings, FF_>;            \
    DECLARE_SUMCHECK_RELATION_CLASS(RelationImplementation##Settings, flavor);

#define DECLARE_IMPLEMENTATIONS_FOR_ALL_SETTINGS(RelationImplementation, flavor)                                       \
    DECLARE_IMPLEMENTATIONS_FOR_SETTINGS(RelationImplementation, flavor, ExampleTuplePermutationSettings);             \
    DECLARE_IMPLEMENTATIONS_FOR_SETTINGS(RelationImplementation, flavor, ExampleSameWirePermutationSettings);
} // namespace proof_system::honk::sumcheck