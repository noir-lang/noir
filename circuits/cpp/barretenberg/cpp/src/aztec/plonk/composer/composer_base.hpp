#pragma once
#include <ecc/curves/bn254/fr.hpp>
#include <proof_system/composer/composer_base.hpp>
#include <plonk/proof_system/prover/prover.hpp>
#include <plonk/proof_system/verifier/verifier.hpp>
#include <plonk/proof_system/types/prover_settings.hpp>
#include <srs/reference_string/file_reference_string.hpp>

using namespace bonk;

namespace waffle {
static constexpr uint32_t DUMMY_TAG = 0;

struct proving_key;
struct verification_key;

class ComposerBase {
  public:
    struct SelectorProperties {
        std::string name;
        bool requires_lagrange_base_polynomial = false; // does the prover need the raw lagrange-base selector values?
    };

    static constexpr uint32_t REAL_VARIABLE = UINT32_MAX - 1;
    static constexpr uint32_t FIRST_VARIABLE_IN_CLASS = UINT32_MAX - 2;
    static constexpr size_t NUM_RESERVED_GATES = 4; // this must be >= num_roots_cut_out_of_vanishing_polynomial

    // Enum values spaced in increments of 30-bits (multiples of 2 ** 30).
    enum WireType { LEFT = 0U, RIGHT = (1U << 30U), OUTPUT = (1U << 31U), FOURTH = 0xc0000000 };

    struct cycle_node {
        uint32_t gate_index;
        WireType wire_type;

        cycle_node(const uint32_t a, const WireType b)
            : gate_index(a)
            , wire_type(b)
        {}
        cycle_node(const cycle_node& other)
            : gate_index(other.gate_index)
            , wire_type(other.wire_type)
        {}
        cycle_node(cycle_node&& other)
            : gate_index(other.gate_index)
            , wire_type(other.wire_type)
        {}
        cycle_node& operator=(const cycle_node& other)
        {
            gate_index = other.gate_index;
            wire_type = other.wire_type;
            return *this;
        }
        bool operator==(const cycle_node& other) const
        {
            return ((gate_index == other.gate_index) && (wire_type == other.wire_type));
        }
    };

    ComposerBase()
        : ComposerBase(std::shared_ptr<ReferenceStringFactory>(new FileReferenceStringFactory("../srs_db/ignition")))
    {}
    ComposerBase(std::shared_ptr<ReferenceStringFactory> const& crs_factory,
                 size_t num_selectors = 0,
                 size_t size_hint = 0,
                 std::vector<SelectorProperties> selector_properties = {})
        : num_gates(0)
        , crs_factory_(crs_factory)
        , num_selectors(num_selectors)
        , selectors(num_selectors)
        , selector_properties(selector_properties)
        , rand_engine(nullptr)
    {
        for (auto& p : selectors) {
            p.reserve(size_hint);
        }
    }

    ComposerBase(size_t num_selectors = 0,
                 size_t size_hint = 0,
                 std::vector<SelectorProperties> selector_properties = {})
        : num_gates(0)
        , crs_factory_(std::make_unique<FileReferenceStringFactory>("../srs_db/ignition"))
        , num_selectors(num_selectors)
        , selectors(num_selectors)
        , selector_properties(selector_properties)
        , rand_engine(nullptr)
    {
        for (auto& p : selectors) {
            p.reserve(size_hint);
        }
    }

    ComposerBase(std::shared_ptr<proving_key> const& p_key,
                 std::shared_ptr<verification_key> const& v_key,
                 size_t num_selectors = 0,
                 size_t size_hint = 0,
                 std::vector<SelectorProperties> selector_properties = {})
        : num_gates(0)
        , circuit_proving_key(p_key)
        , circuit_verification_key(v_key)
        , num_selectors(num_selectors)
        , selectors(num_selectors)
        , selector_properties(selector_properties)
        , rand_engine(nullptr)
    {
        for (auto& p : selectors) {
            p.reserve(size_hint);
        }
    }

    ComposerBase(ComposerBase&& other) = default;
    ComposerBase& operator=(ComposerBase&& other) = default;
    virtual ~ComposerBase(){};

