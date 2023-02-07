#include "ultra_composer.hpp"

#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <numeric/bitop/get_msb.hpp>
#include <algorithm>
#include <optional>
#include <plonk/proof_system/widgets/transition_widgets/plookup_arithmetic_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/turbo_logic_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/genperm_sort_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/elliptic_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/plookup_auxiliary_widget.hpp>
#include <plonk/proof_system/widgets/random_widgets/permutation_widget.hpp>
#include <plonk/proof_system/widgets/random_widgets/plookup_widget.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>
#include <srs/reference_string/file_reference_string.hpp>

#include "plookup_tables/types.hpp"
#include "plookup_tables/plookup_tables.hpp"
#include "plookup_tables/aes128.hpp"
#include "plookup_tables/sha256.hpp"

#ifndef NO_TBB
#include <tbb/atomic.h>
#include <tbb/tbb.h>
#include <tbb/parallel_for.h>
#include <tbb/blocked_range.h>
#include <execution>
#endif

using namespace barretenberg;

namespace waffle {

#define ULTRA_SELECTOR_REFS                                                                                            \
    auto& q_m = selectors[UltraSelectors::QM];                                                                         \
    auto& q_c = selectors[UltraSelectors::QC];                                                                         \
    auto& q_1 = selectors[UltraSelectors::Q1];                                                                         \
    auto& q_2 = selectors[UltraSelectors::Q2];                                                                         \
    auto& q_3 = selectors[UltraSelectors::Q3];                                                                         \
    auto& q_4 = selectors[UltraSelectors::Q4];                                                                         \
    auto& q_arith = selectors[UltraSelectors::QARITH];                                                                 \
    auto& q_fixed_base = selectors[UltraSelectors::QFIXED];                                                            \
    auto& q_sort = selectors[UltraSelectors::QSORT];                                                                   \
    auto& q_elliptic = selectors[UltraSelectors::QELLIPTIC];                                                           \
    auto& q_aux = selectors[UltraSelectors::QAUX];                                                                     \
    auto& q_lookup_type = selectors[UltraSelectors::QLOOKUPTYPE];

std::vector<ComposerBase::SelectorProperties> ultra_selector_properties()
{
    std::vector<ComposerBase::SelectorProperties> result{
        { "q_m", true },     { "q_c", true },         { "q_1", true },      { "q_2", true },
        { "q_3", true },     { "q_4", false },        { "q_arith", false }, { "q_fixed_base", false },
        { "q_sort", false }, { "q_elliptic", false }, { "q_aux", false },   { "table_type", true },
    };
    return result;
}

UltraComposer::UltraComposer()
    : UltraComposer("../srs_db/ignition", 0)
{}

UltraComposer::UltraComposer(std::string const& crs_path, const size_t size_hint)
    : UltraComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory(crs_path)), size_hint){};

UltraComposer::UltraComposer(std::shared_ptr<ReferenceStringFactory> const& crs_factory, const size_t size_hint)
    : ComposerBase(crs_factory, UltraSelectors::NUM, size_hint, ultra_selector_properties())
{
    w_l.reserve(size_hint);
    w_r.reserve(size_hint);
    w_o.reserve(size_hint);
    w_4.reserve(size_hint);
    zero_idx = put_constant_variable(0);
    tau.insert({ DUMMY_TAG, DUMMY_TAG });
}

UltraComposer::UltraComposer(std::shared_ptr<proving_key> const& p_key,
                             std::shared_ptr<verification_key> const& v_key,
                             size_t size_hint)
    : ComposerBase(p_key, v_key, UltraSelectors::NUM, size_hint, ultra_selector_properties())
{
    w_r.reserve(size_hint);
    w_4.reserve(size_hint);
    zero_idx = put_constant_variable(0);
    tau.insert({ DUMMY_TAG, DUMMY_TAG });
}

/**
 * @brief Create an addition gate, where in.a * in.a_scaling + in.b * in.b_scaling + in.c * in.c_scaling +
 * in.const_scaling = 0
 *
 * @details Arithmetic selector is set to 1, all other gate selectors are 0. Mutliplication selector is set to 0
 *
 * @param in A structure with variable indexes and selector values for the gate.
 */
void UltraComposer::create_add_gate(const add_triple& in)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(0);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}

/**
 * @brief Create a big addition gate, where in.a * in.a_scaling + in.b * in.b_scaling + in.c * in.c_scaling + in.d *
 * in.d_scaling + in.const_scaling = 0. If include_next_gate_w_4 is enabled, then thes sum also adds the value of the
 * 4-th witness at the next index.
 *
 * @param in Structure with variable indexes and wire selector values
 * @param include_next_gate_w_4 Switches on/off the addition of w_4 at the next index
 */
void UltraComposer::create_big_add_gate(const add_quad& in, const bool include_next_gate_w_4)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(0);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(include_next_gate_w_4 ? 2 : 1);
    q_4.emplace_back(in.d_scaling);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}

/**
 * @brief A legacy method that was used to extract a bit from c-4d by using gate selectors in the Turboplonk, but is
 * simulated here for ultraplonk
 *
 * @param in Structure with variables and witness selector values
 */
void UltraComposer::create_big_add_gate_with_bit_extraction(const add_quad& in)
{
    // This method is an artifact of a turbo plonk feature that implicitly extracts
    // a high or low bit from a base-4 quad and adds it into the arithmetic gate relationship.
    // This has been removed in the plookup composer due to it's infrequent use not being worth the extra
    // cost incurred by the prover for the extra field muls required.

    // We have wires a, b, c, d, where
    // a + b + c + d + 6 * (extracted bit) = 0
    // (extracted bit) is the high bit pulled from c - 4d

    assert_valid_variables({ in.a, in.b, in.c, in.d });

    const uint256_t quad = get_variable(in.c) - get_variable(in.d) * 4;
    const auto lo_bit = quad & uint256_t(1);
    const auto hi_bit = (quad & uint256_t(2)) >> 1;
    const auto lo_idx = add_variable(lo_bit);
    const auto hi_idx = add_variable(hi_bit);
    // lo + hi * 2 - c + 4 * d = 0
    create_big_add_gate({
        lo_idx,
        hi_idx,
        in.c,
        in.d,
        1,
        2,
        -1,
        4,
        0,
    });

    // create temporary variable t = in.a * in.a_scaling + 6 * hi_bit
    const auto t = get_variable(in.a) * in.a_scaling + fr(hi_bit) * 6;
    const auto t_idx = add_variable(t);
    create_big_add_gate({
        in.a,
        hi_idx,
        t_idx,
        zero_idx,
        in.a_scaling,
        6,
        -1,
        0,
        0,
    });
    // (t = a + 6 * hi_bit) + b + c + d = 0
    create_big_add_gate({
        t_idx,
        in.b,
        in.c,
        in.d,
        1,
        in.b_scaling,
        in.c_scaling,
        in.d_scaling,
        in.const_scaling,
    });
}
/**
 * @brief Create a basic multiplication gate q_m * a * b + q_1 * a + q_2 * b + q_3 * c + q_4 * d + q_c = 0 (q_arith = 1)
 *
 * @param in Structure containing variables and witness selectors
 */
void UltraComposer::create_big_mul_gate(const mul_quad& in)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(in.d_scaling);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}

// Creates a width-4 addition gate, where the fourth witness must be a boolean.
// Can be used to normalize a 32-bit addition
void UltraComposer::create_balanced_add_gate(const add_quad& in)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(0);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(in.d_scaling);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
    // Why 3? TODO: return to this
    create_new_range_constraint(in.d, 3);
}
/**
 * @brief Create a multiplication gate with q_m * a * b + q_3 * c + q_const = 0
 *
 * @details q_arith == 1
 *
 * @param in Structure containing variables and witness selectors
 */
void UltraComposer::create_mul_gate(const mul_triple& in)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}
/**
 * @brief Generate an arithmetic gate equivalent to x^2 - x = 0, which forces x to be 0 or 1
 *
 * @param variable_index Variable which needs to be constrained
 */
void UltraComposer::create_bool_gate(const uint32_t variable_index)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ variable_index });

    w_l.emplace_back(variable_index);
    w_r.emplace_back(variable_index);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(1);
    q_1.emplace_back(-1);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_sort.emplace_back(0);

    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}

/**
 * @brief A plonk gate with disabled (set to zero) fourth wire. q_m * a * b + q_1 * a + q_2 * b + q_3 * c + q_const = 0
 *
 * @param in Structure containing variables and witness selectors
 */
void UltraComposer::create_poly_gate(const poly_triple& in)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(in.q_m);
    q_1.emplace_back(in.q_l);
    q_2.emplace_back(in.q_r);
    q_3.emplace_back(in.q_o);
    q_c.emplace_back(in.q_c);
    q_sort.emplace_back(0);

    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}

// adds a grumpkin point, from a 2-bit lookup table, into an accumulator point
void UltraComposer::create_fixed_group_add_gate(const fixed_group_add_quad& in)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);

    q_1.emplace_back(in.q_x_1);
    q_2.emplace_back(in.q_x_2);
    q_3.emplace_back(in.q_y_1);
    q_fixed_base.emplace_back(in.q_y_2);

    q_arith.emplace_back(0);
    q_4.emplace_back(0);
    q_m.emplace_back(0);
    q_c.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_sort.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}

// adds a grumpkin point into an accumulator, while also initializing the accumulator
void UltraComposer::create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in,
                                                          const fixed_group_init_quad& init)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);

    // Initialization differs slightly with that in TurboComposer.
    q_m.emplace_back(init.q_y_1);
    q_c.emplace_back(init.q_y_2);

    q_1.emplace_back(in.q_x_1);
    q_2.emplace_back(in.q_x_2);
    q_3.emplace_back(in.q_y_1);
    q_fixed_base.emplace_back(in.q_y_2);

    q_4.emplace_back(0);
    q_aux.emplace_back(0);
    q_arith.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_sort.emplace_back(0);
    q_elliptic.emplace_back(0);

    ++num_gates;
}

void UltraComposer::create_fixed_group_add_gate_final(const add_quad& in)
{
    create_big_add_gate(in);
}
/**
 * @brief Create an elliptic curve addition gate
 *
 * @details x and y are defined over scalar field. Addition can handle applying the curve endomorphism to one of the
 * points being summed at the time of addition.
 *
 * @param in Elliptic curve point addition gate parameters, including the the affine coordinates of the two points being
 * added, the resulting point coordinates and the selector values that describe whether the endomorphism is used on the
 * second point and whether it is negated.
 */
void UltraComposer::create_ecc_add_gate(const ecc_add_gate& in)
{
    /**
     * | 1  | 2  | 3  | 4  |
     * | a1 | a2 | x1 | y1 |
     * | x2 | y2 | x3 | y3 |
     * | -- | -- | x4 | y4 |
     *
     **/

    ULTRA_SELECTOR_REFS

    assert_valid_variables({ in.x1, in.x2, in.x3, in.y1, in.y2, in.y3 });

    bool can_fuse_into_previous_gate = true;
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (w_r[num_gates - 1] == in.x1);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (w_o[num_gates - 1] == in.y1);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (q_3[num_gates - 1] == 0);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (q_4[num_gates - 1] == 0);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (q_1[num_gates - 1] == 0);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (q_arith[num_gates - 1] == 0);

    if (can_fuse_into_previous_gate) {

        q_3[num_gates - 1] = in.endomorphism_coefficient;
        q_4[num_gates - 1] = in.endomorphism_coefficient.sqr();
        q_1[num_gates - 1] = in.sign_coefficient;
        q_elliptic[num_gates - 1] = 1;
    } else {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(in.x1);
        w_o.emplace_back(in.y1);
        w_4.emplace_back(zero_idx);
        q_3.emplace_back(in.endomorphism_coefficient);
        q_4.emplace_back(in.endomorphism_coefficient.sqr());
        q_1.emplace_back(in.sign_coefficient);

        q_arith.emplace_back(0);
        q_2.emplace_back(0);
        q_m.emplace_back(0);
        q_c.emplace_back(0);
        q_fixed_base.emplace_back(0);
        q_sort.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_elliptic.emplace_back(1);
        q_aux.emplace_back(0);
        ++num_gates;
    }

    w_l.emplace_back(in.x2);
    w_4.emplace_back(in.y2);
    w_r.emplace_back(in.x3);
    w_o.emplace_back(in.y3);
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_arith.emplace_back(0);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}

