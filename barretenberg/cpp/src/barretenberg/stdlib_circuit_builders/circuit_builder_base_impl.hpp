#pragma once
#include "barretenberg/serialize/cbind.hpp"
#include "circuit_builder_base.hpp"

namespace bb {
template <typename FF_> CircuitBuilderBase<FF_>::CircuitBuilderBase(size_t size_hint)
{
    variables.reserve(size_hint * 3);
    variable_names.reserve(size_hint * 3);
    next_var_index.reserve(size_hint * 3);
    prev_var_index.reserve(size_hint * 3);
    real_variable_index.reserve(size_hint * 3);
    real_variable_tags.reserve(size_hint * 3);
}

template <typename FF_> size_t CircuitBuilderBase<FF_>::get_num_gates() const
{
    return num_gates;
}

template <typename FF_> void CircuitBuilderBase<FF_>::print_num_gates() const
{
    std::cout << num_gates << std::endl;
}

template <typename FF_> size_t CircuitBuilderBase<FF_>::get_num_variables() const
{
    return variables.size();
}

template <typename FF_> uint32_t CircuitBuilderBase<FF_>::get_first_variable_in_class(uint32_t index) const
{
    while (prev_var_index[index] != FIRST_VARIABLE_IN_CLASS) {
        index = prev_var_index[index];
    }
    return index;
}

template <typename FF_>
void CircuitBuilderBase<FF_>::update_real_variable_indices(uint32_t index, uint32_t new_real_index)
{
    auto cur_index = index;
    do {
        real_variable_index[cur_index] = new_real_index;
        cur_index = next_var_index[cur_index];
    } while (cur_index != REAL_VARIABLE);
}

template <typename FF_> uint32_t CircuitBuilderBase<FF_>::get_public_input_index(const uint32_t witness_index) const
{
    uint32_t result = static_cast<uint32_t>(-1);
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        if (real_variable_index[public_inputs[i]] == real_variable_index[witness_index]) {
            result = static_cast<uint32_t>(i);
            break;
        }
    }
    ASSERT(result != static_cast<uint32_t>(-1));
    return result;
}

template <typename FF_>
typename CircuitBuilderBase<FF_>::FF CircuitBuilderBase<FF_>::get_public_input(const uint32_t index) const
{
    return get_variable(public_inputs[index]);
}

template <typename FF_>
std::vector<typename CircuitBuilderBase<FF_>::FF> CircuitBuilderBase<FF_>::get_public_inputs() const
{
    std::vector<FF> result;
    for (uint32_t i = 0; i < get_num_public_inputs(); ++i) {
        result.push_back(get_public_input(i));
    }
    return result;
}

template <typename FF_> uint32_t CircuitBuilderBase<FF_>::add_variable(const FF& in)
{
    variables.emplace_back(in);
    const uint32_t index = static_cast<uint32_t>(variables.size()) - 1U;
    real_variable_index.emplace_back(index);
    next_var_index.emplace_back(REAL_VARIABLE);
    prev_var_index.emplace_back(FIRST_VARIABLE_IN_CLASS);
    real_variable_tags.emplace_back(DUMMY_TAG);
    return index;
}

template <typename FF_> void CircuitBuilderBase<FF_>::set_variable_name(uint32_t index, const std::string& name)
{
    ASSERT(variables.size() > index);
    uint32_t first_idx = get_first_variable_in_class(index);

    if (variable_names.contains(first_idx)) {
        failure("Attempted to assign a name to a variable that already has a name");
        return;
    }
    variable_names.insert({ first_idx, name });
}

template <typename FF_> void CircuitBuilderBase<FF_>::update_variable_names(uint32_t index)
{
    uint32_t first_idx = get_first_variable_in_class(index);

    uint32_t cur_idx = next_var_index[first_idx];
    while (cur_idx != REAL_VARIABLE && !variable_names.contains(cur_idx)) {
        cur_idx = next_var_index[cur_idx];
    }

    if (variable_names.contains(first_idx)) {
        if (cur_idx != REAL_VARIABLE) {
            variable_names.extract(cur_idx);
        }
        return;
    }

    if (cur_idx != REAL_VARIABLE) {
        std::string var_name = variable_names.find(cur_idx)->second;
        variable_names.erase(cur_idx);
        variable_names.insert({ first_idx, var_name });
        return;
    }
    failure("No previously assigned names found");
}

template <typename FF_> void CircuitBuilderBase<FF_>::finalize_variable_names()
{
    std::vector<uint32_t> keys;
    std::vector<uint32_t> firsts;

    for (auto& tup : variable_names) {
        keys.push_back(tup.first);
        firsts.push_back(get_first_variable_in_class(tup.first));
    }

    for (size_t i = 0; i < keys.size() - 1; i++) {
        for (size_t j = i + 1; j < keys.size(); i++) {
            uint32_t first_idx_a = firsts[i];
            uint32_t first_idx_b = firsts[j];
            if (first_idx_a == first_idx_b) {
                std::string substr1 = variable_names[keys[i]];
                std::string substr2 = variable_names[keys[j]];
                failure("Variables from the same equivalence class have separate names: " + substr2 + ", " + substr2);
                update_variable_names(first_idx_b);
            }
        }
    }
}