    virtual size_t get_num_gates() const { return num_gates; }
    virtual void print_num_gates() const { std::cout << num_gates << std::endl; }
    virtual size_t get_num_variables() const { return variables.size(); }
    virtual std::shared_ptr<proving_key> compute_proving_key_base(const waffle::ComposerType type = waffle::STANDARD,
                                                                  const size_t minimum_ciricut_size = 0,
                                                                  const size_t num_reserved_gates = NUM_RESERVED_GATES);
    // This needs to be static as it may be used only to compute the selector commitments.
    static std::shared_ptr<verification_key> compute_verification_key_base(
        std::shared_ptr<proving_key> const& proving_key, std::shared_ptr<VerifierReferenceString> const& vrs);
    virtual std::shared_ptr<proving_key> compute_proving_key() = 0;
    virtual std::shared_ptr<verification_key> compute_verification_key() = 0;
    virtual void compute_witness() = 0;
    template <class program_settings> void compute_witness_base(const size_t minimum_circuit_size = 0);
    uint32_t zero_idx = 0;

    virtual void create_add_gate(const add_triple& in) = 0;
    virtual void create_mul_gate(const mul_triple& in) = 0;
    virtual void create_bool_gate(const uint32_t a) = 0;
    virtual void create_poly_gate(const poly_triple& in) = 0;
    virtual size_t get_num_constant_gates() const = 0;

    /**
     * Get the index of the first variable in class.
     *
     * @param index The index of the variable you want to look up.
     *
     * @return The index of the first variable in the same class as the submitted index.
     * */
    uint32_t get_first_variable_in_class(uint32_t index) const
    {
        while (prev_var_index[index] != FIRST_VARIABLE_IN_CLASS) {
            index = prev_var_index[index];
        }
        return index;
    }
    /**
     * Update all variables from index in equivalence class to have real variable new_real_index.
     *
     * @param index The index of a variable in the class we're updating.
     * @param new_real_index The index of the real variable to update to.
     * */
    void update_real_variable_indices(uint32_t index, uint32_t new_real_index)
    {
        auto cur_index = index;
        do {
            real_variable_index[cur_index] = new_real_index;
            cur_index = next_var_index[cur_index];
        } while (cur_index != REAL_VARIABLE);
    }

    /**
     * Get the value of the variable v_{index}.
     * N.B. We should probably inline this.
     *
     * @param index The index of the variable.
     * @return The value of the variable.
     * */
    inline barretenberg::fr get_variable(const uint32_t index) const
    {
        ASSERT(variables.size() > index);
        return variables[real_variable_index[index]];
    }

    /**
     * Get a reference to the variable v_{index}.
     *
     * We need this function for check_circuit functions.
     *
     * @param index The index of the variable.
     * @return The value of the variable.
     * */
    inline const barretenberg::fr& get_variable_reference(const uint32_t index) const
    {
        ASSERT(variables.size() > index);
        return variables[real_variable_index[index]];
    }

    barretenberg::fr get_public_input(const uint32_t index) const { return get_variable(public_inputs[index]); }

    std::vector<fr> get_public_inputs() const
    {
        std::vector<fr> result;
        for (uint32_t i = 0; i < get_num_public_inputs(); ++i) {
            result.push_back(get_public_input(i));
        }
        return result;
    }

    /**
     * Add a variable to variables
     *
     * @param in The value of the variable
     * @return The index of the new variable in the variables vector
     */
    virtual uint32_t add_variable(const barretenberg::fr& in)
    {
        variables.emplace_back(in);

        // By default, we assume each new variable belongs in its own copy-cycle. These defaults can be modified later
        // by `assert_equal`.
        const uint32_t index = static_cast<uint32_t>(variables.size()) - 1U;
        real_variable_index.emplace_back(index);
        next_var_index.emplace_back(REAL_VARIABLE);
        prev_var_index.emplace_back(FIRST_VARIABLE_IN_CLASS);
        real_variable_tags.emplace_back(DUMMY_TAG);
        wire_copy_cycles.push_back(
            std::vector<cycle_node>()); // Note: this doesn't necessarily need to be initialised here. In fact, the
                                        // number of wire_copy_cycles often won't match the number of variables; its
                                        // non-zero entries will be a smaller vector of size equal to the number of
                                        // "real variables" (i.e. unique indices in the `real_variable_index` vector).
                                        // `wire_copy_cycles` could instead be instantiated during
                                        // compute_wire_copy_cycles(), although that would require a loop to identify
                                        // the number of unique "real variables".
        return index;
    }
    /**
     * Add a public variable to variables
     *
     * The only difference between this and add_variable is that here it is
     * also added to the public_inputs vector
     *
     * @param in The value of the variable
     * @return The index of the new variable in the variables vector
     */
    virtual uint32_t add_public_variable(const barretenberg::fr& in)
    {
        const uint32_t index = add_variable(in);
        public_inputs.emplace_back(index);
        return index;
    }