/**
 * @brief Add a gate equating a particular witness to a constant, fixing it the value
 *
 * @param witness_index The index of the witness we are fixing
 * @param witness_value The value we are fixing it to
 */
void UltraComposer::fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
{
    ULTRA_SELECTOR_REFS
    assert_valid_variables({ witness_index });

    w_l.emplace_back(witness_index);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(0);
    q_1.emplace_back(1);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(-witness_value);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_aux.emplace_back(0);
    ++num_gates;
}

uint32_t UltraComposer::put_constant_variable(const barretenberg::fr& variable)
{
    if (constant_variable_indices.contains(variable)) {
        return constant_variable_indices.at(variable);
    } else {
        uint32_t variable_index = add_variable(variable);
        fix_witness(variable_index, variable);
        constant_variable_indices.insert({ variable, variable_index });
        return variable_index;
    }
}

void UltraComposer::add_table_column_selector_poly_to_proving_key(polynomial& selector_poly_lagrange_form,
                                                                  const std::string& tag)
{
    polynomial selector_poly_lagrange_form_copy(selector_poly_lagrange_form, circuit_proving_key->small_domain.size);

    selector_poly_lagrange_form.ifft(circuit_proving_key->small_domain);
    auto& selector_poly_coeff_form = selector_poly_lagrange_form;

    polynomial selector_poly_coset_form(selector_poly_coeff_form, circuit_proving_key->circuit_size * 4);
    selector_poly_coset_form.coset_fft(circuit_proving_key->large_domain);

    circuit_proving_key->polynomial_cache.put(tag, std::move(selector_poly_coeff_form));
    circuit_proving_key->polynomial_cache.put(tag + "_lagrange", std::move(selector_poly_lagrange_form_copy));
    circuit_proving_key->polynomial_cache.put(tag + "_fft", std::move(selector_poly_coset_form));
}

std::shared_ptr<proving_key> UltraComposer::compute_proving_key()
{
    ULTRA_SELECTOR_REFS;

    /**
     * First of all, add the gates related to ROM arrays and range lists.
     * Note that the total number of rows in an UltraPlonk program can be divided as following:
     *  1. arithmetic gates:  n_computation (includes all computation gates)
     *  2. rom/memory gates:  n_rom
     *  3. range list gates:  n_range
     *  4. public inputs:     n_pub
     *
     * Now we have two variables referred to as `n` in the code:
     *  1. ComposerBase::n => refers to the size of the witness of a given program,
     *  2. proving_key::n => the next power of two ≥ total witness size.
     *
     * In this case, we have composer.num_gates = n_computation before we execute the following two functions.
     * After these functions are executed, the composer's `n` is incremented to include the ROM
     * and range list gates. Therefore we have:
     * composer.num_gates = n_computation + n_rom + n_range.
     *
     * Its necessary to include the (n_rom + n_range) gates at this point because if we already have a
     * proving key, and we just return it without including these ROM and range list gates, the overall
     * circuit size would not be correct (resulting in the code crashing while performing FFT operations).
     *
     * Therefore, we introduce a boolean flag `circuit_finalised` here. Once we add the rom and range gates,
     * our circuit is finalised, and we must not to execute these functions again.
     */
    if (!circuit_finalised) {
        process_ROM_arrays(public_inputs.size());
        process_range_lists();
        circuit_finalised = true;
    }

    if (circuit_proving_key) {
        return circuit_proving_key;
    }

    ASSERT(num_gates == q_m.size());
    ASSERT(num_gates == q_c.size());
    ASSERT(num_gates == q_1.size());
    ASSERT(num_gates == q_2.size());
    ASSERT(num_gates == q_3.size());
    ASSERT(num_gates == q_4.size());
    ASSERT(num_gates == q_arith.size());
    ASSERT(num_gates == q_fixed_base.size());
    ASSERT(num_gates == q_elliptic.size());
    ASSERT(num_gates == q_sort.size());
    ASSERT(num_gates == q_lookup_type.size());
    ASSERT(num_gates == q_aux.size());

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    // Compute selector polynomials and appropriate fft versions and put them in the proving key
    ComposerBase::compute_proving_key_base(type, tables_size + lookups_size, NUM_RESERVED_GATES);

    const size_t subgroup_size = circuit_proving_key->circuit_size;

    polynomial poly_q_table_column_1(subgroup_size);
    polynomial poly_q_table_column_2(subgroup_size);
    polynomial poly_q_table_column_3(subgroup_size);
    polynomial poly_q_table_column_4(subgroup_size);

    size_t offset = subgroup_size - tables_size - s_randomness - 1;

    // Create lookup selector polynomials which interpolate each table column.
    // Our selector polys always need to interpolate the full subgroup size, so here we offset so as to
    // put the table column's values at the end. (The first gates are for non-lookup constraints).
    // [0, ..., 0, ...table, 0, 0, 0, x]
    //  ^^^^^^^^^  ^^^^^^^^  ^^^^^^^  ^nonzero to ensure uniqueness and to avoid infinity commitments
    //  |          table     randomness
    //  ignored, as used for regular constraints and padding to the next power of 2.

    for (size_t i = 0; i < offset; ++i) {
        poly_q_table_column_1[i] = 0;
        poly_q_table_column_2[i] = 0;
        poly_q_table_column_3[i] = 0;
        poly_q_table_column_4[i] = 0;
    }

    for (const auto& table : lookup_tables) {
        const fr table_index(table.table_index);

        for (size_t i = 0; i < table.size; ++i) {
            poly_q_table_column_1[offset] = table.column_1[i];
            poly_q_table_column_2[offset] = table.column_2[i];
            poly_q_table_column_3[offset] = table.column_3[i];
            poly_q_table_column_4[offset] = table_index;
            ++offset;
        }
    }

    // Initialise the last `s_randomness` positions in table polynomials with 0. We don't need to actually randomise the
    // table polynomials.
    for (size_t i = 0; i < s_randomness; ++i) {
        poly_q_table_column_1[offset] = 0;
        poly_q_table_column_2[offset] = 0;
        poly_q_table_column_3[offset] = 0;
        poly_q_table_column_4[offset] = 0;
        ++offset;
    }

    // In the case of using UltraComposer for a circuit which does _not_ make use of any lookup tables, all four table
    // columns would be all zeros. This would result in these polys' commitments all being the point at infinity (which
    // is bad because our point arithmetic assumes we'll never operate on the point at infinity). To avoid this, we set
    // the last evaluation of each poly to be nonzero. The last `num_roots_cut_out_of_vanishing_poly = 4` evaluations
    // are ignored by constraint checks; we arbitrarily choose the very-last evaluation to be nonzero. See
    // ComposerBase::compute_proving_key_base for further explanation, as a similar trick is done there.
    // We could have chosen `1` for each such evaluation here, but that would have resulted in identical commitments for
    // all four columns. We don't want to have equal commitments, because biggroup operations assume no points are
    // equal, so if we tried to verify an ultra proof in a circuit, the biggroup operations would fail. To combat this,
    // we just choose distinct values:
    ASSERT(offset == subgroup_size - 1);
    auto unique_last_value = num_selectors + 1; // Note: in compute_proving_key_base, moments earlier, each selector
                                                // vector was given a unique last value from 1..num_selectors. So we
                                                // avoid those values and continue the count, to ensure uniqueness.
    poly_q_table_column_1[subgroup_size - 1] = unique_last_value;
    poly_q_table_column_2[subgroup_size - 1] = ++unique_last_value;
    poly_q_table_column_3[subgroup_size - 1] = ++unique_last_value;
    poly_q_table_column_4[subgroup_size - 1] = ++unique_last_value;

    add_table_column_selector_poly_to_proving_key(poly_q_table_column_1, "table_value_1");
    add_table_column_selector_poly_to_proving_key(poly_q_table_column_2, "table_value_2");
    add_table_column_selector_poly_to_proving_key(poly_q_table_column_3, "table_value_3");
    add_table_column_selector_poly_to_proving_key(poly_q_table_column_4, "table_value_4");

    // Instantiate z_lookup and s polynomials in the proving key (no values assigned yet).
    // Note: might be better to add these polys to cache only after they've been computed, as is convention
    polynomial z_lookup_fft(subgroup_size * 4);
    polynomial s_fft(subgroup_size * 4);
    circuit_proving_key->polynomial_cache.put("z_lookup_fft", std::move(z_lookup_fft));
    circuit_proving_key->polynomial_cache.put("s_fft", std::move(s_fft));

    // TODO: composer-level constant variable needed for the program width
    compute_sigma_permutations<4, true>(circuit_proving_key.get());

    std::copy(memory_records.begin(), memory_records.end(), std::back_inserter(circuit_proving_key->memory_records));

    circuit_proving_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(recursive_proof_public_input_indices.begin(), recursive_proof_public_input_indices.end());

    circuit_proving_key->contains_recursive_proof = contains_recursive_proof;

    return circuit_proving_key;
}

std::shared_ptr<verification_key> UltraComposer::compute_verification_key()
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }
    if (!circuit_proving_key) {
        compute_proving_key();
    }
    circuit_verification_key =
        ComposerBase::compute_verification_key_base(circuit_proving_key, crs_factory_->get_verifier_crs());

    circuit_verification_key->composer_type = type; // Invariably plookup for this class.

    // See `add_recusrive_proof()` for how this recursive data is assigned.
    circuit_verification_key->recursive_proof_public_input_indices =
        std::vector<uint32_t>(recursive_proof_public_input_indices.begin(), recursive_proof_public_input_indices.end());

    circuit_verification_key->contains_recursive_proof = contains_recursive_proof;

    return circuit_verification_key;
}

/**
 * @brief Computes `this.witness`, which is basiclly a set of polynomials mapped-to by strings.
 *
 * Note: this doesn't actually compute the _entire_ witness. Things missing: randomness for blinding both the wires and
 * sorted `s` poly, lookup rows of the wire witnesses, the values of `z_lookup`, `z`. These are all calculated
 * elsewhere.
 */
