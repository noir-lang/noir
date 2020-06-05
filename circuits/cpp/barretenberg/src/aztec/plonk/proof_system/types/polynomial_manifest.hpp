#pragma once

#include <array>
#include <string>

namespace waffle {

enum PolynomialSource { WITNESS, SELECTOR, PERMUTATION };

enum PolynomialIndex {
    Q_1,
    Q_2,
    Q_3,
    Q_4,
    Q_5,
    Q_M,
    Q_C,
    Q_ARITHMETIC_SELECTOR,
    Q_FIXED_BASE_SELECTOR,
    Q_RANGE_SELECTOR,
    Q_LOGIC_SELECTOR,
    TABLE_1,
    TABLE_2,
    TABLE_3,
    TABLE_4,
    TABLE_INDEX,
    TABLE_TYPE,
    Q_MIMC_COEFFICIENT,
    Q_MIMC_SELECTOR,
    Q_ELLIPTIC,
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
    // LAGRANGE_FIRST,
    // LAGRANGE_LAST,
    // SUBGROUP_GENERATOR,
    MAX_NUM_POLYNOMIALS,
};

struct PolynomialDescriptor {
    constexpr PolynomialDescriptor(std::string_view a = "",
                                   std::string_view b = "",
                                   bool c = false,
                                   bool d = false,
                                   PolynomialSource e = WITNESS,
                                   PolynomialIndex f = Q_1)
        : commitment_label(a)
        , polynomial_label(b)
        , is_linearised(c)
        , requires_shifted_evaluation(d)
        , source(e)
        , index(f)
    {}
    constexpr PolynomialDescriptor(const PolynomialDescriptor& other)
        : commitment_label(other.commitment_label)
        , polynomial_label(other.polynomial_label)
        , is_linearised(other.is_linearised)
        , requires_shifted_evaluation(other.requires_shifted_evaluation)
        , source(other.source)
        , index(other.index)
    {}
    constexpr PolynomialDescriptor& operator=(const PolynomialDescriptor& other)
    {
        commitment_label = other.commitment_label;
        polynomial_label = other.polynomial_label;
        is_linearised = other.is_linearised;
        requires_shifted_evaluation = other.requires_shifted_evaluation;
        source = other.source;
        index = other.index;
        return *this;
    };

