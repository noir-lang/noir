#pragma once

#include "barretenberg/proof_system/types/circuit_type.hpp"
#include <array>
#include <string>
#include <vector>

namespace bb::plonk {

enum PolynomialSource { WITNESS, SELECTOR, PERMUTATION, OTHER };

enum EvaluationType { NON_SHIFTED, SHIFTED };

enum PolynomialIndex {
    Q_1,
    Q_2,
    Q_3,
    Q_4,
    Q_5,
    Q_M,
    Q_C,
    Q_ARITHMETIC,
    Q_FIXED_BASE,
    Q_RANGE,
    Q_SORT,
    TABLE_1,
    TABLE_2,
    TABLE_3,
    TABLE_4,
    TABLE_INDEX,
    TABLE_TYPE,
    Q_ELLIPTIC,
    Q_AUX,
    SIGMA_1,
    SIGMA_2,
    SIGMA_3,
    SIGMA_4,
    ID_1,
    ID_2,
    ID_3,
    ID_4,
    W_1,
    W_2,
    W_3,
    W_4,
    S,
    Z,
    Z_LOOKUP,
    LAGRANGE_FIRST,
    LAGRANGE_LAST,
    // SUBGROUP_GENERATOR,
    MAX_NUM_POLYNOMIALS,
};

struct PolynomialDescriptor {
    constexpr PolynomialDescriptor(std::string_view commitment_label_ = "",
                                   std::string_view polynomial_label_ = "",
                                   bool requires_shifted_evaluation_ = false,
                                   PolynomialSource source_ = WITNESS,
                                   PolynomialIndex index_ = Q_1)
        : commitment_label(commitment_label_)
        , polynomial_label(polynomial_label_)
        , requires_shifted_evaluation(requires_shifted_evaluation_)
        , source(source_)
        , index(index_)
    {}
    constexpr PolynomialDescriptor(const PolynomialDescriptor& other)
        : commitment_label(other.commitment_label)
        , polynomial_label(other.polynomial_label)
        , requires_shifted_evaluation(other.requires_shifted_evaluation)
        , source(other.source)
        , index(other.index)
    {}
    constexpr PolynomialDescriptor& operator=(const PolynomialDescriptor& other)
    {
        commitment_label = other.commitment_label;
        polynomial_label = other.polynomial_label;
        requires_shifted_evaluation = other.requires_shifted_evaluation;
        source = other.source;
        index = other.index;
        return *this;
    };

    std::string_view commitment_label;
    std::string_view polynomial_label;
    bool requires_shifted_evaluation;
    PolynomialSource source;
    PolynomialIndex index;
};

static constexpr size_t STANDARD_MANIFEST_SIZE = 12;
static constexpr PolynomialDescriptor standard_polynomial_manifest[STANDARD_MANIFEST_SIZE]{
    PolynomialDescriptor("W_1", "w_1", false, WITNESS, W_1),                 //
    PolynomialDescriptor("W_2", "w_2", false, WITNESS, W_2),                 //
    PolynomialDescriptor("W_3", "w_3", false, WITNESS, W_3),                 //
    PolynomialDescriptor("Z_PERM", "z_perm", true, WITNESS, Z),              //
    PolynomialDescriptor("Q_1", "q_1", false, SELECTOR, Q_1),                //
    PolynomialDescriptor("Q_2", "q_2", false, SELECTOR, Q_2),                //
    PolynomialDescriptor("Q_3", "q_3", false, SELECTOR, Q_3),                //
    PolynomialDescriptor("Q_M", "q_m", false, SELECTOR, Q_M),                //
    PolynomialDescriptor("Q_C", "q_c", false, SELECTOR, Q_C),                //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, PERMUTATION, SIGMA_1), //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, PERMUTATION, SIGMA_2), //
    PolynomialDescriptor("SIGMA_3", "sigma_3", false, PERMUTATION, SIGMA_3), //
};

static constexpr size_t ULTRA_MANIFEST_SIZE = 30;
static constexpr PolynomialDescriptor ultra_polynomial_manifest[ULTRA_MANIFEST_SIZE]{
    PolynomialDescriptor("W_1", "w_1", true, WITNESS, W_1),                         //
    PolynomialDescriptor("W_2", "w_2", true, WITNESS, W_2),                         //
    PolynomialDescriptor("W_3", "w_3", true, WITNESS, W_3),                         //
    PolynomialDescriptor("W_4", "w_4", true, WITNESS, W_4),                         //
    PolynomialDescriptor("S", "s", true, WITNESS, S),                               //
    PolynomialDescriptor("Z_PERM", "z_perm", true, WITNESS, Z),                     //
    PolynomialDescriptor("Z_LOOKUP", "z_lookup", true, WITNESS, Z_LOOKUP),          //
    PolynomialDescriptor("Q_1", "q_1", false, SELECTOR, Q_1),                       //
    PolynomialDescriptor("Q_2", "q_2", false, SELECTOR, Q_2),                       //
    PolynomialDescriptor("Q_3", "q_3", false, SELECTOR, Q_3),                       //
    PolynomialDescriptor("Q_4", "q_4", false, SELECTOR, Q_4),                       //
    PolynomialDescriptor("Q_M", "q_m", false, SELECTOR, Q_M),                       //
    PolynomialDescriptor("Q_C", "q_c", false, SELECTOR, Q_C),                       //
    PolynomialDescriptor("Q_ARITHMETIC", "q_arith", false, SELECTOR, Q_ARITHMETIC), //
    PolynomialDescriptor("Q_SORT", "q_sort", false, SELECTOR, Q_SORT),              //
    PolynomialDescriptor("Q_ELLIPTIC", "q_elliptic", false, SELECTOR, Q_ELLIPTIC),  //
    PolynomialDescriptor("Q_AUX", "q_aux", false, SELECTOR, Q_AUX),                 //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, PERMUTATION, SIGMA_1),        //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, PERMUTATION, SIGMA_2),        //
    PolynomialDescriptor("SIGMA_3", "sigma_3", false, PERMUTATION, SIGMA_3),        //
    PolynomialDescriptor("SIGMA_4", "sigma_4", false, PERMUTATION, SIGMA_4),        //
    PolynomialDescriptor("TABLE_1", "table_value_1", true, SELECTOR, TABLE_1),      //
    PolynomialDescriptor("TABLE_2", "table_value_2", true, SELECTOR, TABLE_2),      //
    PolynomialDescriptor("TABLE_3", "table_value_3", true, SELECTOR, TABLE_3),      //
    PolynomialDescriptor("TABLE_4", "table_value_4", true, SELECTOR, TABLE_4),      //
    PolynomialDescriptor("TABLE_TYPE", "table_type", false, SELECTOR, TABLE_TYPE),  //
    PolynomialDescriptor("ID_1", "id_1", false, PERMUTATION, ID_1),                 //
    PolynomialDescriptor("ID_2", "id_2", false, PERMUTATION, ID_2),                 //
    PolynomialDescriptor("ID_3", "id_3", false, PERMUTATION, ID_3),                 //
    PolynomialDescriptor("ID_4", "id_4", false, PERMUTATION, ID_4),                 //
};

// Simple class allowing for access to a polynomial manifest based on composer type
class PolynomialManifest {
    // TODO(luke): make this object iterable, i.e. compatible with range-based for loop
  private:
    std::vector<PolynomialDescriptor> manifest;