    /**
     * Make a witness variable public.
     *
     * @param witness_index The index of the witness.
     * */
    virtual void set_public_input(const uint32_t witness_index)
    {
        bool does_not_exist = true;
        for (size_t i = 0; i < public_inputs.size(); ++i) {
            does_not_exist = does_not_exist && (public_inputs[i] != witness_index);
        }
        if (does_not_exist) {
            public_inputs.emplace_back(witness_index);
        }
        ASSERT(does_not_exist);
        if (!does_not_exist && !failed()) {
            failure("Attempted to set a public input that is already public!");
        }
    }

    virtual void assert_equal(const uint32_t a_idx, const uint32_t b_idx, std::string const& msg = "assert_equal");

    template <size_t program_width> void compute_wire_copy_cycles();
    template <size_t program_width, bool with_tags = false> void compute_sigma_permutations(proving_key* key);

    size_t get_circuit_subgroup_size(const size_t num_gates)
    {
        size_t log2_n = static_cast<size_t>(numeric::get_msb(num_gates));
        if ((1UL << log2_n) != (num_gates)) {
            ++log2_n;
        }
        return 1UL << log2_n;
    }

    size_t get_num_public_inputs() const { return public_inputs.size(); }

    // Check whether each variable index points to a witness in the composer
    //
    // Any variable whose index does not point to witness value is deemed invalid.
    //
    // This implicitly checks whether a variable index
    // is equal to IS_CONSTANT; assuming that we will never have
    // uint32::MAX number of variables
    void assert_valid_variables(const std::vector<uint32_t>& variable_indices)
    {
        for (size_t i = 0; i < variable_indices.size(); i++) {
            ASSERT(is_valid_variable(variable_indices[i]));
        }
    }
    bool is_valid_variable(uint32_t variable_index) { return static_cast<uint32_t>(variables.size()) > variable_index; }
    bool _failed = false;
    std::string _err;

  public:
    size_t num_gates;
    std::vector<uint32_t> w_l;
    std::vector<uint32_t> w_r;
    std::vector<uint32_t> w_o;
    std::vector<uint32_t> w_4;
    std::vector<uint32_t> public_inputs;
    std::vector<barretenberg::fr> variables;
    std::vector<uint32_t> next_var_index; // index of next variable in equivalence class (=REAL_VARIABLE if you're last)
    std::vector<uint32_t>
        prev_var_index; // index of  previous variable in equivalence class (=FIRST if you're in a cycle alone)
    std::vector<uint32_t> real_variable_index; // indices of corresponding real variables
    std::vector<uint32_t> real_variable_tags;
    uint32_t current_tag = DUMMY_TAG;
    std::map<uint32_t, uint32_t>
        tau; // The permutation on variable tags. See
             // https://github.com/AztecProtocol/plonk-with-lookups-private/blob/new-stuff/GenPermuations.pdf
             // DOCTODO: replace with the relevant wiki link.
    std::vector<std::vector<cycle_node>> wire_copy_cycles;

    std::shared_ptr<proving_key> circuit_proving_key;
    std::shared_ptr<verification_key> circuit_verification_key;

    bool computed_witness = false;

    std::shared_ptr<ReferenceStringFactory> crs_factory_;
    size_t num_selectors;
    std::vector<std::vector<barretenberg::fr>> selectors;
    /**
     * @brief Contains the properties of each selector:
     * + name
     * + if the polynomial needs to be in lagrange form
     *
     * @details The actual values are always set during class construction. You can find them in:
     * + Standard plonk: standard_composer.hpp, function standard_sel_props()
     * + Turbo plonk: turbo_composer.cpp, function turbo_sel_props()
     * + Ultra plonk: ultra_composer.cpp, fucntion plookup_sel_props()
     */
    std::vector<SelectorProperties> selector_properties;
    numeric::random::Engine* rand_engine;
    bool failed() const { return _failed; };
    const std::string& err() const { return _err; };