    std::string_view commitment_label;
    std::string_view polynomial_label;
    bool is_linearised;
    bool requires_shifted_evaluation;
    PolynomialSource source;
    PolynomialIndex index;
};

static constexpr PolynomialDescriptor standard_polynomial_manifest[12]{
    PolynomialDescriptor("W_1", "w_1", false, false, WITNESS, W_1),                 //
    PolynomialDescriptor("W_2", "w_2", false, false, WITNESS, W_2),                 //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS, W_3),                  //
    PolynomialDescriptor("Z", "z", true, true, WITNESS, Z),                         //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR, Q_1),                 //
    PolynomialDescriptor("Q_2", "q_2", true, false, SELECTOR, Q_2),                 //
    PolynomialDescriptor("Q_3", "q_3", true, false, SELECTOR, Q_3),                 //
    PolynomialDescriptor("Q_M", "q_m", true, false, SELECTOR, Q_M),                 //
    PolynomialDescriptor("Q_C", "q_c", true, false, SELECTOR, Q_C),                 //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION, SIGMA_1), //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION, SIGMA_2), //
    PolynomialDescriptor("SIGMA_3", "sigma_3", true, false, PERMUTATION, SIGMA_3),  //
};

static constexpr PolynomialDescriptor mimc_polynomial_manifest[14]{
    PolynomialDescriptor("W_1", "w_1", false, false, WITNESS, W_1),                                               //
    PolynomialDescriptor("W_2", "w_2", false, false, WITNESS, W_2),                                               //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS, W_3),                                                //
    PolynomialDescriptor("Z", "z", true, true, WITNESS, Z),                                                       //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR, Q_1),                                               //
    PolynomialDescriptor("Q_2", "q_2", true, false, SELECTOR, Q_2),                                               //
    PolynomialDescriptor("Q_3", "q_3", true, false, SELECTOR, Q_3),                                               //
    PolynomialDescriptor("Q_M", "q_m", true, false, SELECTOR, Q_M),                                               //
    PolynomialDescriptor("Q_C", "q_c", true, false, SELECTOR, Q_C),                                               //
    PolynomialDescriptor("Q_MIMC_COEFFICIENT", "q_mimc_coefficient", false, false, SELECTOR, Q_MIMC_COEFFICIENT), //
    PolynomialDescriptor("Q_MIMC_SELECTOR", "q_mimc_selector", true, false, SELECTOR, Q_MIMC_SELECTOR),           //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION, SIGMA_1),                               //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION, SIGMA_2),                               //
    PolynomialDescriptor("SIGMA_3", "sigma_3", true, false, PERMUTATION, SIGMA_3),                                //
};

static constexpr PolynomialDescriptor turbo_polynomial_manifest[20]{
    PolynomialDescriptor("W_1", "w_1", false, true, WITNESS, W_1),                                           //
    PolynomialDescriptor("W_2", "w_2", false, true, WITNESS, W_2),                                           //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS, W_3),                                           //
    PolynomialDescriptor("W_4", "w_4", false, true, WITNESS, W_4),                                           //
    PolynomialDescriptor("Z", "z", true, true, WITNESS, Z),                                                  //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR, Q_1),                                          //
    PolynomialDescriptor("Q_2", "q_2", true, false, SELECTOR, Q_2),                                          //
    PolynomialDescriptor("Q_3", "q_3", true, false, SELECTOR, Q_3),                                          //
    PolynomialDescriptor("Q_4", "q_4", true, false, SELECTOR, Q_4),                                          //
    PolynomialDescriptor("Q_5", "q_5", true, false, SELECTOR, Q_5),                                          //
    PolynomialDescriptor("Q_M", "q_m", true, false, SELECTOR, Q_M),                                          //
    PolynomialDescriptor("Q_C", "q_c", false, false, SELECTOR, Q_C),                                         //
    PolynomialDescriptor("Q_ARITHMETIC_SELECTOR", "q_arith", false, false, SELECTOR, Q_ARITHMETIC_SELECTOR), //
    PolynomialDescriptor("Q_RANGE_SELECTOR", "q_range", true, false, SELECTOR, Q_RANGE_SELECTOR),            //
    PolynomialDescriptor("Q_FIXED_BASE_SELECTOR", "q_ecc_1", false, false, SELECTOR, Q_FIXED_BASE_SELECTOR), //
    PolynomialDescriptor("Q_LOGIC_SELECTOR", "q_logic", true, false, SELECTOR, Q_LOGIC_SELECTOR),            //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION, SIGMA_1),                          //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION, SIGMA_2),                          //
    PolynomialDescriptor("SIGMA_3", "sigma_3", false, false, PERMUTATION, SIGMA_3),                          //
    PolynomialDescriptor("SIGMA_4", "sigma_4", true, false, PERMUTATION, SIGMA_4),                           //
};

static constexpr PolynomialDescriptor plookup_polynomial_manifest[29]{
    PolynomialDescriptor("W_1", "w_1", false, true, WITNESS, W_1),                                           //
    PolynomialDescriptor("W_2", "w_2", false, true, WITNESS, W_2),                                           //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS, W_3),                                           //
    PolynomialDescriptor("W_4", "w_4", false, true, WITNESS, W_4),                                           //
    PolynomialDescriptor("S", "s", false, true, WITNESS, S),                                                 //
    PolynomialDescriptor("Z", "z", true, true, WITNESS, Z),                                                  //
    PolynomialDescriptor("Z_LOOKUP", "z_lookup", false, true, WITNESS, Z_LOOKUP),                            //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR, Q_1),                                          //
    PolynomialDescriptor("Q_2", "q_2", false, false, SELECTOR, Q_2),                                         //
    PolynomialDescriptor("Q_3", "q_3", false, false, SELECTOR, Q_3),                                         //
    PolynomialDescriptor("Q_4", "q_4", false, false, SELECTOR, Q_4),                                         //
    PolynomialDescriptor("Q_5", "q_5", false, false, SELECTOR, Q_5),                                         //
    PolynomialDescriptor("Q_M", "q_m", false, false, SELECTOR, Q_M),                                         //
    PolynomialDescriptor("Q_C", "q_c", false, false, SELECTOR, Q_C),                                         //
    PolynomialDescriptor("Q_ARITHMETIC_SELECTOR", "q_arith", false, false, SELECTOR, Q_ARITHMETIC_SELECTOR), //
    PolynomialDescriptor("Q_RANGE_SELECTOR", "q_range", true, false, SELECTOR, Q_RANGE_SELECTOR),            //
    PolynomialDescriptor("Q_FIXED_BASE_SELECTOR", "q_ecc_1", false, false, SELECTOR, Q_FIXED_BASE_SELECTOR), //
    PolynomialDescriptor("Q_LOGIC_SELECTOR", "q_logic", true, false, SELECTOR, Q_LOGIC_SELECTOR),            //
    PolynomialDescriptor("Q_ELLIPTIC", "q_elliptic", true, false, SELECTOR, Q_ELLIPTIC),                     //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION, SIGMA_1),                          //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION, SIGMA_2),                          //
    PolynomialDescriptor("SIGMA_3", "sigma_3", false, false, PERMUTATION, SIGMA_3),                          //
    PolynomialDescriptor("SIGMA_4", "sigma_4", true, false, PERMUTATION, SIGMA_4),                           //
    PolynomialDescriptor("TABLE_1", "table_value_1", false, true, SELECTOR, TABLE_1),                        //
    PolynomialDescriptor("TABLE_2", "table_value_2", false, true, SELECTOR, TABLE_2),                        //
    PolynomialDescriptor("TABLE_3", "table_value_3", false, true, SELECTOR, TABLE_3),                        //
    PolynomialDescriptor("TABLE_4", "table_value_4", false, true, SELECTOR, TABLE_4),                        //
    PolynomialDescriptor("TABLE_INDEX", "table_index", false, false, SELECTOR, TABLE_INDEX),                 //
    PolynomialDescriptor("TABLE_TYPE", "table_type", false, false, SELECTOR, TABLE_TYPE),                    //
};

static constexpr PolynomialDescriptor genperm_polynomial_manifest[24]{
    PolynomialDescriptor("W_1", "w_1", false, true, WITNESS, W_1),                                           //
    PolynomialDescriptor("W_2", "w_2", false, true, WITNESS, W_2),                                           //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS, W_3),                                           //
    PolynomialDescriptor("W_4", "w_4", false, true, WITNESS, W_4),                                           //
    PolynomialDescriptor("Z", "z", true, true, WITNESS, Z),                                                  //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR, Q_1),                                          //
    PolynomialDescriptor("Q_2", "q_2", true, false, SELECTOR, Q_2),                                          //
    PolynomialDescriptor("Q_3", "q_3", true, false, SELECTOR, Q_3),                                          //
    PolynomialDescriptor("Q_4", "q_4", true, false, SELECTOR, Q_4),                                          //
    PolynomialDescriptor("Q_5", "q_5", true, false, SELECTOR, Q_5),                                          //
    PolynomialDescriptor("Q_M", "q_m", true, false, SELECTOR, Q_M),                                          //
    PolynomialDescriptor("Q_C", "q_c", false, false, SELECTOR, Q_C),                                         //
    PolynomialDescriptor("Q_ARITHMETIC_SELECTOR", "q_arith", false, false, SELECTOR, Q_ARITHMETIC_SELECTOR), //
    PolynomialDescriptor("Q_RANGE_SELECTOR", "q_range", true, false, SELECTOR, Q_RANGE_SELECTOR),            //
    PolynomialDescriptor("Q_FIXED_BASE_SELECTOR", "q_ecc_1", false, false, SELECTOR, Q_FIXED_BASE_SELECTOR), //
    PolynomialDescriptor("Q_LOGIC_SELECTOR", "q_logic", true, false, SELECTOR, Q_LOGIC_SELECTOR),            //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION, SIGMA_1),                          //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION, SIGMA_2),                          //
    PolynomialDescriptor("SIGMA_3", "sigma_3", false, false, PERMUTATION, SIGMA_3),                          //
    PolynomialDescriptor("SIGMA_4", "sigma_4", true, false, PERMUTATION, SIGMA_4),                           //
    PolynomialDescriptor("ID_1", "id_1", false, false, PERMUTATION, ID_1),                                   //
    PolynomialDescriptor("ID_2", "id_2", false, false, PERMUTATION, ID_2),                                   //
    PolynomialDescriptor("ID_3", "id_3", false, false, PERMUTATION, ID_3),                                   //
    PolynomialDescriptor("ID_4", "id_4", false, false, PERMUTATION, ID_4),                                   //
};

} // namespace waffle