void UltraComposer::compute_witness()
{
    if (computed_witness) {
        return;
    }

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    const size_t filled_gates = num_gates + public_inputs.size();
    const size_t total_num_gates = std::max(filled_gates, tables_size + lookups_size);

    const size_t subgroup_size = get_circuit_subgroup_size(total_num_gates + NUM_RESERVED_GATES);

    // Pad the wires (pointers to `witness_indices` of the `variables` vector).
    // Note: the remaining NUM_RESERVED_GATES indices are padded with zeros within `compute_witness_base` (called
    // next).
    for (size_t i = filled_gates; i < total_num_gates; ++i) {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(zero_idx);
        w_o.emplace_back(zero_idx);
        w_4.emplace_back(zero_idx);
    }

    // Create and store polynomials which interpolate the wire values (variable values pointed-to by the `w_`s).
    ComposerBase::compute_witness_base<ultra_settings>(total_num_gates);

    polynomial s_1(subgroup_size);
    polynomial s_2(subgroup_size);
    polynomial s_3(subgroup_size);
    polynomial s_4(subgroup_size);
    polynomial z_lookup(subgroup_size + 1); // Only instantiated in this function; nothing assigned.

    // Save space for adding random scalars in the s polynomial later.
    // The subtracted 1 allows us to insert a `1` at the end, to ensure the evaluations (and hence coefficients) aren't
    // all 0.
    // See ComposerBase::compute_proving_key_base for further explanation, as a similar trick is done there.
    size_t count = subgroup_size - tables_size - lookups_size - s_randomness - 1;
    for (size_t i = 0; i < count; ++i) {
        s_1[i] = 0;
        s_2[i] = 0;
        s_3[i] = 0;
        s_4[i] = 0;
    }

    for (auto& table : lookup_tables) {
        const fr table_index(table.table_index);
        auto& lookup_gates = table.lookup_gates;
        for (size_t i = 0; i < table.size; ++i) {
            if (table.use_twin_keys) {
                lookup_gates.push_back({
                    {
                        table.column_1[i].from_montgomery_form().data[0],
                        table.column_2[i].from_montgomery_form().data[0],
                    },
                    {
                        table.column_3[i],
                        0,
                    },
                });
            } else {
                lookup_gates.push_back({
                    {
                        table.column_1[i].from_montgomery_form().data[0],
                        0,
                    },
                    {
                        table.column_2[i],
                        table.column_3[i],
                    },
                });
            }
        }

#ifdef NO_TBB
        std::sort(lookup_gates.begin(), lookup_gates.end());
#else
        std::sort(std::execution::par_unseq, lookup_gates.begin(), lookup_gates.end());
#endif

        for (const auto& entry : lookup_gates) {
            const auto components = entry.to_sorted_list_components(table.use_twin_keys);
            s_1[count] = components[0];
            s_2[count] = components[1];
            s_3[count] = components[2];
            s_4[count] = table_index;
            ++count;
        }
    }

    // Initialise the `s_randomness` positions in the s polynomials with 0.
    // These will be the positions where we will be adding random scalars to add zero knowledge
    // to plookup (search for `Blinding` in plonk/proof_system/widgets/random_widgets/plookup_widget_impl.hpp
    // ProverPlookupWidget::compute_sorted_list_polynomial())
    for (size_t i = 0; i < s_randomness; ++i) {
        s_1[count] = 0;
        s_2[count] = 0;
        s_3[count] = 0;
        s_4[count] = 0;
        ++count;
    }

    circuit_proving_key->polynomial_cache.put("s_1_lagrange", std::move(s_1));
    circuit_proving_key->polynomial_cache.put("s_2_lagrange", std::move(s_2));
    circuit_proving_key->polynomial_cache.put("s_3_lagrange", std::move(s_3));
    circuit_proving_key->polynomial_cache.put("s_4_lagrange", std::move(s_4));

    computed_witness = true;
}