    void set_err(std::string msg) { _err = msg; }
    void failure(std::string msg)
    {
        _failed = true;
        set_err(msg);
    }
};

extern template void ComposerBase::compute_wire_copy_cycles<3>();
extern template void ComposerBase::compute_wire_copy_cycles<4>();
extern template void ComposerBase::compute_sigma_permutations<3, false>(proving_key* key);
extern template void ComposerBase::compute_sigma_permutations<4, false>(proving_key* key);
extern template void ComposerBase::compute_witness_base<standard_settings>(const size_t);
extern template void ComposerBase::compute_witness_base<turbo_settings>(const size_t);
extern template void ComposerBase::compute_witness_base<ultra_settings>(const size_t);
extern template void ComposerBase::compute_sigma_permutations<4, true>(proving_key* key);

} // namespace waffle

/**
 * Composer Example: Pythagorean triples.
 *
 * (x_1 * x_1) + (x_2 * x_2) == (x_3 * x_3)
 *
 *************************************************************************************************************
 *
 * Notation as per the 'Constraint Systems' section of the Plonk paper:
 *       ______________________
 *      |                      |
 *      |              a_1 = 1 | c_1 = 4
 *      |  w_1 = x_{a_1} = x_1 | w_9 = x_{c_1} = x_4
 *  x_1 |                      * ---------------------- x_4
 *      |              b_1 = 1 | Gate 1                   |
 *      |  w_5 = x_{b_1} = x_1 |                  a_4 = 4 | c_4 = 7
 *      |______________________|      w_4 = x_{a_4} = x_4 | w_12 = x_{c_4} = x_7
 *                                                        + ------------------------ x_7
 *                                                b_4 = 5 | Gate 4                    =
 *       ______________________       w_8 = x_{b_4} = x_5 |                           =
 *      |                      |                          |                           =
 *      |              a_2 = 2 | c_2 = 5                  |                           =
 *      |  w_2 = x_{a_2} = x_2 | w_10 = x_{c_2} = x_5     |                           =
 *  x_2 |                      * ---------------------- x_5                           =
 *      |              b_2 = 2 | Gate 2                                               =
 *      |  w_6 = x_{b_2} = x_2 |                                      These `=`s      =
 *      |______________________|                                      symbolise a     =
 *                                                                    copy-constraint =
 *                                                                                    =
 *       ______________________                                                       =
 *      |                      |                                                      =
 *      |              a_3 = 3 | c_3 = 6                                              =
 *      |  w_3 = x_{a_3} = x_3 | w_11 = x_{c_3} = x_6                                 =
 *  x_3 |                      * --------------------------------------------------- x_6
 *      |              b_3 = 3 | Gate 3                                               ^
 *      |  w_7 = x_{b_3} = x_3 |                           Suppose x_6 is the only____|
 *      |______________________|                           public input.
 *
 * - 4 gates.
 * - 7 "variables" or "witnesses", denoted by the x's, whose indices are pointed-to by the values of the a,b,c's.
 *   #gates <= #variables <= 2 * #gates, always (see plonk paper).
 * - 12 "wires" (inputs / outputs to gates) (= 3 * #gates; for a fan-in-2 gate), denoted by the w's.
 *   Each wire takes the value of a variable (# wires >= # variables).
 *
 * a_1 = b_1 = 1
 * a_2 = b_2 = 2
 * a_3 = b_3 = 3
 * a_4 = c_1 = 4
 * b_4 = c_2 = 5
 * c_3 =       6
 * c_4 =       7
 *   ^     ^   ^
 *   |     |   |____ indices of the x's (variables (witnesses))
 *   |_____|________ indices of the gates
 *
 * So x_{a_1} = x_1, etc.
 *
 * Then we have "wire values":
 * w_1  = x_{a_1} = x_1
 * w_2  = x_{a_2} = x_2
 * w_3  = x_{a_3} = x_3
 * w_4  = x_{a_4} = x_4
 *
 * w_5  = x_{b_1} = x_1
 * w_6  = x_{b_2} = x_2
 * w_7  = x_{b_3} = x_3
 * w_8  = x_{b_4} = x_5
 *
 * w_9  = x_{c_1} = x_4
 * w_10 = x_{c_2} = x_5
 * w_11 = x_{c_3} = x_6
 * w_12 = x_{c_4} = x_7
 *
 ****************************************************************************************************************
 *
 * Notation as per this codebase is different from the Plonk paper:
 * This example is reproduced exactly in the stdlib field test `test_field_pythagorean`.
 *
 * variables[0] = 0 for all circuits <-- this gate is not shown in this diagram.
 *                   ______________________
 *                  |                      |
 *                  |                      |
 *                  |           w_l[1] = 1 | w_o[1] = 4
 *     variables[1] |                      * ------------------- variables[4]
 *                  |           w_r[1] = 1 | Gate 1                   |
 *                  |                      |                          |
 *                  |______________________|               w_l[4] = 4 | w_o[4] = 7
 *                                                                    + --------------------- variables[7]
 *                                                         w_r[4] = 5 | Gate 4                    =
 *                   ______________________                           |                           =
 *                  |                      |                          |                           =
 *                  |                      |                          |                           =
 *                  |           w_l[2] = 2 | w_o[2] = 5               |                           =
 *     variables[2] |                      * ------------------- variables[5]                     =
 *                  |           w_r[2] = 2 | Gate 2                                               =
 *                  |                      |                                      These `=`s      =
 *                  |______________________|                                      symbolise a     =
 *                                                                                copy-constraint =
 *                                                                                                =
 *                   ______________________                                                       =
 *                  |                      |                                                      =
 *                  |                      |                                                      =
 *                  |           w_l[3] = 3 | w_o[3] = 6                                           =
 *     variables[3] |                      * ------------------------------------------------variables[6]
 *                  |           w_r[3] = 3 | Gate 3                                               ^
 *                  |                      |                           Suppose this is the only___|
 *                  |______________________|                           public input.
 *
 * - 5 gates (4 gates plus the 'zero' gate).
 * - 7 "variables" or "witnesses", stored in the `variables` vector.
 *   #gates <= #variables <= 2 * #gates, always (see plonk paper).
 * - 12 "wires" (inputs / outputs to gates) (= 3 * #gates; for a fan-in-2 gate), denoted by the w's.
 *   Each wire takes the value of a variable (# wires >= # variables).
 *
 * ComposerBase naming conventions:
 *   - n = 5 gates (4 gates plus the 'zero' gate).
 *   - variables <-- A.k.a. "witnesses". Indices of this variables vector are referred to as `witness_indices`.
 * Example of varibales in this example (a 3,4,5 triangle):
 *   - variables      = [  0,   3,   4,   5,   9,  16,  25,  25]
 *   - public_inputs  = [6] <-- points to variables[6].
 *
 * These `w`'s are called "wires". In fact, they're witness_indices; pointing to indices in the `variables` vector.
 *   - w_l = [ 0, 1, 2, 3, 4]
 *   - w_r = [ 0, 1, 2, 3, 5]
 *   - w_o = [ 0, 4, 5, 6, 7]
 *             ^ The 0th wires are 0, for the default gate which instantiates the first witness as equal to 0.
 *   - w_4 = [ 0, 0, 0, 0, 0] <-- not used in this example.
 *   - selectors = [
 *                   q_m: [ 0, 1, 1, 1, 0],
 *                   q_c: [ 0, 0, 0, 0, 0],
 *                   q_1: [ 1, 0, 0, 0, 1],
 *                   q_2: [ 0, 0, 0, 0, 1],
 *                   q_3: [ 0,-1,-1,-1,-1],
 *                   q_4: [ 0, 0, 0, 0, 0], <-- not used in this example; doesn't exist in Standard PlonK.
 *                 ]
 *
 * These vectors imply copy-cycles between variables. ("copy-cycle" meaning "a set of variables which must always be
 * equal"). The indices of these vectors correspond to those of the `variables` vector. Each index contains
 * information about the corresponding variable.
 *   - next_var_index = [ -1,  -1,  -1,  -1,  -1,  -1,  -1,   6]
 *   - prev_var_index = [ -2,  -2,  -2,  -2,  -2,  -2,   7,  -2]
 *   - real_var_index = [  0,   1,   2,   3,   4,   5,   6,   6] <-- Notice this repeated 6.
 *
 *   `-1` = "The variable at this index is considered the last in its cycle (no next variable exists)"
 *          Note: we (arbitrarily) consider the "last" variable in a cycle to be the true representative of its
 *          cycle, and so dub it the "real" variable of the cycle.
 *   `-2` = "The variable at this index is considered the first in its cycle (no previous variable exists)"
 *   Any other number denotes the index of another variable in the cycle = "The variable at this index is equal to
 *   the variable at this other index".
 *
 * By default, when a variable is added to the composer, we assume the variable is in a copy-cycle of its own. So
 * we set `next_var_index = -1`, `prev_var_index = -2`, `real_var_index` = the index of the variable in `variables`.
 * You can see in our example that all but the last two indices of each *_index vector contain the default values.
 * In our example, we have `variables[6].assert_equal(variables[7])`. The `assert_equal` function modifies the above
 * vectors' entries for variables 6 & 7 to imply a copy-cycle between them. Arbitrarily, variables[7] is deemed the
 * "first" in the cycle and variables[6] is considered the last (and hence the "real" variable which represents the
 * cycle).
 *
 * By the time we get to computing wire copy-cycles, we need to allow for public_inputs, which in the plonk protocol
 * are positioned to be the first witness values. `variables` doesn't include these public inputs (they're stored
 * separately). In our example, we only have one public input, equal to `variables[6]`. We create a new "gate" for
 * this 'public inputs' version of `variables[6]`, and push it to the front of our gates. (i.e. The first
 * gate_index-es become gates for the public inputs, and all our `variables` occupy gates with gate_index-es shifted
 * up by the number of public inputs (by 1 in this example)):
 *   - wire_copy_cycles = [
 *         // The i-th index of `wire_copy_cycles` details the set of wires which all equal
 *         // variables[real_var_index[i]]. (I.e. equal to the i-th "real" variable):
 *         [
 *             { gate_index: 1, left   }, // w_l[1-#pub] = w_l[0] -> variables[0] = 0 <-- tag = 1 (id_mapping)
 *             { gate_index: 1, right  }, // w_r[1-#pub] = w_r[0] -> variables[0] = 0
 *             { gate_index: 1, output }, // w_o[1-#pub] = w_o[0] -> variables[0] = 0
 *             { gate_index: 1, 4th },    // w_4[1-#pub] = w_4[0] -> variables[0] = 0
 *             { gate_index: 2, 4th },    // w_4[2-#pub] = w_4[1] -> variables[0] = 0
 *             { gate_index: 3, 4th },    // w_4[3-#pub] = w_4[2] -> variables[0] = 0
 *             { gate_index: 4, 4th },    // w_4[4-#pub] = w_4[3] -> variables[0] = 0
 *             { gate_index: 5, 4th },    // w_4[5-#pub] = w_4[4] -> variables[0] = 0
 *         ],
 *         [
 *             { gate_index: 2, left   }, // w_l[2-#pub] = w_l[1] -> variables[1] = 3 <-- tag = 1
 *             { gate_index: 2, right  }, // w_r[2-#pub] = w_r[1] -> variables[1] = 3
 *         ],
 *         [
 *             { gate_index: 3, left   }, // w_l[3-#pub] = w_l[2] -> variables[2] = 4 <-- tag = 1
 *             { gate_index: 3, right  }, // w_r[3-#pub] = w_r[2] -> variables[2] = 4
 *         ],
 *         [
 *             { gate_index: 4, left   }, // w_l[4-#pub] = w_l[3] -> variables[3] = 5 <-- tag = 1
 *             { gate_index: 4, right  }, // w_r[4-#pub] = w_r[3] -> variables[3] = 5
 *         ],
 *         [
 *             { gate_index: 2, output }, // w_o[2-#pub] = w_o[1] -> variables[4] = 9 <-- tag = 1
 *             { gate_index: 5, left   }, // w_l[5-#pub] = w_l[4] -> variables[4] = 9
 *         ],
 *         [
 *             { gate_index: 3, output }, // w_o[3-#pub] = w_o[2] -> variables[5] = 16 <-- tag = 1
 *             { gate_index: 5, right  }, // w_r[5-#pub] = w_r[4] -> variables[5] = 16
 *         ],
 *         [
 *             { gate_index: 0, left   }, // public_inputs[0] -> w_l[0] -> variables[6] = 25 <-- tag = 1
 *             { gate_index: 0, right  }, // public_inputs[0] -> w_r[0] -> variables[6] = 25
 *             { gate_index: 4, output }, // w_o[4-#pub] = w_o[3] -> variables[6] = 25
 *             { gate_index: 5, output }, // w_o[5-#pub] = w_o[4] -> variables[7] == variables[6] = 25
 *         ],
 *     ]
 *
 *
 * Converting the wire_copy_cycles' objects into coordinates [row #, column #] (l=0, r=1, o=2, 4th = 3), and showing
 * how the cycles permute with arrows:
 * Note: the mappings (permutations) shown here (by arrows) are exactly those expressed by `sigma_mappings`.
 * Note: `-*>` denotes when a sigma_mappings entry has `is_tag=true`.
 * Note: [[ , ]] denotes an entry in sigma_mappings which has been modified due to is_tag=true or
 *       is_public_input=true.
 * Note: `-pub->` denotes a sigma_mappings entry from a left-wire which is a public input.
 *
 * Eg: [i, j] -> [k, l] means sigma_mappings[j][i]
 *    has: subgroup_index = k, column_index = l, is_false = true and is_public_input = false
 * Eg: [i, j] -*> [[k, l]]    means sigma_mappings[j][i]
 *    has: subgroup_index = k, column_index = l, is_tag = true
 * Eg: [i, j] -pub-> [[k, l]] means sigma_mappings[j][i]
 *    has: subgroup_index = k, column_index = l, is_public_input = true
 *
 *     [
 *         [1, 0] -> [1, 1] -> [1, 2] -> [1, 3] -> [2, 3] -> [3, 3] -> [4, 3] -> [5, 3] -*> [[0, 0]],
 *         [2, 0] -> [2, 1] -*> [[0, 0]],
 *         [3, 0] -> [3, 1] -*> [[0, 0]],
 *         [4, 0] -> [4, 1] -*> [[0, 0]],
 *         [2, 2] -> [5, 0] -*> [[0, 2]], <-- the column # (2) is ignored if is_tag=true.
 *         [3, 2] -> [5, 1] -*> [[0, 2]], <-- the column # (2) is ignored if is_tag=true.
 *         [0, 0] -----> [0, 1] -> [4, 2] -> [5, 2] -*> [[0, 0]],
 *                -pub->[[0, 0]]
 *         //         self^  ^ignored when is_public_input=true
 *     ]   ^^^^^^
 *         These are tagged with is_tag=true in `id_mappings`...
 *
 * Notice: the 0th row (gate) is the one public input of our example circuit. Two wires point to this public input:
 * w_l[0] and w_r[0]. The reason _two_ wires do (and not just w_l[0]) is (I think) because of what we see above in
 * the sigma_mappings data.
 * - The [0,0] (w_l[0]) entry of sigma_mappings maps to [0,0], with is_public_input=true set. This is used by
 * permutation.hpp to ensure the correct zeta_0 term is created for the ∆_PI term of the separate
 * "plonk public inputs" paper.
 * - The [0,1] (w_r[0]) entry of sigma_mappings maps to the next wire in the cycle ([4,2]=w_o[4-1]=w_o[3]). This is
 * used to create the correct value for the sigma polynomial at S_σ_2(0).
 *
 * `id_mappings` maps every [row #, column #] to itself, except where is_tag=true where:
 *
 *  [1, 0] -*> [[0, 0]],
 *  [2, 0] -*> [[0, 0]],
 *  [3, 0] -*> [[0, 0]],
 *  [4, 0] -*> [[0, 0]],
 *  [2, 2] -*> [[0, 2]],
 *  [3, 2] -*> [[0, 2]],
 *  [0, 0] -*> [[0, 0]],
 *                  ^this column data is ignored by permutation.hpp when is_tag=true
 *
 *
 *
 * The (subgroup.size() * program_width) elements of sigma_mappings are of the form:
 * {
 *     subgroup_index: j, // iterates over all rows in the subgroup
 *     column_index: i, // l,r,o,4
 *     is_public_input: false,
 *     is_tag: false,
 * }
 *   - sigma_mappings = [
 *         // The i-th index of sigma_mappings is the "column" index (l,r,o,4).
 *         [
 *             // The j-th index of sigma_mappings[i] is the subgroup_index or "row"
 *             {
 *                 subgroup_index: j,
 *                 column_index: i,
 *                 is_public_input: false,
 *                 is_tag: false,
 *             },
 *         ],
 *     ];
 *
 */