  public:
    PolynomialManifest() {}

    PolynomialManifest(CircuitType circuit_type)
    {
        switch (circuit_type) {
        case CircuitType::STANDARD: {
            std::copy(standard_polynomial_manifest,
                      standard_polynomial_manifest + STANDARD_MANIFEST_SIZE,
                      std::back_inserter(manifest));
            break;
        };
        case CircuitType::ULTRA: {
            std::copy(ultra_polynomial_manifest,
                      ultra_polynomial_manifest + ULTRA_MANIFEST_SIZE,
                      std::back_inserter(manifest));
            break;
        };
        default: {
            // TODO(luke): reinstate this. Was getting "use of undeclared identifier" error for 'throw_or_abort'.
            // throw_or_abort("Received invalid composer type");
        }
        };
    }

    const std::vector<PolynomialDescriptor>& get() const { return manifest; };

    size_t size() const { return manifest.size(); }

    PolynomialDescriptor operator[](size_t index) const { return manifest[index]; }
};

// This class constructs and provides access to a full list of pre-computed
// polynomial IDs based on the composer type. This is used, for example, for
// serialization of the pre-computed portion of the proving key. The list is
// comprised of IDs corresponding to: the selector polynomials (monomial and
// coset fft forms) and the permutation polynomials (monomial, coset fft and
// lagrange forms).
class PrecomputedPolyList {

  private:
    std::vector<std::string> precomputed_poly_ids;

  public:
    // Upon construction, build the vector of precomputed poly ID strings based on the manifest
    PrecomputedPolyList(CircuitType circuit_type)
    {
        PolynomialManifest manifest(circuit_type);

        for (size_t i = 0; i < manifest.size(); ++i) {
            std::string label = std::string(manifest[i].polynomial_label);
            PolynomialSource source = manifest[i].source;

            switch (source) {
            case PolynomialSource::WITNESS: // no witness polys are precomputed
                break;
            case PolynomialSource::SELECTOR: // monomial and fft
                precomputed_poly_ids.emplace_back(label);
                precomputed_poly_ids.emplace_back(label + "_fft");
                // Store all lagrange forms of selector polynomials for ultra
                if (circuit_type == CircuitType::ULTRA) {
                    precomputed_poly_ids.emplace_back(label + "_lagrange");
                }
                break;
            case PolynomialSource::PERMUTATION: // monomial, fft, and lagrange
                precomputed_poly_ids.emplace_back(label);
                precomputed_poly_ids.emplace_back(label + "_fft");
                precomputed_poly_ids.emplace_back(label + "_lagrange");
                break;
            case PolynomialSource::OTHER:
                break;
            }
        }
    }

    size_t size() const { return precomputed_poly_ids.size(); }

    std::string operator[](size_t index) const { return precomputed_poly_ids[index]; }
};

} // namespace bb::plonk