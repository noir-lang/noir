#pragma once

#include "call_context.hpp"
#include "contract_deployment_data.hpp"
#include <aztec3/constants.hpp>

#include <barretenberg/common/map.hpp>
#include <barretenberg/crypto/generators/generator_data.hpp>
#include <barretenberg/stdlib/hash/pedersen/pedersen.hpp>
#include <barretenberg/stdlib/primitives/witness/witness.hpp>
#include <aztec3/utils/types/circuit_types.hpp>
#include <aztec3/utils/types/convert.hpp>
#include <aztec3/utils/types/native_types.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> class PrivateCircuitPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

  public:
    CallContext<NCT> call_context = CallContext<NCT>();

    std::array<fr, ARGS_LENGTH> args = { 0 };
    std::array<fr, RETURN_VALUES_LENGTH> return_values = { 0 };

    std::array<fr, EMITTED_EVENTS_LENGTH> emitted_events = { 0 };

    std::array<fr, NEW_COMMITMENTS_LENGTH> new_commitments = { 0 };
    std::array<fr, NEW_NULLIFIERS_LENGTH> new_nullifiers = { 0 };

    std::array<fr, PRIVATE_CALL_STACK_LENGTH> private_call_stack = { 0 };
    std::array<fr, PUBLIC_CALL_STACK_LENGTH> public_call_stack = { 0 };
    std::array<fr, L1_MSG_STACK_LENGTH> l1_msg_stack = { 0 };

    fr historic_private_data_tree_root;
    fr historic_nullifier_tree_root;
    fr historic_contract_tree_root;

    ContractDeploymentData<NCT> contract_deployment_data = ContractDeploymentData<NCT>();

    boolean operator==(PrivateCircuitPublicInputs<NCT> const& other) const
    {
        return call_context == other.call_context && args == other.args && return_values == other.return_values &&
               emitted_events == other.emitted_events && new_commitments == other.new_commitments &&
               new_nullifiers == other.new_nullifiers && private_call_stack == other.private_call_stack &&
               public_call_stack == other.public_call_stack && l1_msg_stack == other.l1_msg_stack &&
               historic_private_data_tree_root == other.historic_private_data_tree_root &&
               historic_nullifier_tree_root == other.historic_nullifier_tree_root &&
               historic_contract_tree_root == other.historic_contract_tree_root &&
               contract_deployment_data == other.contract_deployment_data;
    };

    template <typename Composer>
    PrivateCircuitPublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(composer); };

        PrivateCircuitPublicInputs<CircuitTypes<Composer>> pis = {
            to_circuit_type(call_context),

            to_ct(args),
            to_ct(return_values),

            to_ct(emitted_events),

            to_ct(new_commitments),
            to_ct(new_nullifiers),

            to_ct(private_call_stack),
            to_ct(public_call_stack),
            to_ct(l1_msg_stack),

            to_ct(historic_private_data_tree_root),
            to_ct(historic_nullifier_tree_root),
            to_ct(historic_contract_tree_root),

            to_circuit_type(contract_deployment_data),
        };

        return pis;
    };

    template <typename Composer> PrivateCircuitPublicInputs<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        PrivateCircuitPublicInputs<NativeTypes> pis = {
            to_native_type(call_context),

            to_nt(args),
            to_nt(return_values),

            to_nt(emitted_events),

            to_nt(new_commitments),
            to_nt(new_nullifiers),

            to_nt(private_call_stack),
            to_nt(public_call_stack),
            to_nt(l1_msg_stack),

            to_nt(historic_private_data_tree_root),
            to_nt(historic_nullifier_tree_root),
            to_nt(historic_contract_tree_root),

            to_native_type(contract_deployment_data),
        };

        return pis;
    };

    fr hash() const
    {
        // auto to_hashes = []<typename T>(const T& e) { return e.hash(); };

        std::vector<fr> inputs;

        inputs.push_back(call_context.hash());

        spread_arr_into_vec(args, inputs);
        spread_arr_into_vec(return_values, inputs);

        spread_arr_into_vec(emitted_events, inputs);

        spread_arr_into_vec(new_commitments, inputs);
        spread_arr_into_vec(new_nullifiers, inputs);

        spread_arr_into_vec(private_call_stack, inputs);
        spread_arr_into_vec(public_call_stack, inputs);
        spread_arr_into_vec(l1_msg_stack, inputs);

        inputs.push_back(historic_private_data_tree_root);
        inputs.push_back(historic_nullifier_tree_root);
        inputs.push_back(historic_contract_tree_root);

        inputs.push_back(contract_deployment_data.hash());

        return NCT::compress(inputs, GeneratorIndex::PRIVATE_CIRCUIT_PUBLIC_INPUTS);
    }

    template <size_t SIZE> void spread_arr_into_vec(std::array<fr, SIZE> const& arr, std::vector<fr>& vec) const
    {
        const auto arr_size = sizeof(arr) / sizeof(fr);
        vec.insert(vec.end(), &arr[0], &arr[0] + arr_size);
    }
};

