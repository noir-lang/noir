#include <stdint.h>
#include <stddef.h>
#include <vector>
#include <proof_system/proving_key/proving_key.hpp>
namespace honk {
// Enum values spaced in increments of 30-bits (multiples of 2 ** 30).
enum WireType { LEFT = 0U, RIGHT = (1U << 30U), OUTPUT = (1U << 31U), FOURTH = 0xc0000000 };

/**
 * @brief cycle_node represents a particular witness at a particular gate. Used to collect permutation sets
 *
 */
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
typedef std::vector<std::vector<cycle_node>> CycleCollector;

/**
 * Compute wire copy cycles
 *
 * First set all wire_copy_cycles corresponding to public_inputs to point to themselves.
 * Then go through all witnesses in w_l, w_r, w_o and w_4 (if program width is > 3) and
 * add them to cycles of their real indexes.
 *
 * @tparam program_width Program width
 * */
template <size_t program_width, typename CircuitConstructor>
void compute_wire_copy_cycles(CircuitConstructor& circuit_constructor, CycleCollector& wire_copy_cycles)
{
    // Reference circuit constructor members
    const std::vector<uint32_t>& real_variable_index = circuit_constructor.real_variable_index;
    const std::vector<uint32_t>& public_inputs = circuit_constructor.public_inputs;

    const std::vector<uint32_t>& w_l = circuit_constructor.w_l;
    const std::vector<uint32_t>& w_r = circuit_constructor.w_r;
    const std::vector<uint32_t>& w_o = circuit_constructor.w_o;
    const size_t n = circuit_constructor.n;
    const std::vector<uint32_t>& w_4 = circuit_constructor.w_4;

    size_t number_of_cycles = 0;

    const size_t num_public_inputs = public_inputs.size();

    // Initialize wire_copy_cycles of public input variables to point to themselves ( we could actually ignore this step
    // for HONK because of the way we construct the permutation)
    for (size_t counter = 0; counter < num_public_inputs; ++counter) {
        size_t i = num_public_inputs - 1 - counter;
        cycle_node left{ static_cast<uint32_t>(i), WireType::LEFT };
        cycle_node right{ static_cast<uint32_t>(i), WireType::RIGHT };

        const auto public_input_index = real_variable_index[public_inputs[i]];
        if (static_cast<size_t>(public_input_index) >= number_of_cycles) {
            wire_copy_cycles.resize(public_input_index + 1);
        }
        std::vector<cycle_node>& cycle = wire_copy_cycles[static_cast<size_t>(public_input_index)];
        // These two nodes must be in adjacent locations in the cycle for correct handling of public inputs
        cycle.emplace_back(left);
        cycle.emplace_back(right);
    }

    // Go through all witnesses and add them to the wire_copy_cycles
    for (size_t counter = 0; counter < n; ++counter) {
        // Start from the back. This way we quickly get the maximum real variable index
        size_t i = n - 1 - counter;
        const uint32_t w_1_index = real_variable_index[w_l[i]];
        const uint32_t w_2_index = real_variable_index[w_r[i]];
        const uint32_t w_3_index = real_variable_index[w_o[i]];
        // Check the maximum index of a variable. If it is more or equal to the cycle vector size, extend the vector
        const uint32_t max_index = std::max({ w_1_index, w_2_index, w_3_index });
        if (max_index >= number_of_cycles) {
            wire_copy_cycles.resize(max_index + 1);
            number_of_cycles = max_index + 1;
        }
        wire_copy_cycles[static_cast<size_t>(w_1_index)].emplace_back(static_cast<uint32_t>(i + num_public_inputs),
                                                                      WireType::LEFT);
        wire_copy_cycles[static_cast<size_t>(w_2_index)].emplace_back(static_cast<uint32_t>(i + num_public_inputs),
                                                                      WireType::RIGHT);
        wire_copy_cycles[static_cast<size_t>(w_3_index)].emplace_back(static_cast<uint32_t>(i + num_public_inputs),
                                                                      WireType::OUTPUT);

        // Handle width 4 separately
        if constexpr (program_width > 3) {
            static_assert(program_width == 4);
            const auto w_4_index = real_variable_index[w_4[i]];
            if (w_4_index >= number_of_cycles) {
                wire_copy_cycles.resize(w_4_index + 1);
            }
            wire_copy_cycles[static_cast<size_t>(w_4_index)].emplace_back(static_cast<uint32_t>(i + num_public_inputs),
                                                                          WireType::FOURTH);
        }
    }
}

/**
 * @brief Compute sigma permutations for standard honk and put them into polynomial cache
 *
 * @details These permutations don't involve sets. We only care about equating one witness value to another. The
 * sequences don't use cosets unlike FFT-based Plonk, because there is no need for them. We simply use indices based on
 * the witness vector and index within the vector. These values are permuted to account for wire copy cycles
 *
 * @tparam program_width
 * @tparam CircuitConstructor
 * @param circuit_constructor
 * @param key
 */
template <size_t program_width, typename CircuitConstructor>
void compute_standard_honk_sigma_permutations(CircuitConstructor& circuit_constructor, waffle::proving_key* key)
{
    // Compute wire copy cycles for public and private variables
    CycleCollector wire_copy_cycles;
    compute_wire_copy_cycles<program_width>(circuit_constructor, wire_copy_cycles);
    const size_t n = key->n;
    // Fill sigma polynomials with default values
    std::vector<barretenberg::polynomial> sigma_polynomials_lagrange;
    for (size_t i = 0; i < program_width; ++i) {
        // Construct permutation polynomials in lagrange base
        std::string index = std::to_string(i + 1);
        sigma_polynomials_lagrange.push_back(barretenberg::polynomial(key->n));
        barretenberg::polynomial& sigma_polynomial_lagrange = sigma_polynomials_lagrange[i];
        for (size_t j = 0; j < key->n; j++) {
            sigma_polynomial_lagrange[j] = (i * n + j);
        }
    }
    // Go through each cycle
    for (auto& single_copy_cycle : wire_copy_cycles) {

        // If we use assert equal, we lose a real variable index, which creates an empty cycle
        if (single_copy_cycle.size() == 0) {
            continue;
        }
        size_t cycle_size = single_copy_cycle.size();
        // Get the index value of the last element
        cycle_node current_element = single_copy_cycle[cycle_size - 1];
        auto last_index =
            sigma_polynomials_lagrange[current_element.wire_type >> 30].data()[current_element.gate_index];

        // Propagate indices through the cycle
        for (size_t j = 0; j < cycle_size; j++) {

            current_element = single_copy_cycle[j];
            auto temp_index =
                sigma_polynomials_lagrange[current_element.wire_type >> 30].data()[current_element.gate_index];
            sigma_polynomials_lagrange[current_element.wire_type >> 30].data()[current_element.gate_index] = last_index;
            last_index = temp_index;
        }
    }
    // Save to polynomial cache
    for (size_t i = 0; i < program_width; i++) {
        std::string index = std::to_string(i + 1);
        key->polynomial_cache.put("sigma_" + index + "_lagrange", std::move(sigma_polynomials_lagrange[i]));
    }
}

/**
 * @brief Compute standard honk id polynomials and put them into cache
 *
 * @details Honk permutations involve using id and sigma polynomials to generate variable cycles. This function
 * generates the id polynomials and puts them into polynomial cache, so that they can be used by the prover.
 *
 * @tparam program_width The number of witness polynomials
 * @param key Proving key where we will save the polynomials
 */
// TODO(Cody): why not a shared pointer here?/s
template <size_t program_width>
void compute_standard_honk_id_polynomials(auto key) // proving_key* and share_ptr<proving_key>
{
    const size_t n = key->n;
    // Fill id polynomials with default values
    std::vector<barretenberg::polynomial> id_polynomials_lagrange;
    for (size_t i = 0; i < program_width; ++i) {
        // Construct permutation polynomials in lagrange base
        std::string index = std::to_string(i + 1);
        id_polynomials_lagrange.push_back(barretenberg::polynomial(key->n));
        barretenberg::polynomial& id_polynomial_lagrange = id_polynomials_lagrange[i];
        for (size_t j = 0; j < key->n; j++) {
            id_polynomial_lagrange[j] = (i * n + j);
        }
    }
    // Save to polynomial cache
    for (size_t i = 0; i < program_width; i++) {
        std::string index = std::to_string(i + 1);
        key->polynomial_cache.put("id_" + index + "_lagrange", std::move(id_polynomials_lagrange[i]));
    }
}

/**
 * @brief Compute Lagrange Polynomials L_0 and L_{n-1} and put them in the polynomial cache
 *
 * @param key Proving key where we will save the polynomials
 */
inline void compute_first_and_last_lagrange_polynomials(auto key) // proving_key* and share_ptr<proving_key>
{
    const size_t n = key->n;
    // info("Computing Lagrange basis polys, the  value of n is: ",/s n);
    barretenberg::polynomial lagrange_polynomial_0(n, n);
    barretenberg::polynomial lagrange_polynomial_n_min_1(n, n);
    lagrange_polynomial_0[0] = 1;
    lagrange_polynomial_n_min_1[n - 1] = 1;
    key->polynomial_cache.put("L_first_lagrange", std::move(lagrange_polynomial_0));
    key->polynomial_cache.put("L_last_lagrange", std::move(lagrange_polynomial_n_min_1));
}

} // namespace honk
