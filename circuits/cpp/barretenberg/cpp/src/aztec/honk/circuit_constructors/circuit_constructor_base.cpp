#include "circuit_constructor_base.hpp"

namespace waffle {

/**
 * Join variable class b to variable class a.
 *
 * @param a_variable_idx Index of a variable in class a.
 * @param b_variable_idx Index of a variable in class b.
 * @param msg Class tag.
 * */
template <size_t program_width_>
void CircuitConstructorBase<program_width_>::assert_equal(const uint32_t a_variable_idx,
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
    if (real_variable_tags[a_real_idx] == DUMMY_TAG)
        real_variable_tags[a_real_idx] = real_variable_tags[b_real_idx];
}
// Standard honk/ plonk instantiation
template class CircuitConstructorBase<3>;
} // namespace waffle