template <typename NCT> void read(uint8_t const*& it, PrivateCircuitPublicInputs<NCT>& private_circuit_public_inputs)
{
    using serialize::read;

    PrivateCircuitPublicInputs<NCT>& pis = private_circuit_public_inputs;
    read(it, pis.call_context);
    read(it, pis.args);
    read(it, pis.return_values);
    read(it, pis.emitted_events);
    read(it, pis.new_commitments);
    read(it, pis.new_nullifiers);
    read(it, pis.private_call_stack);
    read(it, pis.public_call_stack);
    read(it, pis.l1_msg_stack);
    read(it, pis.historic_private_data_tree_root);
    read(it, pis.historic_nullifier_tree_root);
    read(it, pis.historic_contract_tree_root);
    read(it, pis.contract_deployment_data);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf, PrivateCircuitPublicInputs<NCT> const& private_circuit_public_inputs)
{
    using serialize::write;

    PrivateCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;

    write(buf, pis.call_context);
    write(buf, pis.args);
    write(buf, pis.return_values);
    write(buf, pis.emitted_events);
    write(buf, pis.new_commitments);
    write(buf, pis.new_nullifiers);
    write(buf, pis.private_call_stack);
    write(buf, pis.public_call_stack);
    write(buf, pis.l1_msg_stack);
    write(buf, pis.historic_private_data_tree_root);
    write(buf, pis.historic_nullifier_tree_root);
    write(buf, pis.historic_contract_tree_root);

    write(buf, pis.contract_deployment_data);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, PrivateCircuitPublicInputs<NCT> const& private_circuit_public_inputs)

{
    PrivateCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;
    return os << "call_context: " << pis.call_context << "\n"
              << "args: " << pis.args << "\n"
              << "return_values: " << pis.return_values << "\n"
              << "emitted_events: " << pis.emitted_events << "\n"
              << "new_commitments: " << pis.new_commitments << "\n"
              << "new_nullifiers: " << pis.new_nullifiers << "\n"
              << "private_call_stack: " << pis.private_call_stack << "\n"
              << "public_call_stack: " << pis.public_call_stack << "\n"
              << "l1_msg_stack: " << pis.l1_msg_stack << "\n"
              << "historic_private_data_tree_root: " << pis.historic_private_data_tree_root << "\n"
              << "historic_nullifier_tree_root: " << pis.historic_nullifier_tree_root << "\n"
              << "contract_deployment_data: " << pis.contract_deployment_data << "\n"
              << "contract_deployment_data: " << pis.contract_deployment_data << "\n";
}

