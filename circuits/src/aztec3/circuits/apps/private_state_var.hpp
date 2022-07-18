#pragma once
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include "private_state_note.hpp"
#include "private_state_note_preimage.hpp"
#include "private_state_operand.hpp"
#include <aztec3/constants.hpp>

namespace aztec3::circuits::apps {

using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

// template <typename Composer> class PrivateStateFactory;
template <typename Composer> class FunctionExecutionContext;

template <typename Composer> class PrivateStateVar {
  public:
    typedef CircuitTypes<Composer> CT;
    typedef typename CT::fr fr;
    typedef typename CT::address address;
    typedef typename CT::grumpkin_point grumpkin_point;

    FunctionExecutionContext<Composer>*
        exec_ctx; // Pointer, because PrivateStateVar can't hold a reference (because it gets initialised as a member of
                  // a map within PrivateStateFactory, so might not be passed data upon initialisation)

    PrivateStateType private_state_type;
    std::string name;
    fr start_slot;
    grumpkin_point slot_point; /// TODO: make this std::optional (since mapping vars won't have this (only specific
                               /// mapping values will))?

    // A mapping var (in particular) can point to many private_states:
    std::map<NativeTypes::fr, PrivateStateVar<Composer>> private_states;

    bool is_mapping = false;
    std::optional<std::vector<std::string>> mapping_key_names = std::nullopt;

    bool is_partial_slot = false;

    PrivateStateVar(){};

    PrivateStateVar(PrivateStateVar<Composer> const& private_state_var)
        : exec_ctx(private_state_var.exec_ctx)
        , private_state_type(private_state_var.private_state_type)
        , name(private_state_var.name)
        , start_slot(private_state_var.start_slot)
        , slot_point(private_state_var.slot_point)
        , is_mapping(private_state_var.is_mapping)
        , mapping_key_names(private_state_var.mapping_key_names)
        , is_partial_slot(private_state_var.is_partial_slot){};

    // For initialising a basic fr state:
    PrivateStateVar(FunctionExecutionContext<Composer>* exec_ctx,
                    PrivateStateType const& private_state_type,
                    std::string const& name,
                    fr const& start_slot)
        : exec_ctx(exec_ctx)
        , private_state_type(private_state_type)
        , name(name)
        , start_slot(start_slot)
    {
        slot_point = compute_start_slot_point();
    };

    // For initialising a mapping var:
    PrivateStateVar(FunctionExecutionContext<Composer>* exec_ctx,
                    PrivateStateType const& private_state_type,
                    std::string const& name,
                    fr const start_slot,
                    std::vector<std::string> const& mapping_key_names)
        : exec_ctx(exec_ctx)
        , private_state_type(private_state_type)
        , name(name)
        , start_slot(start_slot)
        , is_mapping(mapping_key_names.size() > 0)
        , mapping_key_names(mapping_key_names.size() > 0 ? std::make_optional(mapping_key_names) : std::nullopt)
        , is_partial_slot(true)
    {
        if (mapping_key_names.size() == 0) {
            throw_or_abort("Error. Empty mapping_key_names argument. Try calling the other constructor if you don't "
                           "want to initialise a mapping state.");
        }
    };

    bool operator==(PrivateStateVar<Composer> const&) const = default;

    PrivateStateVar<Composer>& at(std::optional<fr> const& key) { return at(std::vector<std::optional<fr>>{ key }); };

    PrivateStateVar<Composer>& at(std::vector<std::optional<fr>> const& keys);

    std::vector<std::string> get_mapping_key_names() const;

    size_t get_index_of_mapping_key_name(std::string const& mapping_key_name) const;

    PrivateStateNote<Composer> new_note(PrivateStateNotePreimage<CT>& preimage);

    // Arithmetic on private states:
    void add(PrivateStateOperand<CT> const& operand);

    void subtract(PrivateStateOperand<CT> const& operant);

    static std::tuple<NativeTypes::grumpkin_point, bool> compute_slot_point_at_mapping_keys(
        NativeTypes::fr const& start_slot, std::vector<std::optional<NativeTypes::fr>> const& keys);

    std::tuple<grumpkin_point, bool> compute_slot_point_at_mapping_keys(std::vector<std::optional<fr>> const& keys);

  private:
    grumpkin_point compute_start_slot_point();

    // For initialising an fr state from within a mapping:
    PrivateStateVar(FunctionExecutionContext<Composer>* exec_ctx,
                    PrivateStateType const& private_state_type,
                    std::string const& name,
                    fr const& start_slot,
                    grumpkin_point const& slot_point,
                    bool const& is_partial_slot)
        : exec_ctx(exec_ctx)
        , private_state_type(private_state_type)
        , name(name)
        , start_slot(start_slot)
        , slot_point(slot_point)
        , is_partial_slot(is_partial_slot){};

    void arithmetic_checks();
    void validate_operand(PrivateStateOperand<CT> const& operand) const;

    size_t op_count = 0;
};

} // namespace aztec3::circuits::apps

// Importing in this way (rather than explicit instantiation of a template class at the bottom of a .cpp file) preserves
// the following:
// - We retain implicit instantiation of templates, meaning we can pick and choose (with static_assert) which class
// methods support native,
//   circuit or both types.
// - We don't implement method definitions in this file, to avoid a circular dependency with
// function_execution_context.hpp.
#include "private_state_var.tpp"