template <typename FF_> size_t CircuitBuilderBase<FF_>::get_circuit_subgroup_size(const size_t num_gates) const
{
    auto log2_n = static_cast<size_t>(numeric::get_msb(num_gates));
    if ((1UL << log2_n) != (num_gates)) {
        ++log2_n;
    }
    return 1UL << log2_n;
}

template <typename FF_> msgpack::sbuffer CircuitBuilderBase<FF_>::export_circuit()
{
    info("not implemented");
    return { 0 };
}

template <typename FF_> uint32_t CircuitBuilderBase<FF_>::add_public_variable(const FF& in)
{
    const uint32_t index = add_variable(in);
    public_inputs.emplace_back(index);
    return index;
}

template <typename FF_> void CircuitBuilderBase<FF_>::set_public_input(const uint32_t witness_index)
{
    for (const uint32_t public_input : public_inputs) {
        if (public_input == witness_index) {
            if (!failed()) {
                failure("Attempted to set a public input that is already public!");
            }
            return;
        }
    }
    public_inputs.emplace_back(witness_index);
}

/**
 * Join variable class b to variable class a.
 *
 * @param a_variable_idx Index of a variable in class a.
 * @param b_variable_idx Index of a variable in class b.
 * @param msg Class tag.
 * */
template <typename FF>
void CircuitBuilderBase<FF>::assert_equal(const uint32_t a_variable_idx,
                                          const uint32_t b_variable_idx,
                                          std::string const& msg)
{
    assert_valid_variables({ a_variable_idx, b_variable_idx });
    bool values_equal = (get_variable(a_variable_idx) == get_variable(b_variable_idx));
    if (!values_equal && !failed()) {
        failure(msg);
    }
    uint32_t a_real_idx = real_variable_index[a_variable_idx];
    uint32_t b_real_idx = real_variable_index[b_variable_idx];
    // If a==b is already enforced, exit method
    if (a_real_idx == b_real_idx)
        return;
    // Otherwise update the real_idx of b-chain members to that of a

    auto b_start_idx = get_first_variable_in_class(b_variable_idx);
    update_real_variable_indices(b_start_idx, a_real_idx);
    // Now merge equivalence classes of a and b by tying last (= real) element of b-chain to first element of a-chain
    auto a_start_idx = get_first_variable_in_class(a_variable_idx);
    next_var_index[b_real_idx] = a_start_idx;
    prev_var_index[a_start_idx] = b_real_idx;
    bool no_tag_clash = (real_variable_tags[a_real_idx] == DUMMY_TAG || real_variable_tags[b_real_idx] == DUMMY_TAG ||
                         real_variable_tags[a_real_idx] == real_variable_tags[b_real_idx]);
    if (!no_tag_clash && !failed()) {
        failure(msg);
    }
    if (real_variable_tags[a_real_idx] == DUMMY_TAG) {
        real_variable_tags[a_real_idx] = real_variable_tags[b_real_idx];
    }
}

template <typename FF_>
void CircuitBuilderBase<FF_>::assert_valid_variables(const std::vector<uint32_t>& variable_indices)
{
    for (const auto& variable_index : variable_indices) {
        ASSERT(is_valid_variable(variable_index));
    }
}

template <typename FF_>
void CircuitBuilderBase<FF_>::add_recursive_proof(const AggregationObjectIndices& proof_output_witness_indices)
{
    if (contains_recursive_proof) {
        failure("added recursive proof when one already exists");
    }
    contains_recursive_proof = true;

    size_t i = 0;
    for (const auto& idx : proof_output_witness_indices) {
        set_public_input(idx);
        recursive_proof_public_input_indices[i] = static_cast<uint32_t>(public_inputs.size() - 1);
        ++i;
    }
}

template <typename FF_>
void CircuitBuilderBase<FF_>::set_recursive_proof(const AggregationObjectIndices& proof_output_witness_indices)
{
    if (contains_recursive_proof) {
        failure("added recursive proof when one already exists");
    }
    contains_recursive_proof = true;
    for (size_t i = 0; i < proof_output_witness_indices.size(); ++i) {
        recursive_proof_public_input_indices[i] =
            get_public_input_index(real_variable_index[proof_output_witness_indices[i]]);
    }
}

template <typename FF_> bool CircuitBuilderBase<FF_>::failed() const
{
    return _failed;
}

template <typename FF_> const std::string& CircuitBuilderBase<FF_>::err() const
{
    return _err;
}

template <typename FF_> void CircuitBuilderBase<FF_>::set_err(std::string msg)
{
    _err = std::move(msg);
}

template <typename FF_> void CircuitBuilderBase<FF_>::failure(std::string msg)
{
    _failed = true;
    set_err(std::move(msg));
}
} // namespace bb