// It's been extremely useful for all members here to be std::optional. It allows test app circuits to be very
// quickly drafted without worrying about any of the public inputs which aren't relevant to that circuit. Any values
// which aren't set by the circuit can then be safely set to zero when calling `set_public` (by checking for
// std::nullopt)
template <typename NCT> class OptionalPrivateCircuitPublicInputs {
    typedef typename NCT::fr fr;
    typedef typename std::optional<fr> opt_fr;

  public:
    std::optional<CallContext<NCT>> call_context;

    std::array<opt_fr, ARGS_LENGTH> args;
    std::array<opt_fr, RETURN_VALUES_LENGTH> return_values;

    std::array<opt_fr, EMITTED_EVENTS_LENGTH> emitted_events;

    std::array<opt_fr, NEW_COMMITMENTS_LENGTH> new_commitments;
    std::array<opt_fr, NEW_NULLIFIERS_LENGTH> new_nullifiers;

    std::array<opt_fr, PRIVATE_CALL_STACK_LENGTH> private_call_stack;
    std::array<opt_fr, PUBLIC_CALL_STACK_LENGTH> public_call_stack;
    std::array<opt_fr, L1_MSG_STACK_LENGTH> l1_msg_stack;

    opt_fr historic_private_data_tree_root;
    opt_fr historic_nullifier_tree_root;
    opt_fr historic_contract_tree_root;

    std::optional<ContractDeploymentData<NCT>> contract_deployment_data;

    OptionalPrivateCircuitPublicInputs<NCT>(){};

    OptionalPrivateCircuitPublicInputs<NCT>(std::optional<CallContext<NCT>> const& call_context,

                                            std::array<opt_fr, ARGS_LENGTH> const& args,
                                            std::array<opt_fr, RETURN_VALUES_LENGTH> const& return_values,

                                            std::array<opt_fr, EMITTED_EVENTS_LENGTH> const& emitted_events,

                                            std::array<opt_fr, NEW_COMMITMENTS_LENGTH> const& new_commitments,
                                            std::array<opt_fr, NEW_NULLIFIERS_LENGTH> const& new_nullifiers,

                                            std::array<opt_fr, PRIVATE_CALL_STACK_LENGTH> const& private_call_stack,
                                            std::array<opt_fr, PUBLIC_CALL_STACK_LENGTH> const& public_call_stack,
                                            std::array<opt_fr, L1_MSG_STACK_LENGTH> const& l1_msg_stack,

                                            opt_fr const& historic_private_data_tree_root,
                                            opt_fr const& historic_nullifier_tree_root,
                                            opt_fr const& historic_contract_tree_root,

                                            std::optional<ContractDeploymentData<NCT>> const& contract_deployment_data)
        : call_context(call_context)
        , args(args)
        , return_values(return_values)
        , emitted_events(emitted_events)
        , new_commitments(new_commitments)
        , new_nullifiers(new_nullifiers)
        , private_call_stack(private_call_stack)
        , public_call_stack(public_call_stack)
        , l1_msg_stack(l1_msg_stack)
        , historic_private_data_tree_root(historic_private_data_tree_root)
        , historic_nullifier_tree_root(historic_nullifier_tree_root)
        , historic_contract_tree_root(historic_contract_tree_root)
        , contract_deployment_data(contract_deployment_data){};

    bool operator==(OptionalPrivateCircuitPublicInputs<NCT> const&) const = default;

    static OptionalPrivateCircuitPublicInputs<NCT> create()
    {

        auto new_inputs = OptionalPrivateCircuitPublicInputs<NCT>();

        new_inputs.call_context = std::nullopt;

        new_inputs.args.fill(std::nullopt);
        new_inputs.return_values.fill(std::nullopt);

        new_inputs.emitted_events.fill(std::nullopt);

        new_inputs.new_commitments.fill(std::nullopt);
        new_inputs.new_nullifiers.fill(std::nullopt);

        new_inputs.private_call_stack.fill(std::nullopt);
        new_inputs.public_call_stack.fill(std::nullopt);
        new_inputs.l1_msg_stack.fill(std::nullopt);

        new_inputs.historic_private_data_tree_root = std::nullopt;
        new_inputs.historic_nullifier_tree_root = std::nullopt;
        new_inputs.historic_contract_tree_root = std::nullopt;

        new_inputs.contract_deployment_data = std::nullopt;

        return new_inputs;
    };

    void set_commitments(std::vector<fr> commitments)
    {
        if (commitments.size() > new_commitments.size()) {
            throw_or_abort("Too many commitments for the number supported by the public inputs ABI.");
        }
        for (size_t i = 0; i < commitments.size(); ++i) {
            new_commitments[i] = commitments[i];
        }
    }

    void set_nullifiers(std::vector<fr> nullifiers)
    {
        if (nullifiers.size() > new_nullifiers.size()) {
            throw_or_abort("Too many commitments for the number supported by the public inputs ABI.");
        }
        for (size_t i = 0; i < nullifiers.size(); ++i) {
            new_nullifiers[i] = nullifiers[i];
        }
    }

    template <typename Composer> void make_unused_inputs_zero(Composer& composer)
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        make_unused_element_zero(composer, call_context);

        make_unused_array_elements_zero(composer, args);
        make_unused_array_elements_zero(composer, return_values);

        make_unused_array_elements_zero(composer, emitted_events);

        make_unused_array_elements_zero(composer, new_commitments);
        make_unused_array_elements_zero(composer, new_nullifiers);

        make_unused_array_elements_zero(composer, private_call_stack);
        make_unused_array_elements_zero(composer, public_call_stack);
        make_unused_array_elements_zero(composer, l1_msg_stack);

        make_unused_element_zero(composer, historic_private_data_tree_root);
        make_unused_element_zero(composer, historic_nullifier_tree_root);
        make_unused_element_zero(composer, historic_contract_tree_root);

        make_unused_element_zero(composer, contract_deployment_data);

        all_elements_populated = true;
    }

    template <typename Composer> void set_public(Composer& composer)
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        make_unused_inputs_zero(composer);

        // Optional members are guaranteed to be nonempty from here.

        (*call_context).set_public();

        set_array_public(args);
        set_array_public(return_values);

        set_array_public(emitted_events);

        set_array_public(new_commitments);
        set_array_public(new_nullifiers);

        set_array_public(private_call_stack);
        set_array_public(public_call_stack);
        set_array_public(l1_msg_stack);

        (*historic_private_data_tree_root).set_public();
        (*historic_nullifier_tree_root).set_public();
        (*historic_contract_tree_root).set_public();

        (*contract_deployment_data).set_public();
    }

    template <typename Composer>
    OptionalPrivateCircuitPublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) {
            return e ? std::make_optional((*e).to_circuit_type(composer)) : std::nullopt;
        };

        OptionalPrivateCircuitPublicInputs<CircuitTypes<Composer>> pis = {
            to_circuit_type(call_context),

            to_ct(args),
            to_ct(return_values),

            to_ct(emitted_events),

            to_ct(new_commitments),
            to_ct(new_nullifiers),

            to_ct(private_call_stack),
            to_ct(public_call_stack),
            to_ct(l1_msg_stack),

            to_ct(historic_private_data_tree_root),
            to_ct(historic_nullifier_tree_root),
            to_ct(historic_contract_tree_root),

            to_circuit_type(contract_deployment_data),
        };

        return pis;
    };

    template <typename Composer> OptionalPrivateCircuitPublicInputs<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(const std::optional<T>& e) {
            return e ? std::make_optional((*e).template to_native_type<Composer>()) : std::nullopt;
        };
        // auto to_native_type = [&]<typename T>(T& e) { return e.to_native_type(); };

        OptionalPrivateCircuitPublicInputs<NativeTypes> pis = {
            to_native_type(call_context),

            to_nt(args),
            to_nt(return_values),

            to_nt(emitted_events),

            to_nt(new_commitments),
            to_nt(new_nullifiers),

            to_nt(private_call_stack),
            to_nt(public_call_stack),
            to_nt(l1_msg_stack),

            to_nt(historic_private_data_tree_root),
            to_nt(historic_nullifier_tree_root),
            to_nt(historic_contract_tree_root),

            to_native_type(contract_deployment_data),
        };

        return pis;
    };

    fr hash() const
    {
        // auto to_hashes = []<typename T>(const std::optional<T>& e) {
        //     if (!e) {
        //         throw_or_abort("Value is nullopt");
        //     }
        //     return (*e).hash();
        // };

        std::vector<fr> inputs;

        inputs.push_back((*call_context).hash());

        spread_arr_opt_into_vec(args, inputs);
        spread_arr_opt_into_vec(return_values, inputs);

        spread_arr_opt_into_vec(emitted_events, inputs);

        spread_arr_opt_into_vec(new_commitments, inputs);
        spread_arr_opt_into_vec(new_nullifiers, inputs);

        spread_arr_opt_into_vec(private_call_stack, inputs);
        spread_arr_opt_into_vec(public_call_stack, inputs);
        spread_arr_opt_into_vec(l1_msg_stack, inputs);

        inputs.push_back(*historic_private_data_tree_root);
        inputs.push_back(*historic_nullifier_tree_root);
        inputs.push_back(*historic_contract_tree_root);

        inputs.push_back((*contract_deployment_data).hash());

        return NCT::compress(inputs, GeneratorIndex::PRIVATE_CIRCUIT_PUBLIC_INPUTS);
    }

    // We can remove optionality when using the inputs in a kernel or rollup circuit, for ease of use.
    PrivateCircuitPublicInputs<NCT> remove_optionality() const
    {
        auto get_value = [&](auto& e) { return e.value(); };

        return PrivateCircuitPublicInputs<NCT>{
            .call_context = call_context.value(),

            .args = map(args, get_value),
            .return_values = map(return_values, get_value),

            .emitted_events = map(emitted_events, get_value),

            .new_commitments = map(new_commitments, get_value),
            .new_nullifiers = map(new_nullifiers, get_value),

            .private_call_stack = map(private_call_stack, get_value),
            .public_call_stack = map(public_call_stack, get_value),
            .l1_msg_stack = map(l1_msg_stack, get_value),

            .historic_private_data_tree_root = historic_private_data_tree_root.value(),
            .historic_nullifier_tree_root = historic_nullifier_tree_root.value(),
            .historic_contract_tree_root = historic_contract_tree_root.value(),

            .contract_deployment_data = contract_deployment_data.value(),
        };
    }

  private:
    bool all_elements_populated = false;

    template <size_t SIZE>
    void spread_arr_opt_into_vec(std::array<std::optional<fr>, SIZE> const& arr, std::vector<fr>& vec) const
    {
        auto get_opt_value = [](const std::optional<fr>& e) {
            if (!e) {
                throw_or_abort("Value is nullopt");
            }
            return *e;
        };

        std::array<fr, SIZE> arr_values = map(arr, get_opt_value);
        const auto arr_size = sizeof(arr_values) / sizeof(fr);
        vec.insert(vec.end(), &arr_values[0], &arr_values[0] + arr_size);
    }

    template <size_t SIZE> void spread_arr_into_vec(std::array<fr, SIZE> const& arr, std::vector<fr>& vec) const
    {
        const auto arr_size = sizeof(arr) / sizeof(fr);
        vec.insert(vec.end(), &arr[0], &arr[0] + arr_size);
    }

    template <typename Composer, typename T, size_t SIZE>
    void make_unused_array_elements_zero(Composer& composer, std::array<std::optional<T>, SIZE>& arr)
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        for (std::optional<T>& e : arr) {
            make_unused_element_zero(composer, e);
        }
    }

    template <typename Composer, typename T>
    void make_unused_element_zero(Composer& composer, std::optional<T>& element)
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        if (!element) {
            element =
                T(witness_t<Composer>(&composer, 0)); // convert the nullopt value to a circuit witness value of `0`
            fr(*element).assert_is_zero();
        }
    }

    // ABIStruct is a template for any of the structs in the abis/ dir. E.g. ExecutedCallback, CallbackStackItem.
    template <typename Composer, template <class> class ABIStruct>
    void make_unused_element_zero(Composer& composer, std::optional<ABIStruct<CircuitTypes<Composer>>>& element)
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        if (!element) {
            element = ABIStruct<NativeTypes>().to_circuit_type(
                composer); // convert the nullopt value to a circuit witness value of `0`
            (*element).template assert_is_zero<Composer>();
        }
    }

    // Make sure this is only called by functions which have implemented a "CT only" check.
    template <typename T, size_t SIZE> void set_array_public(std::array<std::optional<T>, SIZE>& arr)
    {
        for (std::optional<T>& e : arr) {
            fr(*e).set_public();
        }
    }
}; // namespace aztec3::circuits::abis

