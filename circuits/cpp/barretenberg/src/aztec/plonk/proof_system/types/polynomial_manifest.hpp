#pragma once

#include <array>
#include <string>

namespace waffle {

enum PolynomialSource { WITNESS, SELECTOR, PERMUTATION };

struct PolynomialDescriptor {
    constexpr PolynomialDescriptor(
        std::string_view a = "", std::string_view b = "", bool c = false, bool d = false, PolynomialSource e = WITNESS)
        : commitment_label(a)
        , polynomial_label(b)
        , is_linearised(c)
        , requires_shifted_evaluation(d)
        , source(e)
    {}
    constexpr PolynomialDescriptor(const PolynomialDescriptor& other)
        : commitment_label(other.commitment_label)
        , polynomial_label(other.polynomial_label)
        , is_linearised(other.is_linearised)
        , requires_shifted_evaluation(other.requires_shifted_evaluation)
        , source(other.source)
    {}
    constexpr PolynomialDescriptor& operator=(const PolynomialDescriptor& other)
    {
        commitment_label = other.commitment_label;
        polynomial_label = other.polynomial_label;
        is_linearised = other.is_linearised;
        requires_shifted_evaluation = other.requires_shifted_evaluation;
        source = other.source;
        return *this;
    };

    std::string_view commitment_label;
    std::string_view polynomial_label;
    bool is_linearised;
    bool requires_shifted_evaluation;
    PolynomialSource source;
};

static constexpr PolynomialDescriptor standard_polynomial_manifest[12]{
    PolynomialDescriptor("W_1", "w_1", false, false, WITNESS),             //
    PolynomialDescriptor("W_2", "w_2", false, false, WITNESS),             //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS),              //
    PolynomialDescriptor("Z", "z", true, true, WITNESS),                   //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR),             //
    PolynomialDescriptor("Q_2", "q_2", true, false, SELECTOR),             //
    PolynomialDescriptor("Q_3", "q_3", true, false, SELECTOR),             //
    PolynomialDescriptor("Q_M", "q_m", true, false, SELECTOR),             //
    PolynomialDescriptor("Q_C", "q_c", true, false, SELECTOR),             //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION), //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION), //
    PolynomialDescriptor("SIGMA_3", "sigma_3", true, false, PERMUTATION),  //
};

static constexpr PolynomialDescriptor mimc_polynomial_manifest[14]{
    PolynomialDescriptor("W_1", "w_1", false, false, WITNESS),                                //
    PolynomialDescriptor("W_2", "w_2", false, false, WITNESS),                                //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS),                                 //
    PolynomialDescriptor("Z", "z", true, true, WITNESS),                                      //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR),                                //
    PolynomialDescriptor("Q_2", "q_2", true, false, SELECTOR),                                //
    PolynomialDescriptor("Q_3", "q_3", true, false, SELECTOR),                                //
    PolynomialDescriptor("Q_M", "q_m", true, false, SELECTOR),                                //
    PolynomialDescriptor("Q_C", "q_c", true, false, SELECTOR),                                //
    PolynomialDescriptor("Q_MIMC_COEFFICIENT", "q_mimc_coefficient", false, false, SELECTOR), //
    PolynomialDescriptor("Q_MIMC_SELECTOR", "q_mimc_selector", true, false, SELECTOR),        //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION),                    //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION),                    //
    PolynomialDescriptor("SIGMA_3", "sigma_3", true, false, PERMUTATION),                     //
};

static constexpr PolynomialDescriptor turbo_polynomial_manifest[20]{
    PolynomialDescriptor("W_1", "w_1", false, true, WITNESS),                         //
    PolynomialDescriptor("W_2", "w_2", false, true, WITNESS),                         //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS),                         //
    PolynomialDescriptor("W_4", "w_4", false, true, WITNESS),                         //
    PolynomialDescriptor("Z", "z", true, true, WITNESS),                              //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_2", "q_2", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_3", "q_3", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_4", "q_4", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_5", "q_5", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_M", "q_m", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_C", "q_c", false, false, SELECTOR),                       //
    PolynomialDescriptor("Q_ARITHMETIC_SELECTOR", "q_arith", false, false, SELECTOR), //
    PolynomialDescriptor("Q_RANGE_SELECTOR", "q_range", true, false, SELECTOR),       //
    PolynomialDescriptor("Q_FIXED_BASE_SELECTOR", "q_ecc_1", false, false, SELECTOR), //
    PolynomialDescriptor("Q_LOGIC_SELECTOR", "q_logic", true, false, SELECTOR),       //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION),            //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION),            //
    PolynomialDescriptor("SIGMA_3", "sigma_3", false, false, PERMUTATION),            //
    PolynomialDescriptor("SIGMA_4", "sigma_4", true, false, PERMUTATION),             //
};

static constexpr PolynomialDescriptor plookup_polynomial_manifest[28]{
    PolynomialDescriptor("W_1", "w_1", false, true, WITNESS),                         //
    PolynomialDescriptor("W_2", "w_2", false, true, WITNESS),                         //
    PolynomialDescriptor("W_3", "w_3", false, true, WITNESS),                         //
    PolynomialDescriptor("W_4", "w_4", false, true, WITNESS),                         //
    PolynomialDescriptor("S", "s", false, true, WITNESS),                             //
    PolynomialDescriptor("Z", "z", true, true, WITNESS),                              //
    PolynomialDescriptor("Z_LOOKUP", "z_lookup", false, true, WITNESS),               //
    PolynomialDescriptor("Q_1", "q_1", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_2", "q_2", false, false, SELECTOR),                       //
    PolynomialDescriptor("Q_3", "q_3", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_4", "q_4", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_5", "q_5", true, false, SELECTOR),                        //
    PolynomialDescriptor("Q_M", "q_m", false, false, SELECTOR),                       //
    PolynomialDescriptor("Q_C", "q_c", false, false, SELECTOR),                       //
    PolynomialDescriptor("Q_ARITHMETIC_SELECTOR", "q_arith", false, false, SELECTOR), //
    PolynomialDescriptor("Q_RANGE_SELECTOR", "q_range", true, false, SELECTOR),       //
    PolynomialDescriptor("Q_FIXED_BASE_SELECTOR", "q_ecc_1", false, false, SELECTOR), //
    PolynomialDescriptor("Q_LOGIC_SELECTOR", "q_logic", true, false, SELECTOR),       //
    PolynomialDescriptor("SIGMA_1", "sigma_1", false, false, PERMUTATION),            //
    PolynomialDescriptor("SIGMA_2", "sigma_2", false, false, PERMUTATION),            //
    PolynomialDescriptor("SIGMA_3", "sigma_3", false, false, PERMUTATION),            //
    PolynomialDescriptor("SIGMA_4", "sigma_4", true, false, PERMUTATION),             //
    PolynomialDescriptor("TABLE_1", "table_value_1", false, true, PERMUTATION),       //
    PolynomialDescriptor("TABLE_2", "table_value_2", false, true, PERMUTATION),       //
    PolynomialDescriptor("TABLE_3", "table_value_3", false, true, PERMUTATION),       //
    PolynomialDescriptor("TABLE_4", "table_value_4", false, true, PERMUTATION),       //
    PolynomialDescriptor("TABLE_INDEX", "table_index", false, false, PERMUTATION),    //
    PolynomialDescriptor("TABLE_TYPE", "table_type", false, false, PERMUTATION),      //
};
} // namespace waffle