UltraProver UltraComposer::create_prover()
{
    compute_proving_key();
    compute_witness();
    UltraProver output_state(circuit_proving_key, create_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get());
    std::unique_ptr<ProverPlookupWidget<>> plookup_widget =
        std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupArithmeticWidget<ultra_settings>> arithmetic_widget =
        std::make_unique<ProverPlookupArithmeticWidget<ultra_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverUltraFixedBaseWidget<ultra_settings>> fixed_base_widget =
        std::make_unique<ProverUltraFixedBaseWidget<ultra_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverGenPermSortWidget<ultra_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<ultra_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverEllipticWidget<ultra_settings>> elliptic_widget =
        std::make_unique<ProverEllipticWidget<ultra_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupAuxiliaryWidget<ultra_settings>> auxiliary_widget =
        std::make_unique<ProverPlookupAuxiliaryWidget<ultra_settings>>(circuit_proving_key.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.random_widgets.emplace_back(std::move(plookup_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(fixed_base_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));
    output_state.transition_widgets.emplace_back(std::move(elliptic_widget));
    output_state.transition_widgets.emplace_back(std::move(auxiliary_widget));

    std::unique_ptr<KateCommitmentScheme<ultra_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<ultra_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

/**
 * @note 'unrolled' means 'don't use linearisation techniques from the plonk paper'.
 */
UnrolledUltraProver UltraComposer::create_unrolled_prover()
{
    compute_proving_key();
    compute_witness();

    UnrolledUltraProver output_state(circuit_proving_key, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupWidget<>> plookup_widget =
        std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupArithmeticWidget<unrolled_ultra_settings>> arithmetic_widget =
        std::make_unique<ProverPlookupArithmeticWidget<unrolled_ultra_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverUltraFixedBaseWidget<unrolled_ultra_settings>> fixed_base_widget =
        std::make_unique<ProverUltraFixedBaseWidget<unrolled_ultra_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverGenPermSortWidget<unrolled_ultra_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<unrolled_ultra_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverEllipticWidget<unrolled_ultra_settings>> elliptic_widget =
        std::make_unique<ProverEllipticWidget<unrolled_ultra_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupAuxiliaryWidget<unrolled_ultra_settings>> auxiliary_widget =
        std::make_unique<ProverPlookupAuxiliaryWidget<unrolled_ultra_settings>>(circuit_proving_key.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.random_widgets.emplace_back(std::move(plookup_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(fixed_base_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));
    output_state.transition_widgets.emplace_back(std::move(elliptic_widget));
    output_state.transition_widgets.emplace_back(std::move(auxiliary_widget));

    std::unique_ptr<KateCommitmentScheme<unrolled_ultra_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<unrolled_ultra_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

/**
 * @brief Uses slightly different settings from the UnrolledUltraProver.
 * @note 'unrolled' means 'don't use linearisation techniques from the plonk paper'.
 */
UnrolledUltraToStandardProver UltraComposer::create_unrolled_ultra_to_standard_prover()
{
    compute_proving_key();
    compute_witness();

    UnrolledUltraToStandardProver output_state(circuit_proving_key, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupWidget<>> plookup_widget =
        std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupArithmeticWidget<unrolled_ultra_to_standard_settings>> arithmetic_widget =
        std::make_unique<ProverPlookupArithmeticWidget<unrolled_ultra_to_standard_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverUltraFixedBaseWidget<unrolled_ultra_to_standard_settings>> fixed_base_widget =
        std::make_unique<ProverUltraFixedBaseWidget<unrolled_ultra_to_standard_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverGenPermSortWidget<unrolled_ultra_to_standard_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<unrolled_ultra_to_standard_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverEllipticWidget<unrolled_ultra_to_standard_settings>> elliptic_widget =
        std::make_unique<ProverEllipticWidget<unrolled_ultra_to_standard_settings>>(circuit_proving_key.get());

    std::unique_ptr<ProverPlookupAuxiliaryWidget<unrolled_ultra_to_standard_settings>> auxiliary_widget =
        std::make_unique<ProverPlookupAuxiliaryWidget<unrolled_ultra_to_standard_settings>>(circuit_proving_key.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.random_widgets.emplace_back(std::move(plookup_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(fixed_base_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));
    output_state.transition_widgets.emplace_back(std::move(elliptic_widget));
    output_state.transition_widgets.emplace_back(std::move(auxiliary_widget));

    std::unique_ptr<KateCommitmentScheme<unrolled_ultra_to_standard_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<unrolled_ultra_to_standard_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

UltraVerifier UltraComposer::create_verifier()
{
    compute_verification_key();

    UltraVerifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));

    std::unique_ptr<KateCommitmentScheme<turbo_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<turbo_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

UnrolledUltraVerifier UltraComposer::create_unrolled_verifier()
{
    compute_verification_key();

    UnrolledUltraVerifier output_state(circuit_verification_key, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<KateCommitmentScheme<unrolled_ultra_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<unrolled_ultra_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

UnrolledUltraToStandardVerifier UltraComposer::create_unrolled_ultra_to_standard_verifier()
{
    compute_verification_key();

    UnrolledUltraToStandardVerifier output_state(circuit_verification_key,
                                                 create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<KateCommitmentScheme<unrolled_ultra_to_standard_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<unrolled_ultra_to_standard_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

void UltraComposer::initialize_precomputed_table(
    const plookup::BasicTableId id,
    bool (*generator)(std::vector<fr>&, std ::vector<fr>&, std::vector<fr>&),
    std::array<fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>))
{
    for (auto table : lookup_tables) {
        ASSERT(table.id != id);
    }
    plookup::BasicTable new_table;
    new_table.id = id;
    new_table.table_index = lookup_tables.size() + 1;
    new_table.use_twin_keys = generator(new_table.column_1, new_table.column_2, new_table.column_3);
    new_table.size = new_table.column_1.size();
    new_table.get_values_from_key = get_values_from_key;
    lookup_tables.emplace_back(new_table);
}

plookup::BasicTable& UltraComposer::get_table(const plookup::BasicTableId id)
{
    for (plookup::BasicTable& table : lookup_tables) {
        if (table.id == id) {
            return table;
        }
    }
    // Table doesn't exist! So try to create it.
    lookup_tables.emplace_back(plookup::create_basic_table(id, lookup_tables.size()));
    return lookup_tables[lookup_tables.size() - 1];
}

/**
 * @brief Perform a series of lookups, one for each 'row' in read_values.
 */
plookup::ReadData<uint32_t> UltraComposer::create_gates_from_plookup_accumulators(
    const plookup::MultiTableId& id,
    const plookup::ReadData<barretenberg::fr>& read_values,
    const uint32_t key_a_index,
    std::optional<uint32_t> key_b_index)
{
    ULTRA_SELECTOR_REFS;
    const auto& multi_table = plookup::create_table(id);
    const size_t num_lookups = read_values[plookup::ColumnIdx::C1].size();
    plookup::ReadData<uint32_t> read_data;
    for (size_t i = 0; i < num_lookups; ++i) {
        auto& table = get_table(multi_table.lookup_ids[i]);

        table.lookup_gates.emplace_back(read_values.key_entries[i]);

        const auto first_idx = (i == 0) ? key_a_index : add_variable(read_values[plookup::ColumnIdx::C1][i]);
        const auto second_idx = (i == 0 && (key_b_index.has_value()))
                                    ? key_b_index.value()
                                    : add_variable(read_values[plookup::ColumnIdx::C2][i]);
        const auto third_idx = add_variable(read_values[plookup::ColumnIdx::C3][i]);

        read_data[plookup::ColumnIdx::C1].push_back(first_idx);
        read_data[plookup::ColumnIdx::C2].push_back(second_idx);
        read_data[plookup::ColumnIdx::C3].push_back(third_idx);
        assert_valid_variables({ first_idx, second_idx, third_idx });

        q_lookup_type.emplace_back(fr(1));
        q_3.emplace_back(fr(table.table_index));
        w_l.emplace_back(first_idx);
        w_r.emplace_back(second_idx);
        w_o.emplace_back(third_idx);
        w_4.emplace_back(zero_idx);
        q_1.emplace_back(0);
        q_2.emplace_back((i == (num_lookups - 1) ? 0 : -multi_table.column_1_step_sizes[i + 1]));
        q_m.emplace_back((i == (num_lookups - 1) ? 0 : -multi_table.column_2_step_sizes[i + 1]));
        q_c.emplace_back((i == (num_lookups - 1) ? 0 : -multi_table.column_3_step_sizes[i + 1]));
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_fixed_base.emplace_back(0);
        q_sort.emplace_back(0);
        q_elliptic.emplace_back(0);
        q_aux.emplace_back(0);
        ++num_gates;
    }
    return read_data;
}

/**
 * Generalized Permutation Methods
 **/

UltraComposer::RangeList UltraComposer::create_range_list(const uint64_t target_range)
{
    RangeList result;
    const auto range_tag = get_new_tag(); // current_tag + 1;
    const auto tau_tag = get_new_tag();   // current_tag + 2;
    create_tag(range_tag, tau_tag);
    create_tag(tau_tag, range_tag);
    result.target_range = target_range;
    result.range_tag = range_tag;
    result.tau_tag = tau_tag;

    uint64_t num_multiples_of_three = (target_range / DEFAULT_PLOOKUP_RANGE_STEP_SIZE);

    result.variable_indices.reserve((uint32_t)num_multiples_of_three);
    for (uint64_t i = 0; i <= num_multiples_of_three; ++i) {
        const uint32_t index = add_variable(i * DEFAULT_PLOOKUP_RANGE_STEP_SIZE);
        result.variable_indices.emplace_back(index);
        assign_tag(index, result.range_tag);
    }
    {
        const uint32_t index = add_variable(target_range);
        result.variable_indices.emplace_back(index);
        assign_tag(index, result.range_tag);
    }
    // Need this because these variables will not appear in the witness otherwise
    create_dummy_constraints(result.variable_indices);

    return result;
}

// range constraint a value by decomposing it into limbs whose size should be the default range constraint size
std::vector<uint32_t> UltraComposer::decompose_into_default_range(const uint32_t variable_index,
                                                                  const uint64_t num_bits,
                                                                  const uint64_t target_range_bitnum,
                                                                  std::string const& msg)
{
    assert_valid_variables({ variable_index });

    ASSERT(num_bits > 0);

    uint256_t val = (uint256_t)(get_variable(variable_index));

    // If the value is out of range, set the composer error to the given msg.
    if (val.get_msb() >= num_bits && !failed()) {
        failure(msg);
    }

    const uint64_t sublimb_mask = (1ULL << target_range_bitnum) - 1;
    std::vector<uint64_t> sublimbs;
    std::vector<uint32_t> sublimb_indices;

    const bool has_remainder_bits = (num_bits % target_range_bitnum != 0);
    const uint64_t num_limbs = (num_bits / target_range_bitnum) + has_remainder_bits;
    const uint64_t last_limb_size = num_bits - ((num_bits / target_range_bitnum) * target_range_bitnum);
    const uint64_t last_limb_range = ((uint64_t)1 << last_limb_size) - 1;

    uint256_t accumulator = val;
    for (size_t i = 0; i < num_limbs; ++i) {
        sublimbs.push_back(accumulator.data[0] & sublimb_mask);
        accumulator = accumulator >> target_range_bitnum;
    }
    for (size_t i = 0; i < sublimbs.size(); ++i) {
        const auto limb_idx = add_variable(sublimbs[i]);
        sublimb_indices.emplace_back(limb_idx);
        if ((i == sublimbs.size() - 1) && has_remainder_bits) {
            create_new_range_constraint(limb_idx, last_limb_range);
        } else {
            create_new_range_constraint(limb_idx, sublimb_mask);
        }
    }

    const uint64_t num_limb_triples = (num_limbs / 3) + ((num_limbs % 3) != 0);
    const uint64_t leftovers = (num_limbs % 3) == 0 ? 3 : (num_limbs % 3);

    accumulator = val;
    uint32_t accumulator_idx = variable_index;

    for (size_t i = 0; i < num_limb_triples; ++i) {
        const bool real_limbs[3]{
            (i == (num_limb_triples - 1) && (leftovers < 1)) ? false : true,
            (i == (num_limb_triples - 1) && (leftovers < 2)) ? false : true,
            (i == (num_limb_triples - 1) && (leftovers < 3)) ? false : true,
        };

        const uint64_t round_sublimbs[3]{
            real_limbs[0] ? sublimbs[3 * i] : 0,
            real_limbs[1] ? sublimbs[3 * i + 1] : 0,
            real_limbs[2] ? sublimbs[3 * i + 2] : 0,
        };
        const uint32_t new_limbs[3]{
            real_limbs[0] ? sublimb_indices[3 * i] : zero_idx,
            real_limbs[1] ? sublimb_indices[3 * i + 1] : zero_idx,
            real_limbs[2] ? sublimb_indices[3 * i + 2] : zero_idx,
        };
        const uint64_t shifts[3]{
            target_range_bitnum * (3 * i),
            target_range_bitnum * (3 * i + 1),
            target_range_bitnum * (3 * i + 2),
        };
        uint256_t new_accumulator = accumulator - (uint256_t(round_sublimbs[0]) << shifts[0]) -
                                    (uint256_t(round_sublimbs[1]) << shifts[1]) -
                                    (uint256_t(round_sublimbs[2]) << shifts[2]);

        create_big_add_gate(
            {
                new_limbs[0],
                new_limbs[1],
                new_limbs[2],
                accumulator_idx,
                uint256_t(1) << shifts[0],
                uint256_t(1) << shifts[1],
                uint256_t(1) << shifts[2],
                -1,
                0,
            },
            ((i == num_limb_triples - 1) ? false : true));
        accumulator_idx = add_variable(new_accumulator);
        accumulator = new_accumulator;
    }
    return sublimb_indices;
}

/**
 * @brief Constrain a variable to a range
 *
 * @details Checks if the range [0, target_range] already exists. If it doesn't, then creates a new range. Then tags
 * variable as belonging to this set.
 *
 * @param variable_index
 * @param target_range
 */
void UltraComposer::create_new_range_constraint(const uint32_t variable_index, const uint64_t target_range)
{
    ASSERT(target_range != 0);
    if (range_lists.count(target_range) == 0) {
        range_lists.insert({ target_range, create_range_list(target_range) });
    }

    auto& list = range_lists[target_range];
    assign_tag(variable_index, list.range_tag);
    list.variable_indices.emplace_back(variable_index);
}

void UltraComposer::process_range_list(const RangeList& list)
{
    assert_valid_variables(list.variable_indices);

    ASSERT(list.variable_indices.size() > 0);
    // go over variables
    // for each variable, create mirror variable with same value - with tau tag
    // need to make sure that, in original list, increments of at most 3
    std::vector<uint64_t> sorted_list;
    sorted_list.reserve(list.variable_indices.size());
    for (const auto variable_index : list.variable_indices) {
        const auto& field_element = get_variable(variable_index);
        const uint64_t shrinked_value = field_element.from_montgomery_form().data[0];
        sorted_list.emplace_back(shrinked_value);
    }

#ifdef NO_TBB
    std::sort(sorted_list.begin(), sorted_list.end());
#else
    std::sort(std::execution::par_unseq, sorted_list.begin(), sorted_list.end());
#endif
    std::vector<uint32_t> indices;

    // list must be padded to a multipe of 4 and larger than 4 (gate_width)
    constexpr size_t gate_width = ultra_settings::program_width;
    size_t padding = (gate_width - (list.variable_indices.size() % gate_width)) % gate_width;
    if (list.variable_indices.size() <= gate_width)
        padding += gate_width;
    for (size_t i = 0; i < padding; ++i) {
        indices.emplace_back(zero_idx);
    }
    for (const auto sorted_value : sorted_list) {
        const uint32_t index = add_variable(sorted_value);
        assign_tag(index, list.tau_tag);
        indices.emplace_back(index);
    }
    create_sort_constraint_with_edges(indices, 0, list.target_range);
}

void UltraComposer::process_range_lists()
{
    for (const auto& i : range_lists)
        process_range_list(i.second);
}

/*
 Create range constraint:
  * add variable index to a list of range constrained variables
  * data structures: vector of lists, each list contains:
  *    - the range size
  *    - the list of variables in the range
  *    - a generalised permutation tag
  *
  * create range constraint parameters: variable index && range size
  *
  * std::map<uint64_t, RangeList> range_lists;
*/
// Check for a sequence of variables that neighboring differences are at most 3 (used for batched range checkj)
void UltraComposer::create_sort_constraint(const std::vector<uint32_t>& variable_index)
{
    ULTRA_SELECTOR_REFS
    constexpr size_t gate_width = ultra_settings::program_width;
    ASSERT(variable_index.size() % gate_width == 0);
    assert_valid_variables(variable_index);

    for (size_t i = 0; i < variable_index.size(); i += gate_width) {

        w_l.emplace_back(variable_index[i]);
        w_r.emplace_back(variable_index[i + 1]);
        w_o.emplace_back(variable_index[i + 2]);
        w_4.emplace_back(variable_index[i + 3]);
        ++num_gates;
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_fixed_base.emplace_back(0);
        q_sort.emplace_back(1);
        q_elliptic.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_aux.emplace_back(0);
    }
    // dummy gate needed because of sort widget's check of next row
    w_l.emplace_back(variable_index[variable_index.size() - 1]);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    ++num_gates;
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_arith.emplace_back(0);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_aux.emplace_back(0);
}

// useful to put variables in the witness that aren't already used - e.g. the dummy variables of the range constraint in
// multiples of three
void UltraComposer::create_dummy_constraints(const std::vector<uint32_t>& variable_index)
{
    ULTRA_SELECTOR_REFS
    std::vector<uint32_t> padded_list = variable_index;
    constexpr size_t gate_width = ultra_settings::program_width;
    const uint64_t padding = (gate_width - (padded_list.size() % gate_width)) % gate_width;
    for (uint64_t i = 0; i < padding; ++i) {
        padded_list.emplace_back(zero_idx);
    }
    assert_valid_variables(variable_index);
    assert_valid_variables(padded_list);

    for (size_t i = 0; i < padded_list.size(); i += gate_width) {
        w_l.emplace_back(padded_list[i]);
        w_r.emplace_back(padded_list[i + 1]);
        w_o.emplace_back(padded_list[i + 2]);
        w_4.emplace_back(padded_list[i + 3]);
        ++num_gates;
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_fixed_base.emplace_back(0);
        q_sort.emplace_back(0);
        q_elliptic.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_aux.emplace_back(0);
    }
}

// Check for a sequence of variables that neighboring differences are at most 3 (used for batched range checks)
void UltraComposer::create_sort_constraint_with_edges(const std::vector<uint32_t>& variable_index,
                                                      const fr& start,
                                                      const fr& end)
{
    ULTRA_SELECTOR_REFS
    // Convenient to assume size is at least 8 (gate_width = 4) for separate gates for start and end conditions
    constexpr size_t gate_width = ultra_settings::program_width;
    ASSERT(variable_index.size() % gate_width == 0 && variable_index.size() > gate_width);
    assert_valid_variables(variable_index);

    // enforce range checks of first row and starting at start
    w_l.emplace_back(variable_index[0]);
    w_r.emplace_back(variable_index[1]);
    w_o.emplace_back(variable_index[2]);
    w_4.emplace_back(variable_index[3]);
    ++num_gates;
    q_m.emplace_back(0);
    q_1.emplace_back(1);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(-start);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(1);
    q_elliptic.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_aux.emplace_back(0);
    // enforce range check for middle rows
    for (size_t i = gate_width; i < variable_index.size() - gate_width; i += gate_width) {

        w_l.emplace_back(variable_index[i]);
        w_r.emplace_back(variable_index[i + 1]);
        w_o.emplace_back(variable_index[i + 2]);
        w_4.emplace_back(variable_index[i + 3]);
        ++num_gates;
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_fixed_base.emplace_back(0);
        q_sort.emplace_back(1);
        q_elliptic.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_aux.emplace_back(0);
    }
    // enforce range checks of last row and ending at end
    if (variable_index.size() > gate_width) {
        w_l.emplace_back(variable_index[variable_index.size() - 4]);
        w_r.emplace_back(variable_index[variable_index.size() - 3]);
        w_o.emplace_back(variable_index[variable_index.size() - 2]);
        w_4.emplace_back(variable_index[variable_index.size() - 1]);
        ++num_gates;
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_fixed_base.emplace_back(0);
        q_sort.emplace_back(1);
        q_elliptic.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_aux.emplace_back(0);
    }

    // dummy gate needed because of sort widget's check of next row
    // use this gate to check end condition
    w_l.emplace_back(variable_index[variable_index.size() - 1]);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    ++num_gates;
    q_m.emplace_back(0);
    q_1.emplace_back(1);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(-end);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_sort.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_aux.emplace_back(0);
}

// range constraint a value by decomposing it into limbs whose size should be the default range constraint size
std::vector<uint32_t> UltraComposer::decompose_into_default_range_better_for_oddlimbnum(const uint32_t variable_index,
                                                                                        const size_t num_bits,
                                                                                        std::string const& msg)
{
    std::vector<uint32_t> sums;
    const size_t limb_num = (size_t)num_bits / DEFAULT_PLOOKUP_RANGE_BITNUM;
    const size_t last_limb_size = num_bits - (limb_num * DEFAULT_PLOOKUP_RANGE_BITNUM);
    if (limb_num < 3) {
        std::cerr
            << "number of bits in range must be an integer multipe of DEFAULT_PLOOKUP_RANGE_BITNUM of size at least 3"
            << std::endl;
        return sums;
    }

    const uint256_t val = (uint256_t)(get_variable(variable_index));
    // check witness value is indeed in range (commented out cause interferes with negative tests)
    // ASSERT(val < ((uint256_t)1 << num_bits) - 1); // Q:ask Zac what happens with wrapping when converting fr to
    // uint256
    // ASSERT(limb_num % 3 == 0); // TODO: write version of method that doesn't need this
    std::vector<uint32_t> val_limbs;
    std::vector<fr> val_slices;
    for (size_t i = 0; i < limb_num; i++) {
        val_slices.emplace_back(
            barretenberg::fr(val.slice(DEFAULT_PLOOKUP_RANGE_BITNUM * i, DEFAULT_PLOOKUP_RANGE_BITNUM * (i + 1) - 1)));
        val_limbs.emplace_back(add_variable(val_slices[i]));
        create_new_range_constraint(val_limbs[i], DEFAULT_PLOOKUP_RANGE_SIZE);
    }

    uint64_t last_limb_range = ((uint64_t)1 << last_limb_size) - 1;
    fr last_slice(0);
    uint32_t last_limb(zero_idx);
    size_t total_limb_num = limb_num;
    if (last_limb_size > 0) {
        val_slices.emplace_back(fr(val.slice(num_bits - last_limb_size, num_bits)));
        val_limbs.emplace_back(add_variable(last_slice));
        create_new_range_constraint(last_limb, last_limb_range);
        total_limb_num++;
    }
    // pad slices and limbs in case they are not 2 mod 3
    if (total_limb_num % 3 == 1) {
        val_limbs.emplace_back(zero_idx); // TODO: check this is zero
        val_slices.emplace_back(0);
        total_limb_num++;
    }
    fr shift = fr(1 << DEFAULT_PLOOKUP_RANGE_BITNUM);
    fr second_shift = shift * shift;
    sums.emplace_back(add_variable(val_slices[0] + shift * val_slices[1] + second_shift * val_slices[2]));
    create_big_add_gate({ val_limbs[0], val_limbs[1], val_limbs[2], sums[0], 1, shift, second_shift, -1, 0 });
    fr cur_shift = (shift * second_shift);
    fr cur_second_shift = cur_shift * shift;
    for (size_t i = 3; i < total_limb_num; i = i + 2) {
        sums.emplace_back(add_variable(get_variable(sums[sums.size() - 1]) + cur_shift * val_slices[i] +
                                       cur_second_shift * val_slices[i + 1]));
        create_big_add_gate({ sums[sums.size() - 2],
                              val_limbs[i],
                              val_limbs[i + 1],
                              sums[sums.size() - 1],
                              1,
                              cur_shift,
                              cur_second_shift,
                              -1,
                              0 });
        cur_shift *= second_shift;
        cur_second_shift *= second_shift;
    }
    assert_equal(sums[sums.size() - 1], variable_index, msg);
    return sums;
}

/**
 * @brief Enable the auxilary gate of particular type
 *
 * @details If we have several operations being performed do not require parametrization
 * (if we put each of them into a separate widget they would not require any selectors other than the ones enabling the
 * operation itself, for example q_special*(w_l-2*w_r)), we can group them all into one widget, by using a special
 * selector q_aux for all of them and enabling each in particular, depending on the combination of standard selector
 * values. So you can do:
 * q_aux * (q_1 * q_2 * statement_1 + q_3 * q_4 * statement_2). q_1=q_2=1 would activate statement_1, while q_3=q_4=1
 * would activate statement_2
 * @param type
 */
void UltraComposer::apply_aux_selectors(const AUX_SELECTORS type)
{
    ULTRA_SELECTOR_REFS;
    q_arith.emplace_back(0);
    q_fixed_base.emplace_back(0);
    q_aux.emplace_back(type == AUX_SELECTORS::NONE ? 0 : 1);
    q_c.emplace_back(0);
    q_sort.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    switch (type) {
    case AUX_SELECTORS::LIMB_ACCUMULATE_1: {
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(1);
        q_4.emplace_back(1);
        q_m.emplace_back(0);
        break;
    }
    case AUX_SELECTORS::LIMB_ACCUMULATE_2: {
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(1);
        q_4.emplace_back(0);
        q_m.emplace_back(1);
        break;
    }
    case AUX_SELECTORS::NON_NATIVE_FIELD_1: {
        q_1.emplace_back(0);
        q_2.emplace_back(1);
        q_3.emplace_back(1);
        q_4.emplace_back(0);
        q_m.emplace_back(0);
        break;
    }
    case AUX_SELECTORS::NON_NATIVE_FIELD_2: {
        q_1.emplace_back(0);
        q_2.emplace_back(1);
        q_3.emplace_back(0);
        q_4.emplace_back(1);
        q_m.emplace_back(0);
        break;
    }
    case AUX_SELECTORS::NON_NATIVE_FIELD_3: {
        q_1.emplace_back(0);
        q_2.emplace_back(1);
        q_3.emplace_back(0);
        q_4.emplace_back(0);
        q_m.emplace_back(1);
        break;
    }
    case AUX_SELECTORS::CONSISTENT_SORTED_MEMORY_READ: {
        // Memory read gate used with the sorted list of memory reads.
        // Apply sorted memory read checks with the following additional check:
        // 1. Assert that if index field across two gates does not change, the value field does not change.
        // Used for ROM reads and RAM reads across write/read boundaries
        q_1.emplace_back(1);
        q_2.emplace_back(1);
        q_3.emplace_back(0);
        q_4.emplace_back(0);
        q_m.emplace_back(0);
        break;
    }
    case AUX_SELECTORS::SORTED_MEMORY_READ: {
        // Memory read gate used with the sorted list of memory reads.
        // 1. Validate adjacent index values across 2 gates increases by 0 or 1
        // 2. Validate record witness computation (r = index + \kappa timestamp * \kappa^2 * value)
        // Used for ROM reads and RAM reads across read/write boundaries
        q_1.emplace_back(1);
        q_2.emplace_back(0);
        q_3.emplace_back(1);
        q_4.emplace_back(0);
        q_m.emplace_back(0);
        break;
    }
    case AUX_SELECTORS::MEMORY_TIMESTAMP_CORRECTNESS: {
        q_1.emplace_back(1);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_4.emplace_back(1);
        q_m.emplace_back(0);
        break;
    }
    case AUX_SELECTORS::MEMORY_READ: {
        // Memory read gate for reading/writing memory cells.
        // Validates record witness computation (r = index + \kappa timestamp * \kappa^2 * value)
        q_1.emplace_back(1);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_4.emplace_back(0);
        q_m.emplace_back(1); // validate record witness is correctly computed
        break;
    }
    default: {
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_4.emplace_back(0);
        q_m.emplace_back(0);
        break;
    }
    }
}

/**
 * NON NATIVE FIELD METHODS
 *
 * Methods to efficiently apply constraints that evaluate non-native field multiplications
 **/

/**
 * Applies range constraints to two 70-bit limbs, splititng each into 5 14-bit sublimbs.
 * We can efficiently chain together two 70-bit limb checks in 3 gates, using auxiliary gates
 **/
void UltraComposer::range_constrain_two_limbs(const uint32_t lo_idx,
                                              const uint32_t hi_idx,
                                              const size_t lo_limb_bits,
                                              const size_t hi_limb_bits)
{
    // Validate limbs are <= 70 bits. If limbs are larger we require more witnesses and cannot use our limb accumulation
    // custom gate
    ASSERT(lo_limb_bits <= (14 * 5));
    ASSERT(hi_limb_bits <= (14 * 5));

    // Sometimes we try to use limbs that are too large. It's easier to catch this issue here
    const auto get_sublimbs = [&](const uint32_t& limb_idx, const std::array<uint64_t, 5>& sublimb_masks) {
        const uint256_t limb = get_variable(limb_idx);
        // we can use constant 2^14 - 1 mask here. If the sublimb value exceeds the expected value then witness will
        // fail the range check below
        // We also use zero_idx to substitute variables that should be zero
        constexpr uint256_t MAX_SUBLIMB_MASK = (uint256_t(1) << 14) - 1;
        std::array<uint32_t, 5> sublimb_indices;
        sublimb_indices[0] = sublimb_masks[0] != 0 ? add_variable(limb & MAX_SUBLIMB_MASK) : zero_idx;
        sublimb_indices[1] = sublimb_masks[1] != 0 ? add_variable((limb >> 14) & MAX_SUBLIMB_MASK) : zero_idx;
        sublimb_indices[2] = sublimb_masks[2] != 0 ? add_variable((limb >> 28) & MAX_SUBLIMB_MASK) : zero_idx;
        sublimb_indices[3] = sublimb_masks[3] != 0 ? add_variable((limb >> 42) & MAX_SUBLIMB_MASK) : zero_idx;
        sublimb_indices[4] = sublimb_masks[4] != 0 ? add_variable((limb >> 56) & MAX_SUBLIMB_MASK) : zero_idx;
        return sublimb_indices;
    };

    const auto get_limb_masks = [](size_t limb_bits) {
        std::array<uint64_t, 5> sublimb_masks;
        sublimb_masks[0] = limb_bits >= 14 ? 14 : limb_bits;
        sublimb_masks[1] = limb_bits >= 28 ? 14 : (limb_bits > 14 ? limb_bits - 14 : 0);
        sublimb_masks[2] = limb_bits >= 42 ? 14 : (limb_bits > 28 ? limb_bits - 28 : 0);
        sublimb_masks[3] = limb_bits >= 56 ? 14 : (limb_bits > 42 ? limb_bits - 42 : 0);
        sublimb_masks[4] = (limb_bits > 56 ? limb_bits - 56 : 0);

        for (auto& mask : sublimb_masks) {
            mask = (1ULL << mask) - 1ULL;
        }
        return sublimb_masks;
    };

    const auto lo_masks = get_limb_masks(lo_limb_bits);
    const auto hi_masks = get_limb_masks(hi_limb_bits);
    const std::array<uint32_t, 5> lo_sublimbs = get_sublimbs(lo_idx, lo_masks);
    const std::array<uint32_t, 5> hi_sublimbs = get_sublimbs(hi_idx, hi_masks);

    w_l.emplace_back(lo_sublimbs[0]);
    w_r.emplace_back(lo_sublimbs[1]);
    w_o.emplace_back(lo_sublimbs[2]);
    w_4.emplace_back(lo_idx);

    w_l.emplace_back(lo_sublimbs[3]);
    w_r.emplace_back(lo_sublimbs[4]);
    w_o.emplace_back(hi_sublimbs[0]);
    w_4.emplace_back(hi_sublimbs[1]);

    w_l.emplace_back(hi_sublimbs[2]);
    w_r.emplace_back(hi_sublimbs[3]);
    w_o.emplace_back(hi_sublimbs[4]);
    w_4.emplace_back(hi_idx);

    apply_aux_selectors(AUX_SELECTORS::LIMB_ACCUMULATE_1);
    apply_aux_selectors(AUX_SELECTORS::LIMB_ACCUMULATE_2);
    apply_aux_selectors(AUX_SELECTORS::NONE);
    num_gates += 3;

    for (size_t i = 0; i < 5; i++) {
        if (lo_masks[i] != 0) {
            create_new_range_constraint(lo_sublimbs[i], lo_masks[i]);
        }
        if (hi_masks[i] != 0) {
            create_new_range_constraint(hi_sublimbs[i], hi_masks[i]);
        }
    }
};

/**
 * @brief Decompose a single witness into two, where the lowest is DEFAULT_NON_NATIVE_FIELD_LIMB_BITS (68) range
 * constrained and the lowst is num_limb_bits - DEFAULT.. range constrained.
 *
 * @details Doesn't create gates constraining the limbs to each other.
 *
 * @param limb_idx The index of the limb that will be decomposed
 * @param num_limb_bits The range we want to constrain the original limb to
 * @return std::array<uint32_t, 2> The indices of new limbs.
 */
std::array<uint32_t, 2> UltraComposer::decompose_non_native_field_double_width_limb(const uint32_t limb_idx,
                                                                                    const size_t num_limb_bits)
{
    ASSERT(uint256_t(get_variable_reference(limb_idx)) < (uint256_t(1) << num_limb_bits));
    constexpr barretenberg::fr LIMB_MASK = (uint256_t(1) << DEFAULT_NON_NATIVE_FIELD_LIMB_BITS) - 1;
    const uint256_t value = get_variable(limb_idx);
    const uint256_t low = value & LIMB_MASK;
    const uint256_t hi = value >> DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;
    ASSERT(low + (hi << DEFAULT_NON_NATIVE_FIELD_LIMB_BITS) == value);

    const uint32_t low_idx = add_variable(low);
    const uint32_t hi_idx = add_variable(hi);

    ASSERT(num_limb_bits > DEFAULT_NON_NATIVE_FIELD_LIMB_BITS);
    const size_t lo_bits = DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;
    const size_t hi_bits = num_limb_bits - DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;
    range_constrain_two_limbs(low_idx, hi_idx, lo_bits, hi_bits);

    return std::array<uint32_t, 2>{ low_idx, hi_idx };
}

/**
 * NON NATIVE FIELD MULTIPLICATION CUSTOM GATE SEQUENCE
 *
 * This method will evaluate the equation (a * b = q * p + r)
 * Where a, b, q, r are all emulated non-native field elements that are each split across 4 distinct witness variables
 *
 * The non-native field modulus, p, is a circuit constant
 *
 * The return value are the witness indices of the two remainder limbs `lo_1, hi_2`
 *
 * N.B. this method does NOT evaluate the prime field component of non-native field multiplications
 **/
std::array<uint32_t, 2> UltraComposer::evaluate_non_native_field_multiplication(
    const non_native_field_witnesses& input, const bool range_constrain_quotient_and_remainder)
{

    std::array<fr, 4> a{
        get_variable(input.a[0]),
        get_variable(input.a[1]),
        get_variable(input.a[2]),
        get_variable(input.a[3]),
    };
    std::array<fr, 4> b{
        get_variable(input.b[0]),
        get_variable(input.b[1]),
        get_variable(input.b[2]),
        get_variable(input.b[3]),
    };
    std::array<fr, 4> q{
        get_variable(input.q[0]),
        get_variable(input.q[1]),
        get_variable(input.q[2]),
        get_variable(input.q[3]),
    };
    std::array<fr, 4> r{
        get_variable(input.r[0]),
        get_variable(input.r[1]),
        get_variable(input.r[2]),
        get_variable(input.r[3]),
    };

    constexpr barretenberg::fr LIMB_SHIFT = uint256_t(1) << DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;
    constexpr barretenberg::fr LIMB_SHIFT_2 = uint256_t(1) << (2 * DEFAULT_NON_NATIVE_FIELD_LIMB_BITS);
    constexpr barretenberg::fr LIMB_SHIFT_3 = uint256_t(1) << (3 * DEFAULT_NON_NATIVE_FIELD_LIMB_BITS);
    constexpr barretenberg::fr LIMB_RSHIFT =
        barretenberg::fr(1) / barretenberg::fr(uint256_t(1) << DEFAULT_NON_NATIVE_FIELD_LIMB_BITS);
    constexpr barretenberg::fr LIMB_RSHIFT_2 =
        barretenberg::fr(1) / barretenberg::fr(uint256_t(1) << (2 * DEFAULT_NON_NATIVE_FIELD_LIMB_BITS));

    barretenberg::fr lo_0 = a[0] * b[0] - r[0] + (a[1] * b[0] + a[0] * b[1]) * LIMB_SHIFT;
    barretenberg::fr lo_1 = (lo_0 + q[0] * input.neg_modulus[0] +
                             (q[1] * input.neg_modulus[0] + q[0] * input.neg_modulus[1] - r[1]) * LIMB_SHIFT) *
                            LIMB_RSHIFT_2;

    barretenberg::fr hi_0 = a[2] * b[0] + a[0] * b[2] + (a[0] * b[3] + a[3] * b[0] - r[3]) * LIMB_SHIFT;
    barretenberg::fr hi_1 = hi_0 + a[1] * b[1] - r[2] + (a[1] * b[2] + a[2] * b[1]) * LIMB_SHIFT;
    barretenberg::fr hi_2 = (hi_1 + lo_1 + q[2] * input.neg_modulus[0] +
                             (q[3] * input.neg_modulus[0] + q[2] * input.neg_modulus[1]) * LIMB_SHIFT);
    barretenberg::fr hi_3 = (hi_2 + (q[0] * input.neg_modulus[3] + q[1] * input.neg_modulus[2]) * LIMB_SHIFT +
                             (q[0] * input.neg_modulus[2] + q[1] * input.neg_modulus[1])) *
                            LIMB_RSHIFT_2;

    const uint32_t lo_0_idx = add_variable(lo_0);
    const uint32_t lo_1_idx = add_variable(lo_1);
    const uint32_t hi_0_idx = add_variable(hi_0);
    const uint32_t hi_1_idx = add_variable(hi_1);
    const uint32_t hi_2_idx = add_variable(hi_2);
    const uint32_t hi_3_idx = add_variable(hi_3);

    // Sometimes we have already applied range constraints on the quotient and remainder
    if (range_constrain_quotient_and_remainder) {
        // /**
        //  * r_prime - r_0 - r_1 * 2^b - r_2 * 2^2b - r_3 * 2^3b = 0
        //  *
        //  * w_4_omega - w_4 + w_1(2^b) + w_2(2^2b) + w_3(2^3b)  = 0
        //  *
        //  **/
        create_big_add_gate(
            { input.r[1], input.r[2], input.r[3], input.r[4], LIMB_SHIFT, LIMB_SHIFT_2, LIMB_SHIFT_3, -1, 0 }, true);
        range_constrain_two_limbs(input.r[0], input.r[1]);
        range_constrain_two_limbs(input.r[2], input.r[3]);

        // /**
        //  * q_prime - q_0 - q_1 * 2^b - q_2 * 2^2b - q_3 * 2^3b = 0
        //  *
        //  * w_4_omega - w_4 + w_1(2^b) + w_2(2^2b) + w_3(2^3b)  = 0
        //  *
        //  **/
        create_big_add_gate(
            { input.q[1], input.q[2], input.q[3], input.q[4], LIMB_SHIFT, LIMB_SHIFT_2, LIMB_SHIFT_3, -1, 0 }, true);
        range_constrain_two_limbs(input.q[0], input.q[1]);
        range_constrain_two_limbs(input.q[2], input.q[3]);
    }

    // product gate 1
    // (lo_0 + q_0(p_0 + p_1*2^b) + q_1(p_0*2^b) - (r_1)2^b)2^-2b - lo_1 = 0
    create_big_add_gate({ input.q[0],
                          input.q[1],
                          input.r[1],
                          lo_1_idx,
                          input.neg_modulus[0] + input.neg_modulus[1] * LIMB_SHIFT,
                          input.neg_modulus[0] * LIMB_SHIFT,
                          -LIMB_SHIFT,
                          -LIMB_SHIFT.sqr(),
                          0 },
                        true);

    w_l.emplace_back(input.a[1]);
    w_r.emplace_back(input.b[1]);
    w_o.emplace_back(input.r[0]);
    w_4.emplace_back(lo_0_idx);
    apply_aux_selectors(AUX_SELECTORS::NON_NATIVE_FIELD_1);
    ++num_gates;
    w_l.emplace_back(input.a[0]);
    w_r.emplace_back(input.b[0]);
    w_o.emplace_back(input.a[3]);
    w_4.emplace_back(input.b[3]);
    apply_aux_selectors(AUX_SELECTORS::NON_NATIVE_FIELD_2);
    ++num_gates;
    w_l.emplace_back(input.a[2]);
    w_r.emplace_back(input.b[2]);
    w_o.emplace_back(input.r[3]);
    w_4.emplace_back(hi_0_idx);
    apply_aux_selectors(AUX_SELECTORS::NON_NATIVE_FIELD_3);
    ++num_gates;
    w_l.emplace_back(input.a[1]);
    w_r.emplace_back(input.b[1]);
    w_o.emplace_back(input.r[2]);
    w_4.emplace_back(hi_1_idx);
    apply_aux_selectors(AUX_SELECTORS::NONE);
    ++num_gates;

    /**
     * product gate 6
     *
     * hi_2 - hi_1 - lo_1 - q[2](p[1].2^b + p[0]) - q[3](p[0].2^b) = 0
     *
     **/
    create_big_add_gate(
        {
            input.q[2],
            input.q[3],
            lo_1_idx,
            hi_1_idx,
            -input.neg_modulus[1] * LIMB_SHIFT - input.neg_modulus[0],
            -input.neg_modulus[0] * LIMB_SHIFT,
            -1,
            -1,
            0,
        },
        true);

    /**
     * product gate 7
     *
     * hi_3 - (hi_2 - q[0](p[3].2^b + p[2]) - q[1](p[2].2^b + p[1])).2^-2b
     **/
    create_big_add_gate({
        hi_3_idx,
        input.q[0],
        input.q[1],
        hi_2_idx,
        -1,
        input.neg_modulus[3] * LIMB_RSHIFT + input.neg_modulus[2] * LIMB_RSHIFT_2,
        input.neg_modulus[2] * LIMB_RSHIFT + input.neg_modulus[1] * LIMB_RSHIFT_2,
        LIMB_RSHIFT_2,
        0,
    });

    return std::array<uint32_t, 2>{ lo_1_idx, hi_3_idx };
}

/**
 * Compute the limb-multiplication part of a non native field mul
 *
 * i.e. compute the low 204 and high 204 bit components of `a * b` where `a, b` are nnf elements composed of 4
 * limbs with size DEFAULT_NON_NATIVE_FIELD_LIMB_BITS
 *
 **/
std::array<uint32_t, 2> UltraComposer::evaluate_partial_non_native_field_multiplication(
    const non_native_field_witnesses& input)
{

    std::array<fr, 4> a{
        get_variable(input.a[0]),
        get_variable(input.a[1]),
        get_variable(input.a[2]),
        get_variable(input.a[3]),
    };
    std::array<fr, 4> b{
        get_variable(input.b[0]),
        get_variable(input.b[1]),
        get_variable(input.b[2]),
        get_variable(input.b[3]),
    };

    constexpr barretenberg::fr LIMB_SHIFT = uint256_t(1) << DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;

    barretenberg::fr lo_0 = a[0] * b[0] + (a[1] * b[0] + a[0] * b[1]) * LIMB_SHIFT;

    barretenberg::fr hi_0 = a[2] * b[0] + a[0] * b[2] + (a[0] * b[3] + a[3] * b[0]) * LIMB_SHIFT;
    barretenberg::fr hi_1 = hi_0 + a[1] * b[1] + (a[1] * b[2] + a[2] * b[1]) * LIMB_SHIFT;

    const uint32_t lo_0_idx = add_variable(lo_0);
    const uint32_t hi_0_idx = add_variable(hi_0);
    const uint32_t hi_1_idx = add_variable(hi_1);

    w_l.emplace_back(input.a[1]);
    w_r.emplace_back(input.b[1]);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(lo_0_idx);
    apply_aux_selectors(AUX_SELECTORS::NON_NATIVE_FIELD_1);
    ++num_gates;
    w_l.emplace_back(input.a[0]);
    w_r.emplace_back(input.b[0]);
    w_o.emplace_back(input.a[3]);
    w_4.emplace_back(input.b[3]);
    apply_aux_selectors(AUX_SELECTORS::NON_NATIVE_FIELD_2);
    ++num_gates;
    w_l.emplace_back(input.a[2]);
    w_r.emplace_back(input.b[2]);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(hi_0_idx);
    apply_aux_selectors(AUX_SELECTORS::NON_NATIVE_FIELD_3);
    ++num_gates;
    w_l.emplace_back(input.a[1]);
    w_r.emplace_back(input.b[1]);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(hi_1_idx);
    apply_aux_selectors(AUX_SELECTORS::NONE);
    ++num_gates;
    return std::array<uint32_t, 2>{ lo_0_idx, hi_1_idx };
}

/**
 * Uses a sneaky extra mini-addition gate in `plookup_arithmetic_widget.hpp` to add two non-native
 * field elements in 4 gates (would normally take 5)
 **/
std::array<uint32_t, 5> UltraComposer::evaluate_non_native_field_addition(
    add_simple limb0,
    add_simple limb1,
    add_simple limb2,
    add_simple limb3,
    std::tuple<uint32_t, uint32_t, barretenberg::fr> limbp)
{
    const auto& x_0 = std::get<0>(limb0).first;
    const auto& x_1 = std::get<0>(limb1).first;
    const auto& x_2 = std::get<0>(limb2).first;
    const auto& x_3 = std::get<0>(limb3).first;
    const auto& x_p = std::get<0>(limbp);

    const auto& x_mulconst0 = std::get<0>(limb0).second;
    const auto& x_mulconst1 = std::get<0>(limb1).second;
    const auto& x_mulconst2 = std::get<0>(limb2).second;
    const auto& x_mulconst3 = std::get<0>(limb3).second;

    const auto& y_0 = std::get<1>(limb0).first;
    const auto& y_1 = std::get<1>(limb1).first;
    const auto& y_2 = std::get<1>(limb2).first;
    const auto& y_3 = std::get<1>(limb3).first;
    const auto& y_p = std::get<1>(limbp);

    const auto& y_mulconst0 = std::get<1>(limb0).second;
    const auto& y_mulconst1 = std::get<1>(limb1).second;
    const auto& y_mulconst2 = std::get<1>(limb2).second;
    const auto& y_mulconst3 = std::get<1>(limb3).second;

    // constant additive terms
    const auto& addconst0 = std::get<2>(limb0);
    const auto& addconst1 = std::get<2>(limb1);
    const auto& addconst2 = std::get<2>(limb2);
    const auto& addconst3 = std::get<2>(limb3);
    const auto& addconstp = std::get<2>(limbp);

    // get value of result limbs
    const auto z_0value = get_variable(x_0) * x_mulconst0 + get_variable(y_0) * y_mulconst0 + addconst0;
    const auto z_1value = get_variable(x_1) * x_mulconst1 + get_variable(y_1) * y_mulconst1 + addconst1;
    const auto z_2value = get_variable(x_2) * x_mulconst2 + get_variable(y_2) * y_mulconst2 + addconst2;
    const auto z_3value = get_variable(x_3) * x_mulconst3 + get_variable(y_3) * y_mulconst3 + addconst3;
    const auto z_pvalue = get_variable(x_p) + get_variable(y_p) + addconstp;

    const auto z_0 = add_variable(z_0value);
    const auto z_1 = add_variable(z_1value);
    const auto z_2 = add_variable(z_2value);
    const auto z_3 = add_variable(z_3value);
    const auto z_p = add_variable(z_pvalue);

    ULTRA_SELECTOR_REFS

    /**
     *   we want the following layout in program memory
     *   (x - y = z)
     *
     *   |  1  |  2  |  3  |  4  |
     *   |-----|-----|-----|-----|
     *   | y.p | x.0 | y.0 | x.p | (b.p + c.p - a.p = 0) AND (a.0 - b.0 - c.0 = 0)
     *   | z.p | x.1 | y.1 | z.0 | (a.1 - b.1 - c.1 = 0)
     *   | x.2 | y.2 | z.2 | z.1 | (a.2 - b.2 - c.2 = 0)
     *   | x.3 | y.3 | z.3 | --- | (a.3 - b.3 - c.3 = 0)
     *
     * By setting `q_arith` to `3`, we can validate `x_p + y_p + q_m = z_p`
     **/
    // GATE 1
    w_l.emplace_back(y_p);
    w_r.emplace_back(x_0);
    w_o.emplace_back(y_0);
    w_4.emplace_back(x_p);
    w_l.emplace_back(z_p);
    w_r.emplace_back(x_1);
    w_o.emplace_back(y_1); // |  1  |  2  |  3  |  4  |
    w_4.emplace_back(z_0); // |-----|-----|-----|-----|
    w_l.emplace_back(x_2); // | y.p | x.0 | y.0 | z.p | (b.p + b.p - c.p = 0) AND (a.0 + b.0 - c.0 = 0)
    w_r.emplace_back(y_2); // | x.p | x.1 | y.1 | z.0 | (a.1  + b.1 - c.1 = 0)
    w_o.emplace_back(z_2); // | x.2 | y.2 | z.2 | z.1 | (a.2  + b.2 - c.2 = 0)
    w_4.emplace_back(z_1); // | x.3 | y.3 | z.3 | --- | (a.3  + b.3 - c.3 = 0)
    w_l.emplace_back(x_3);
    w_r.emplace_back(y_3);
    w_o.emplace_back(z_3);
    w_4.emplace_back(zero_idx);

    q_m.emplace_back(addconstp);
    q_1.emplace_back(0);
    q_2.emplace_back(-x_mulconst0 *
                     2); // scale constants by 2. If q_arith = 3 then w_4_omega value (z0) gets scaled by 2x
    q_3.emplace_back(-y_mulconst0 * 2); // z_0 - (x_0 * -xmulconst0) - (y_0 * ymulconst0) = 0 => z_0 = x_0 + y_0
    q_4.emplace_back(0);
    q_c.emplace_back(-addconst0 * 2);
    q_arith.emplace_back(3);

    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(-x_mulconst1);
    q_3.emplace_back(-y_mulconst1);
    q_4.emplace_back(0);
    q_c.emplace_back(-addconst1);
    q_arith.emplace_back(2);

    q_m.emplace_back(0);
    q_1.emplace_back(-x_mulconst2);
    q_2.emplace_back(-y_mulconst2);
    q_3.emplace_back(1);
    q_4.emplace_back(0);
    q_c.emplace_back(-addconst2);
    q_arith.emplace_back(1);

    q_m.emplace_back(0);
    q_1.emplace_back(-x_mulconst3);
    q_2.emplace_back(-y_mulconst3);
    q_3.emplace_back(1);
    q_4.emplace_back(0);
    q_c.emplace_back(-addconst3);
    q_arith.emplace_back(1);

    for (size_t i = 0; i < 4; ++i) {
        q_fixed_base.emplace_back(0);
        q_sort.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_elliptic.emplace_back(0);
        q_aux.emplace_back(0);
    }

    num_gates += 4;
    return std::array<uint32_t, 5>{
        z_0, z_1, z_2, z_3, z_p,
    };
}

std::array<uint32_t, 5> UltraComposer::evaluate_non_native_field_subtraction(
    add_simple limb0,
    add_simple limb1,
    add_simple limb2,
    add_simple limb3,
    std::tuple<uint32_t, uint32_t, barretenberg::fr> limbp)
{
    const auto& x_0 = std::get<0>(limb0).first;
    const auto& x_1 = std::get<0>(limb1).first;
    const auto& x_2 = std::get<0>(limb2).first;
    const auto& x_3 = std::get<0>(limb3).first;
    const auto& x_p = std::get<0>(limbp);

    const auto& x_mulconst0 = std::get<0>(limb0).second;
    const auto& x_mulconst1 = std::get<0>(limb1).second;
    const auto& x_mulconst2 = std::get<0>(limb2).second;
    const auto& x_mulconst3 = std::get<0>(limb3).second;

    const auto& y_0 = std::get<1>(limb0).first;
    const auto& y_1 = std::get<1>(limb1).first;
    const auto& y_2 = std::get<1>(limb2).first;
    const auto& y_3 = std::get<1>(limb3).first;
    const auto& y_p = std::get<1>(limbp);

    const auto& y_mulconst0 = std::get<1>(limb0).second;
    const auto& y_mulconst1 = std::get<1>(limb1).second;
    const auto& y_mulconst2 = std::get<1>(limb2).second;
    const auto& y_mulconst3 = std::get<1>(limb3).second;

    // constant additive terms
    const auto& addconst0 = std::get<2>(limb0);
    const auto& addconst1 = std::get<2>(limb1);
    const auto& addconst2 = std::get<2>(limb2);
    const auto& addconst3 = std::get<2>(limb3);
    const auto& addconstp = std::get<2>(limbp);

    // get value of result limbs
    const auto z_0value = get_variable(x_0) * x_mulconst0 - get_variable(y_0) * y_mulconst0 + addconst0;
    const auto z_1value = get_variable(x_1) * x_mulconst1 - get_variable(y_1) * y_mulconst1 + addconst1;
    const auto z_2value = get_variable(x_2) * x_mulconst2 - get_variable(y_2) * y_mulconst2 + addconst2;
    const auto z_3value = get_variable(x_3) * x_mulconst3 - get_variable(y_3) * y_mulconst3 + addconst3;
    const auto z_pvalue = get_variable(x_p) - get_variable(y_p) + addconstp;

    const auto z_0 = add_variable(z_0value);
    const auto z_1 = add_variable(z_1value);
    const auto z_2 = add_variable(z_2value);
    const auto z_3 = add_variable(z_3value);
    const auto z_p = add_variable(z_pvalue);

    ULTRA_SELECTOR_REFS

    /**
     *   we want the following layout in program memory
     *   (x - y = z)
     *
     *   |  1  |  2  |  3  |  4  |
     *   |-----|-----|-----|-----|
     *   | y.p | x.0 | y.0 | z.p | (b.p + c.p - a.p = 0) AND (a.0 - b.0 - c.0 = 0)
     *   | x.p | x.1 | y.1 | z.0 | (a.1 - b.1 - c.1 = 0)
     *   | x.2 | y.2 | z.2 | z.1 | (a.2 - b.2 - c.2 = 0)
     *   | x.3 | y.3 | z.3 | --- | (a.3 - b.3 - c.3 = 0)
     *
     **/
    // GATE 1
    w_l.emplace_back(y_p);
    w_r.emplace_back(x_0);
    w_o.emplace_back(y_0);
    w_4.emplace_back(z_p);
    w_l.emplace_back(x_p);
    w_r.emplace_back(x_1);
    w_o.emplace_back(y_1); // |  1  |  2  |  3  |  4  |
    w_4.emplace_back(z_0); // |-----|-----|-----|-----|
    w_l.emplace_back(x_2); // | y.p | x.0 | y.0 | z.p | (b.p + c.p - a.p = 0) AND (a.0 - b.0 - c.0 = 0)
    w_r.emplace_back(y_2); // | x.p | x.1 | y.1 | z.0 | (a.1 - b.1 - c.1 = 0)
    w_o.emplace_back(z_2); // | x.2 | y.2 | z.2 | z.1 | (a.2 - b.2 - c.2 = 0)
    w_4.emplace_back(z_1); // | x.3 | y.3 | z.3 | --- | (a.3 - b.3 - c.3 = 0)
    w_l.emplace_back(x_3);
    w_r.emplace_back(y_3);
    w_o.emplace_back(z_3);
    w_4.emplace_back(zero_idx);

    q_m.emplace_back(-addconstp);
    q_1.emplace_back(0);
    q_2.emplace_back(-x_mulconst0 * 2);
    q_3.emplace_back(y_mulconst0 * 2); // z_0 + (x_0 * -xmulconst0) + (y_0 * ymulconst0) = 0 => z_0 = x_0 - y_0
    q_4.emplace_back(0);
    q_c.emplace_back(-addconst0 * 2);
    q_arith.emplace_back(3);

    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(-x_mulconst1);
    q_3.emplace_back(y_mulconst1);
    q_4.emplace_back(0);
    q_c.emplace_back(-addconst1);
    q_arith.emplace_back(2);

    q_m.emplace_back(0);
    q_1.emplace_back(-x_mulconst2);
    q_2.emplace_back(y_mulconst2);
    q_3.emplace_back(1);
    q_4.emplace_back(0);
    q_c.emplace_back(-addconst2);
    q_arith.emplace_back(1);

    q_m.emplace_back(0);
    q_1.emplace_back(-x_mulconst3);
    q_2.emplace_back(y_mulconst3);
    q_3.emplace_back(1);
    q_4.emplace_back(0);
    q_c.emplace_back(-addconst3);
    q_arith.emplace_back(1);

    for (size_t i = 0; i < 4; ++i) {
        q_fixed_base.emplace_back(0);
        q_sort.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_elliptic.emplace_back(0);
        q_aux.emplace_back(0);
    }

    num_gates += 4;
    return std::array<uint32_t, 5>{
        z_0, z_1, z_2, z_3, z_p,
    };
}

void UltraComposer::create_memory_gate(MemoryRecord& record)
{
    // Record wire value can't yet be computed
    record.record_witness = add_variable(0);
    apply_aux_selectors(AUX_SELECTORS::MEMORY_READ);
    w_l.emplace_back(record.index_witness);
    w_r.emplace_back(record.timestamp_witness);
    w_o.emplace_back(record.value_witness);
    w_4.emplace_back(record.record_witness);
    record.gate_index = num_gates;
    ++num_gates;
}

void UltraComposer::create_sorted_memory_gate(MemoryRecord& record, const bool is_ram_transition_or_rom)
{
    record.record_witness = add_variable(0);
    apply_aux_selectors(is_ram_transition_or_rom ? AUX_SELECTORS::CONSISTENT_SORTED_MEMORY_READ
                                                 : AUX_SELECTORS::SORTED_MEMORY_READ);
    w_l.emplace_back(record.index_witness);
    w_r.emplace_back(record.timestamp_witness);
    w_o.emplace_back(record.value_witness);
    w_4.emplace_back(record.record_witness);

    record.gate_index = num_gates;
    ++num_gates;
}

/**
 * @brief Create a new memory region
 *
 * @details Creates a transcript object, where the inside memory state array is filled with "uninitialized memory" and
 * and empty memory record array. Puts this object into the vector of ROM arrays.
 *
 * @param array_size The size of region in elements
 * @return size_t The index of the element
 */
size_t UltraComposer::create_ROM_array(const size_t array_size)
{
    MemoryTranscript new_transcript;
    for (size_t i = 0; i < array_size; ++i) {
        new_transcript.state.emplace_back(
            std::array<uint32_t, 2>{ UNINITIALIZED_MEMORY_RECORD, UNINITIALIZED_MEMORY_RECORD });
    }
    rom_arrays.emplace_back(new_transcript);
    return rom_arrays.size() - 1;
}

/**
 * Initialize a ROM cell to equal `value_witness`
 * `index_value` is a RAW VALUE that describes the cell index. It is NOT a witness
 * When intializing ROM arrays, it is important that the index of the cell is known when compiling the circuit.
 * This ensures that, for a given circuit, we know with 100% certainty that EVERY rom cell is initialized
 **/

/**
 * @brief Initialize a rom cell to equal `value_witness`
 *
 * @param rom_id The index of the ROM array, which cell we are initializing
 * @param index_value The index of the cell within the array (an actual index, not a witness index)
 * @param value_witness The index of the witness with the value that should be in the
 */
void UltraComposer::set_ROM_element(const size_t rom_id, const size_t index_value, const uint32_t value_witness)
{
    ASSERT(rom_arrays.size() > rom_id);
    auto& rom_array = rom_arrays[rom_id];
    const uint32_t index_witness = (index_value == 0) ? zero_idx : put_constant_variable((uint64_t)index_value);
    ASSERT(rom_array.state.size() > index_value);
    ASSERT(rom_array.state[index_value][0] == UNINITIALIZED_MEMORY_RECORD);
    /**
     * The structure MemoryRecord contains the following members in this order:
     *   uint32_t index_witness;
     *   uint32_t timestamp_witness;
     *   uint32_t value_witness;
     *   uint32_t index;
     *   uint32_t timestamp;
     *   uint32_t record_witness;
     *   size_t gate_index;
     * The second initialization value here is the witness, because in ROM it doesn't matter. We will decouple this
     * logic later.
     */
    MemoryRecord new_record{
        index_witness, value_witness, zero_idx, static_cast<uint32_t>(index_value), 0, 0, 0,
    };
    rom_array.state[index_value][0] = value_witness;
    rom_array.state[index_value][1] = zero_idx;
    create_memory_gate(new_record);
    rom_array.records.emplace_back(new_record);
}

void UltraComposer::set_ROM_element_pair(const size_t rom_id,
                                         const size_t index_value,
                                         const std::array<uint32_t, 2>& value_witnesses)
{
    ASSERT(rom_arrays.size() > rom_id);
    auto& rom_array = rom_arrays[rom_id];
    const uint32_t index_witness = (index_value == 0) ? zero_idx : put_constant_variable((uint64_t)index_value);
    ASSERT(rom_array.state.size() > index_value);
    ASSERT(rom_array.state[index_value][0] == UNINITIALIZED_MEMORY_RECORD);
    MemoryRecord new_record{
        index_witness, value_witnesses[0], value_witnesses[1], static_cast<uint32_t>(index_value), 0, 0, 0,
    };
    rom_array.state[index_value][0] = value_witnesses[0];
    rom_array.state[index_value][1] = value_witnesses[1];
    create_memory_gate(new_record);
    rom_array.records.emplace_back(new_record);
}

uint32_t UltraComposer::read_ROM_array(const size_t rom_id, const uint32_t index_witness)
{
    ASSERT(rom_arrays.size() > rom_id);
    auto& rom_array = rom_arrays[rom_id];
    const uint32_t index = static_cast<uint32_t>(uint256_t(get_variable(index_witness)));
    ASSERT(rom_array.state.size() > index);
    ASSERT(rom_array.state[index][0] != UNINITIALIZED_MEMORY_RECORD);
    const auto value = get_variable(rom_array.state[index][0]);
    const uint32_t value_witness = add_variable(value);
    MemoryRecord new_record{
        index_witness, value_witness, zero_idx, index, 0, 0, 0,
    };
    create_memory_gate(new_record);
    rom_array.records.emplace_back(new_record);

    // create_read_gate
    return value_witness;
}

std::array<uint32_t, 2> UltraComposer::read_ROM_array_pair(const size_t rom_id, const uint32_t index_witness)
{
    std::array<uint32_t, 2> value_witnesses;

    const uint32_t index = static_cast<uint32_t>(uint256_t(get_variable(index_witness)));
    ASSERT(rom_arrays.size() > rom_id);
    auto& rom_array = rom_arrays[rom_id];
    ASSERT(rom_array.state.size() > index);
    ASSERT(rom_array.state[index][0] != UNINITIALIZED_MEMORY_RECORD);
    ASSERT(rom_array.state[index][1] != UNINITIALIZED_MEMORY_RECORD);
    const auto value1 = get_variable(rom_array.state[index][0]);
    const auto value2 = get_variable(rom_array.state[index][1]);
    value_witnesses[0] = add_variable(value1);
    value_witnesses[1] = add_variable(value2);
    MemoryRecord new_record{
        index_witness, value_witnesses[0], value_witnesses[1], index, 0, 0, 0,
    };

    create_memory_gate(new_record);
    rom_array.records.emplace_back(new_record);

    // create_read_gate
    return value_witnesses;
}

void UltraComposer::process_ROM_array(const size_t rom_id, const size_t gate_offset_from_public_inputs)
{
    auto& rom_array = rom_arrays[rom_id];
    const auto read_tag = get_new_tag();        // current_tag + 1;
    const auto sorted_list_tag = get_new_tag(); // current_tag + 2;
    create_tag(read_tag, sorted_list_tag);
    create_tag(sorted_list_tag, read_tag);

    // Make sure that every cell has been initialized
    for (size_t i = 0; i < rom_array.state.size(); ++i) {
        if (rom_array.state[i][0] == UNINITIALIZED_MEMORY_RECORD) {
            set_ROM_element_pair(rom_id, static_cast<uint32_t>(i), { zero_idx, zero_idx });
        }
    }

#ifdef NO_TBB
    std::sort(rom_array.records.begin(), rom_array.records.end());
#else
    std::sort(std::execution::par_unseq, rom_array.records.begin(), rom_array.records.end());
#endif

    for (const auto& record : rom_array.records) {
        const auto index = record.index;
        const auto value1 = get_variable(record.timestamp_witness);
        const auto value2 = get_variable(record.value_witness);
        const auto index_witness = add_variable(fr((uint64_t)index));
        const auto value1_witness = add_variable(value1);
        const auto value2_witness = add_variable(value2);
        MemoryRecord sorted_record{
            index_witness, value1_witness, value2_witness, index, 0, 0, 0,
        };
        create_sorted_memory_gate(sorted_record, true);
        assign_tag(record.record_witness, read_tag);
        assign_tag(sorted_record.record_witness, sorted_list_tag);

        memory_records.push_back(static_cast<uint32_t>(sorted_record.gate_index + gate_offset_from_public_inputs));
        memory_records.push_back(static_cast<uint32_t>(record.gate_index + gate_offset_from_public_inputs));
    }
    // One of the checks we run on the sorted list, is to validate the difference between
    // the index field across two gates is either 0 or 1.
    // If we add a dummy gate at the end of the sorted list, where we force the first wire to
    // equal `m + 1`, where `m` is the maximum allowed index in the sorted list,
    // we have validated that all ROM reads are correctly constrained
    fr max_index_value((uint64_t)rom_array.state.size());
    uint32_t max_index = add_variable(max_index_value);
    create_big_add_gate({
        max_index,
        zero_idx,
        zero_idx,
        zero_idx,
        1,
        0,
        0,
        0,
        -max_index_value,
    });
    // N.B. If the above check holds, we know the sorted list begins with an index value of 0,
    // because the first cell is explicitly initialized using zero_idx as the index field.
}

void UltraComposer::process_ROM_arrays(const size_t gate_offset_from_public_inputs)
{
    for (size_t i = 0; i < rom_arrays.size(); ++i) {
        process_ROM_array(i, gate_offset_from_public_inputs);
    }
}
} // namespace waffle