template <typename NCT>
void read(uint8_t const*& it, OptionalPrivateCircuitPublicInputs<NCT>& private_circuit_public_inputs)
{
    using serialize::read;

    OptionalPrivateCircuitPublicInputs<NCT>& pis = private_circuit_public_inputs;
    read(it, pis.call_context);
    read(it, pis.args);
    read(it, pis.return_values);
    read(it, pis.emitted_events);
    read(it, pis.new_commitments);
    read(it, pis.new_nullifiers);
    read(it, pis.private_call_stack);
    read(it, pis.public_call_stack);
    read(it, pis.l1_msg_stack);
    read(it, pis.historic_private_data_tree_root);
    read(it, pis.historic_nullifier_tree_root);
    read(it, pis.historic_contract_tree_root);
    read(it, pis.contract_deployment_data);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf, OptionalPrivateCircuitPublicInputs<NCT> const& private_circuit_public_inputs)
{
    using serialize::write;

    OptionalPrivateCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;

    write(buf, pis.call_context);
    write(buf, pis.args);
    write(buf, pis.return_values);
    write(buf, pis.emitted_events);
    write(buf, pis.new_commitments);
    write(buf, pis.new_nullifiers);
    write(buf, pis.private_call_stack);
    write(buf, pis.public_call_stack);
    write(buf, pis.l1_msg_stack);
    write(buf, pis.historic_private_data_tree_root);
    write(buf, pis.historic_nullifier_tree_root);
    write(buf, pis.historic_contract_tree_root);
    write(buf, pis.contract_deployment_data);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, OptionalPrivateCircuitPublicInputs<NCT> const& private_circuit_public_inputs)

{
    OptionalPrivateCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;
    return os << "call_context: " << pis.call_context << "\n"
              << "args: " << pis.args << "\n"
              << "return_values: " << pis.return_values << "\n"
              << "emitted_events: " << pis.emitted_events << "\n"
              << "new_commitments: " << pis.new_commitments << "\n"
              << "new_nullifiers: " << pis.new_nullifiers << "\n"
              << "private_call_stack: " << pis.private_call_stack << "\n"
              << "public_call_stack: " << pis.public_call_stack << "\n"
              << "l1_msg_stack: " << pis.l1_msg_stack << "\n"
              << "historic_private_data_tree_root: " << pis.historic_private_data_tree_root << "\n"
              << "historic_nullifier_tree_root: " << pis.historic_nullifier_tree_root << "\n"
              << "historic_nullifier_tree_root: " << pis.historic_nullifier_tree_root << "\n"
              << "contract_deployment_data: " << pis.contract_deployment_data << "\n";
}

} // namespace aztec3::circuits::abis