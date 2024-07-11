#pragma once
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/gate_data.hpp"
#include <msgpack/sbuffer_decl.hpp>
#include <utility>

#include <unordered_map>

namespace bb {
static constexpr uint32_t DUMMY_TAG = 0;

template <typename FF_> class CircuitBuilderBase {
  public:
    using FF = FF_;
    using EmbeddedCurve = std::conditional_t<std::same_as<FF, bb::g1::coordinate_field>, curve::BN254, curve::Grumpkin>;

    size_t num_gates = 0;

    std::vector<uint32_t> public_inputs;
    std::vector<FF> variables;
    std::unordered_map<uint32_t, std::string> variable_names;

    // index of next variable in equivalence class (=REAL_VARIABLE if you're last)
    std::vector<uint32_t> next_var_index;
    // index of  previous variable in equivalence class (=FIRST if you're in a cycle alone)
    std::vector<uint32_t> prev_var_index;
    // indices of corresponding real variables
    std::vector<uint32_t> real_variable_index;
    std::vector<uint32_t> real_variable_tags;
    uint32_t current_tag = DUMMY_TAG;
    // The permutation on variable tags. See
    // https://github.com/AztecProtocol/plonk-with-lookups-private/blob/new-stuff/GenPermuations.pdf
    // DOCTODO(#231): replace with the relevant wiki link.
    std::map<uint32_t, uint32_t> tau;

    // Public input indices which contain recursive proof information
    std::vector<uint32_t> recursive_proof_public_input_indices;
    bool contains_recursive_proof = false;

    // We only know from the circuit description whether a circuit should use a prover which produces
    // proofs that are friendly to verify in a circuit themselves. However, a verifier does not need a full circuit
    // description and should be able to verify a proof with just the verification key and the proof.
    // This field exists to later set the same field in the verification key, and make sure
    // that we are using the correct prover/verifier.
    bool is_recursive_circuit = false;

    bool _failed = false;
    std::string _err;
    static constexpr uint32_t REAL_VARIABLE = UINT32_MAX - 1;
    static constexpr uint32_t FIRST_VARIABLE_IN_CLASS = UINT32_MAX - 2;

    CircuitBuilderBase(size_t size_hint = 0);

    CircuitBuilderBase(const CircuitBuilderBase& other) = default;
    CircuitBuilderBase(CircuitBuilderBase&& other) noexcept = default;
    CircuitBuilderBase& operator=(const CircuitBuilderBase& other) = default;
    CircuitBuilderBase& operator=(CircuitBuilderBase&& other) noexcept = default;
    virtual ~CircuitBuilderBase() = default;

    bool operator==(const CircuitBuilderBase& other) const = default;

    virtual size_t get_num_gates() const;
    virtual void print_num_gates() const;
    virtual size_t get_num_variables() const;
    // TODO(#216)(Adrian): Feels wrong to let the zero_idx be changed.
    uint32_t zero_idx = 0;
    uint32_t one_idx = 1;

    virtual void create_add_gate(const add_triple_<FF>& in) = 0;
    virtual void create_mul_gate(const mul_triple_<FF>& in) = 0;
    virtual void create_bool_gate(const uint32_t a) = 0;
    virtual void create_poly_gate(const poly_triple_<FF>& in) = 0;
    virtual size_t get_num_constant_gates() const = 0;

    /**
     * Get the index of the first variable in class.
     *
     * @param index The index of the variable you want to look up.
     *
     * @return The index of the first variable in the same class as the submitted index.
     * */
    uint32_t get_first_variable_in_class(uint32_t index) const;
    /**
     * Update all variables from index in equivalence class to have real variable new_real_index.
     *
     * @param index The index of a variable in the class we're updating.
     * @param new_real_index The index of the real variable to update to.
     * */
    void update_real_variable_indices(uint32_t index, uint32_t new_real_index);

    /**
     * Get the value of the variable v_{index}.
     *
     * @param index The index of the variable.
     * @return The value of the variable.
     * */
    inline FF get_variable(const uint32_t index) const
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
    inline const FF& get_variable_reference(const uint32_t index) const
    {
        ASSERT(variables.size() > index);
        return variables[real_variable_index[index]];
    }

    uint32_t get_public_input_index(const uint32_t witness_index) const;

    FF get_public_input(const uint32_t index) const;

    std::vector<FF> get_public_inputs() const;

    /**
     * Add a variable to variables
     *
     * @param in The value of the variable
     * @return The index of the new variable in the variables vector
     */
    virtual uint32_t add_variable(const FF& in);

    /**
     * Assign a name to a variable(equivalence class). Should be one name per equivalence class.
     *
     * @param index Index of the variable you want to name.
     * @param name  Name of the variable.
     *
     */
    virtual void set_variable_name(uint32_t index, const std::string& name);

    /**
     * After assert_equal() merge two class names if present.
     * Preserves the first name in class.
     *
     * @param index Index of the variable you have previously named and used in assert_equal.
     *
     */
    virtual void update_variable_names(uint32_t index);

    /**
     * After finishing the circuit can be called for automatic merging
     * all existing collisions.
     *
     */
    virtual void finalize_variable_names();

    /**
     * Export the existing circuit as msgpack compatible buffer.
     *
     * @return msgpack compatible buffer
     */
    virtual msgpack::sbuffer export_circuit();

    /**
     * Add a public variable to variables
     *
     * The only difference between this and add_variable is that here it is
     * also added to the public_inputs vector
     *
     * @param in The value of the variable
     * @return The index of the new variable in the variables vector
     */
    virtual uint32_t add_public_variable(const FF& in);

    /**
     * Make a witness variable public.
     *
     * @param witness_index The index of the witness.
     * */
    virtual void set_public_input(uint32_t witness_index);
    virtual void assert_equal(uint32_t a_idx, uint32_t b_idx, std::string const& msg = "assert_equal");

    // TODO(#216)(Adrian): This method should belong in the ComposerHelper, where the number of reserved gates can be
    // correctly set.
    size_t get_circuit_subgroup_size(size_t num_gates) const;

    size_t get_num_public_inputs() const { return public_inputs.size(); }

    // Check whether each variable index points to a witness in the composer
    //
    // Any variable whose index does not point to witness value is deemed invalid.
    //
    // This implicitly checks whether a variable index
    // is equal to IS_CONSTANT; assuming that we will never have
    // uint32::MAX number of variables
    void assert_valid_variables(const std::vector<uint32_t>& variable_indices);
    bool is_valid_variable(uint32_t variable_index) { return variable_index < variables.size(); };

    /**
     * @brief Add information about which witnesses contain the recursive proof computation information
     *
     * @param circuit_constructor Object with the circuit
     * @param proof_output_witness_indices Witness indices that need to become public and stored as recurisve proof
     * specific
     */
    void add_recursive_proof(const std::vector<uint32_t>& proof_output_witness_indices);

    /**
     * TODO: We can remove this and use `add_recursive_proof` once my question has been addressed
     * TODO: using `add_recursive_proof` also means that we will need to remove the cde which is
     * TODO: adding the public_inputs
     * @brief Update recursive_proof_public_input_indices with existing public inputs that represent a recursive proof
     *
     * @param proof_output_witness_indices
     */
    void set_recursive_proof(const std::vector<uint32_t>& proof_output_witness_indices);

    bool failed() const;
    const std::string& err() const;

    void set_err(std::string msg);
    void failure(std::string msg);
};

/**
 * @brief Serialized state of a circuit
 *
 * @details Used to transfer the state of the circuit
 * to Symbolic Circuit class.
 * Symbolic circuit is then used to produce SMT statements
 * that describe needed properties of the circuit.
 *
 * @param modulus Modulus of the field we are working with
 * @param public_inps Public inputs to the current circuit
 * @param vars_of_interest Map wires indices to their given names
 * @param variables List of wires values in the current circuit
 * @param selectors List of selectors in the current circuit
 * @param wires List of wires indices for each selector
 * @param real_variable_index Encoded copy constraints
 * @param lookup_tables List of lookup tables
 * @param real_variable_tag Variables' tags for range constraints
 * @param range_lists Existing range lists
 */
template <typename FF> struct CircuitSchemaInternal {
    std::string modulus;
    std::vector<uint32_t> public_inps;
    std::unordered_map<uint32_t, std::string> vars_of_interest;
    std::vector<FF> variables;
    std::vector<std::vector<std::vector<FF>>> selectors;
    std::vector<std::vector<std::vector<uint32_t>>> wires;
    std::vector<uint32_t> real_variable_index;
    std::vector<std::vector<std::vector<FF>>> lookup_tables;
    std::vector<uint32_t> real_variable_tags;
    std::unordered_map<uint32_t, uint64_t> range_tags;
    MSGPACK_FIELDS(modulus,
                   public_inps,
                   vars_of_interest,
                   variables,
                   selectors,
                   wires,
                   real_variable_index,
                   lookup_tables,
                   real_variable_tags,
                   range_tags);
};
} // namespace bb

// TODO(#217)(Cody): This will need updating based on the approach we take to ensure no multivariate is zero.
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
 *    has: row_index = k, column_index = l, is_false = true and is_public_input = false
 * Eg: [i, j] -*> [[k, l]]    means sigma_mappings[j][i]
 *    has: row_index = k, column_index = l, is_tag = true
 * Eg: [i, j] -pub-> [[k, l]] means sigma_mappings[j][i]
 *    has: row_index = k, column_index = l, is_public_input = true
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
 *     row_index: j, // iterates over all rows in the subgroup
 *     column_index: i, // l,r,o,4
 *     is_public_input: false,
 *     is_tag: false,
 * }
 *   - sigma_mappings = [
 *         // The i-th index of sigma_mappings is the "column" index (l,r,o,4).
 *         [
 *             // The j-th index of sigma_mappings[i] is the row_index or "row"
 *             {
 *                 row_index: j,
 *                 column_index: i,
 *                 is_public_input: false,
 *                 is_tag: false,
 *             },
 *         ],
 *     ];
 *
 */
