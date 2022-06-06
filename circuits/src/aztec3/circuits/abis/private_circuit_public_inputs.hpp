#pragma once
// #include <stdlib/hash/pedersen/pedersen.hpp>
#include <common/map.hpp>
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>
#include "../../constants.hpp"
#include "call_context.hpp"
#include "executed_callback.hpp"
#include "callback_stack_item.hpp"
#include <crypto/pedersen/generator_data.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
namespace aztec3::circuits::abis {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;

template <typename NCT> class PrivateCircuitPublicInputs {
    typedef typename NCT::address address;
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;
    typedef typename std::optional<fr> opt_fr;
    typedef typename std::optional<boolean> opt_boolean;

  public:
    std::optional<CallContext<NCT>> call_context;

    std::array<opt_fr, CUSTOM_PUBLIC_INPUTS_LENGTH> custom_public_inputs;
    std::array<opt_fr, EMITTED_PUBLIC_INPUTS_LENGTH> emitted_public_inputs;

    std::optional<ExecutedCallback<NCT>> executed_callback;

    std::array<opt_fr, OUTPUT_COMMITMENTS_LENGTH> output_commitments;
    std::array<opt_fr, INPUT_NULLIFIERS_LENGTH> input_nullifiers;

    std::array<opt_fr, PRIVATE_CALL_STACK_LENGTH> private_call_stack;
    std::array<opt_fr, PUBLIC_CALL_STACK_LENGTH> public_call_stack;
    std::array<opt_fr, CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH> contract_deployment_call_stack;
    std::array<opt_fr, PARTIAL_L1_CALL_STACK_LENGTH> partial_l1_call_stack;
    std::array<std::optional<CallbackStackItem<NCT>>, CALLBACK_STACK_LENGTH> callback_stack;

    opt_fr old_private_data_tree_root;

    opt_boolean is_fee_payment;
    opt_boolean pay_fee_from_l1;
    opt_boolean pay_fee_from_public_l2;
    opt_boolean called_from_l1;

    PrivateCircuitPublicInputs<NCT>(){};

    PrivateCircuitPublicInputs<NCT>(
        std::optional<CallContext<NCT>> const& call_context,

        std::array<opt_fr, CUSTOM_PUBLIC_INPUTS_LENGTH> const& custom_public_inputs,
        std::array<opt_fr, EMITTED_PUBLIC_INPUTS_LENGTH> const& emitted_public_inputs,

        std::optional<ExecutedCallback<NCT>> const& executed_callback,

        std::array<opt_fr, OUTPUT_COMMITMENTS_LENGTH> const& output_commitments,
        std::array<opt_fr, INPUT_NULLIFIERS_LENGTH> const& input_nullifiers,

        std::array<opt_fr, PRIVATE_CALL_STACK_LENGTH> const& private_call_stack,
        std::array<opt_fr, PUBLIC_CALL_STACK_LENGTH> const& public_call_stack,
        std::array<opt_fr, CONTRACT_DEPLOYMENT_CALL_STACK_LENGTH> const& contract_deployment_call_stack,
        std::array<opt_fr, PARTIAL_L1_CALL_STACK_LENGTH> const& partial_l1_call_stack,
        std::array<std::optional<CallbackStackItem<NCT>>, CALLBACK_STACK_LENGTH> const& callback_stack,

        opt_fr const& old_private_data_tree_root,

        opt_boolean const& is_fee_payment,
        opt_boolean const& pay_fee_from_l1,
        opt_boolean const& pay_fee_from_public_l2,
        opt_boolean const& called_from_l1)
        : call_context(call_context)
        , custom_public_inputs(custom_public_inputs)
        , emitted_public_inputs(emitted_public_inputs)
        , executed_callback(executed_callback)
        , output_commitments(output_commitments)
        , input_nullifiers(input_nullifiers)
        , private_call_stack(private_call_stack)
        , public_call_stack(public_call_stack)
        , contract_deployment_call_stack(contract_deployment_call_stack)
        , partial_l1_call_stack(partial_l1_call_stack)
        , callback_stack(callback_stack)
        , old_private_data_tree_root(old_private_data_tree_root)
        , is_fee_payment(is_fee_payment)
        , pay_fee_from_l1(pay_fee_from_l1)
        , pay_fee_from_public_l2(pay_fee_from_public_l2)
        , called_from_l1(called_from_l1){};

    bool operator==(PrivateCircuitPublicInputs<NCT> const&) const = default;

    static PrivateCircuitPublicInputs<NCT> create()
    {

        auto new_inputs = PrivateCircuitPublicInputs<NCT>();

        new_inputs.call_context = std::nullopt;

        new_inputs.custom_public_inputs.fill(std::nullopt);
        new_inputs.emitted_public_inputs.fill(std::nullopt);

        new_inputs.executed_callback = std::nullopt;

        new_inputs.output_commitments.fill(std::nullopt);
        new_inputs.input_nullifiers.fill(std::nullopt);

        new_inputs.private_call_stack.fill(std::nullopt);
        new_inputs.public_call_stack.fill(std::nullopt);
        new_inputs.contract_deployment_call_stack.fill(std::nullopt);
        new_inputs.partial_l1_call_stack.fill(std::nullopt);
        new_inputs.callback_stack.fill(std::nullopt);

        new_inputs.old_private_data_tree_root = std::nullopt;

        return new_inputs;
    };

    void set_commitments(std::vector<fr> commitments)
    {
        if (commitments.size() > output_commitments.size()) {
            throw_or_abort("Too many commitments for the number supported by the public inputs ABI.");
        }
        for (size_t i = 0; i < commitments.size(); ++i) {
            output_commitments[i] = commitments[i];
        }
    }

    void set_nullifiers(std::vector<fr> nullifiers)
    {
        if (nullifiers.size() > input_nullifiers.size()) {
            throw_or_abort("Too many commitments for the number supported by the public inputs ABI.");
        }
        for (size_t i = 0; i < nullifiers.size(); ++i) {
            input_nullifiers[i] = nullifiers[i];
        }
    }

    template <typename Composer> void make_unused_inputs_zero(Composer& composer)
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        make_unused_element_zero(composer, call_context);

        make_unused_array_elements_zero(composer, custom_public_inputs);
        make_unused_array_elements_zero(composer, emitted_public_inputs);

        make_unused_element_zero(composer, executed_callback);

        make_unused_array_elements_zero(composer, output_commitments);
        make_unused_array_elements_zero(composer, input_nullifiers);

        make_unused_array_elements_zero(composer, private_call_stack);
        make_unused_array_elements_zero(composer, public_call_stack);
        make_unused_array_elements_zero(composer, contract_deployment_call_stack);
        make_unused_array_elements_zero(composer, partial_l1_call_stack);
        make_unused_array_elements_zero(composer, callback_stack);

        make_unused_element_zero(composer, old_private_data_tree_root);

        make_unused_element_zero(composer, is_fee_payment);
        make_unused_element_zero(composer, pay_fee_from_l1);
        make_unused_element_zero(composer, pay_fee_from_public_l2);
        make_unused_element_zero(composer, called_from_l1);

        all_elements_populated = true;
    }

    template <typename Composer> void set_public(Composer& composer)
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        make_unused_inputs_zero(composer);

        // Optional members are guaranteed to be nonempty from here.

        (*call_context).set_public();

        set_array_public(custom_public_inputs);
        set_array_public(emitted_public_inputs);

        (*executed_callback).set_public();

        set_array_public(output_commitments);
        set_array_public(input_nullifiers);

        set_array_public(private_call_stack);
        set_array_public(public_call_stack);
        set_array_public(contract_deployment_call_stack);
        set_array_public(partial_l1_call_stack);
        set_array_public(callback_stack);

        (*old_private_data_tree_root).set_public();

        fr(*is_fee_payment).set_public();
        fr(*pay_fee_from_l1).set_public();
        fr(*pay_fee_from_public_l2).set_public();
        fr(*called_from_l1).set_public();
    }

    // TODO: can't use designated constructor anymore, so need to copy the to_native_type() function methodology below.
    template <typename Composer>
    PrivateCircuitPublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };
        auto to_circuit_type = [&](auto& e) {
            return e ? std::make_optional((*e).to_circuit_type(composer)) : std::nullopt;
        };

        PrivateCircuitPublicInputs<CircuitTypes<Composer>> pis = {
            to_circuit_type(call_context),

            to_ct(custom_public_inputs),
            to_ct(emitted_public_inputs),

            to_circuit_type(executed_callback),

            to_ct(output_commitments),
            to_ct(input_nullifiers),

            to_ct(private_call_stack),
            to_ct(public_call_stack),
            to_ct(contract_deployment_call_stack),
            to_ct(partial_l1_call_stack),
            map(callback_stack, to_circuit_type),

            to_ct(old_private_data_tree_root),

            to_ct(is_fee_payment),
            to_ct(pay_fee_from_l1),
            to_ct(pay_fee_from_public_l2),
            to_ct(called_from_l1),
        };

        return pis;
    };

    template <typename Composer> PrivateCircuitPublicInputs<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return plonk::stdlib::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(const std::optional<T>& e) {
            return e ? std::make_optional((*e).template to_native_type<Composer>()) : std::nullopt;
        };
        // auto to_native_type = [&]<typename T>(T& e) { return e.to_native_type(); };

        PrivateCircuitPublicInputs<NativeTypes> pis = {
            to_native_type(call_context),

            to_nt(custom_public_inputs),
            to_nt(emitted_public_inputs),

            to_native_type(executed_callback),

            to_nt(output_commitments),
            to_nt(input_nullifiers),

            to_nt(private_call_stack),
            to_nt(public_call_stack),
            to_nt(contract_deployment_call_stack),
            to_nt(partial_l1_call_stack),
            map(callback_stack, to_native_type),

            to_nt(old_private_data_tree_root),

            to_nt(is_fee_payment),
            to_nt(pay_fee_from_l1),
            to_nt(pay_fee_from_public_l2),
            to_nt(called_from_l1),
        };

        return pis;
    };

    fr hash() const
    {
        auto to_hashes = []<typename T>(const std::optional<T>& e) {
            if (!e) {
                throw_or_abort("Value is nullopt");
            }
            return (*e).hash();
        };

        std::vector<fr> inputs;

        inputs.push_back((*call_context).hash());

        spread_arr_opt_into_vec(custom_public_inputs, inputs);
        spread_arr_opt_into_vec(emitted_public_inputs, inputs);

        inputs.push_back((*executed_callback).hash());

        spread_arr_opt_into_vec(output_commitments, inputs);
        spread_arr_opt_into_vec(input_nullifiers, inputs);

        spread_arr_opt_into_vec(private_call_stack, inputs);
        spread_arr_opt_into_vec(public_call_stack, inputs);
        spread_arr_opt_into_vec(contract_deployment_call_stack, inputs);
        spread_arr_opt_into_vec(partial_l1_call_stack, inputs);
        spread_arr_into_vec(map(callback_stack, to_hashes), inputs);

        inputs.push_back(*old_private_data_tree_root);

        inputs.push_back(fr(*is_fee_payment));
        inputs.push_back(fr(*pay_fee_from_l1));
        inputs.push_back(fr(*pay_fee_from_public_l2));
        inputs.push_back(fr(*called_from_l1));

        return NCT::compress(inputs, GeneratorIndex::PRIVATE_CIRCUIT_PUBLIC_INPUTS);
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

    /// TODO: unused?
    // template <typename Composer> bool check_all_elements_populated()
    // {
    //     static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

    //     bool check_if_populated = [&]<typename T>(std::optional<T>& e) {
    //         if (!e) {
    //             return false;
    //         }
    //         return true;
    //     };

    //     bool still_populated = true;

    //     // call_context is not optional.

    //     still_populated &= std::all_of(custom_public_inputs.begin(), custom_public_inputs.end(), check_if_populated);
    //     still_populated &= std::all_of(emitted_public_inputs.begin(), emitted_public_inputs.end(),
    //     check_if_populated);

    //     // executed_callback.make_unused_inputs_zero(composer);

    //     still_populated &= std::all_of(output_commitments.begin(), output_commitments.end(), check_if_populated);
    //     still_populated &= std::all_of(input_nullifiers.begin(), input_nullifiers.end(), check_if_populated);

    //     still_populated &= std::all_of(private_call_stack.begin(), private_call_stack.end(), check_if_populated);
    //     still_populated &= std::all_of(public_call_stack.begin(), public_call_stack.end(), check_if_populated);
    //     still_populated &= std::all_of(
    //         contract_deployment_call_stack.begin(), contract_deployment_call_stack.end(), check_if_populated);
    //     still_populated &= std::all_of(partial_l1_call_stack.begin(), partial_l1_call_stack.end(),
    //     check_if_populated); still_populated &= std::all_of(callback_stack.begin(), callback_stack.end(),
    //     check_if_populated);

    //     still_populated &= check_if_populated(old_private_data_tree_root);
    //     still_populated &= check_if_populated(is_fee_payment);
    //     still_populated &= check_if_populated(pay_fee_from_l1);
    //     still_populated &= check_if_populated(pay_fee_from_public_l2);
    //     still_populated &= check_if_populated(called_from_l1);

    //     all_elements_populated = still_populated;

    //     return all_elements_populated;
    // }

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

    template <typename Composer>
    void make_unused_element_zero(Composer& composer, std::optional<CallbackStackItem<CircuitTypes<Composer>>>& element)
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        if (!element) {
            element = CallbackStackItem<NativeTypes>::empty().to_circuit_type(
                composer); // convert the nullopt value to a circuit witness value of `0`
            (*element).template assert_is_zero<Composer>();
        }
    }

    template <typename Composer>
    void make_unused_element_zero(Composer& composer, std::optional<CallContext<CircuitTypes<Composer>>>& element)
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        if (!element) {
            element = CallContext<NativeTypes>::empty().to_circuit_type(composer);
            (*element).template assert_is_zero<Composer>();
        }
    }

    template <typename Composer>
    void make_unused_element_zero(Composer& composer, std::optional<ExecutedCallback<CircuitTypes<Composer>>>& element)
    {
        static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

        if (!element) {
            element = ExecutedCallback<NativeTypes>::empty().to_circuit_type(composer);
            (*element).template assert_is_zero<Composer>();
        }
    }

    //     template <typename Composer>
    // void make_unused_element_zero(Composer& composer, std::optional<CallContext<CircuitTypes<Composer>>>& element)
    // {
    //     static_assert((std::is_same<CircuitTypes<Composer>, NCT>::value));

    //     if (!element) {
    //         element = CallContext<NativeTypes>::empty().to_circuit_type(composer);
    //         (*element).template assert_is_zero<Composer>();
    //     }
    // }

    // Make sure this is only called by functions which have implemented a "CT only" check.
    template <typename T, size_t SIZE> void set_array_public(std::array<std::optional<T>, SIZE>& arr)
    {
        for (std::optional<T>& e : arr) {
            fr(*e).set_public();
        }
    }

    template <size_t SIZE, typename Composer>
    void set_array_public(std::array<std::optional<CallbackStackItem<CircuitTypes<Composer>>>, SIZE>& arr)
    {
        for (auto& e : arr) {
            (*e).template set_public<Composer>();
        }
    }
}; // namespace aztec3::circuits::abis

template <typename NCT> void read(uint8_t const*& it, PrivateCircuitPublicInputs<NCT>& private_circuit_public_inputs)
{
    using serialize::read;

    PrivateCircuitPublicInputs<NCT>& pis = private_circuit_public_inputs;
    read(it, pis.call_context);
    read(it, pis.custom_public_inputs);
    read(it, pis.emitted_public_inputs);
    read(it, pis.executed_callback);
    read(it, pis.output_commitments);
    read(it, pis.input_nullifiers);
    read(it, pis.private_call_stack);
    read(it, pis.public_call_stack);
    read(it, pis.contract_deployment_call_stack);
    read(it, pis.partial_l1_call_stack);
    read(it, pis.callback_stack);
    read(it, pis.old_private_data_tree_root);
    read(it, pis.is_fee_payment);
    read(it, pis.pay_fee_from_l1);
    read(it, pis.pay_fee_from_public_l2);
    read(it, pis.called_from_l1);
};

template <typename NCT>
void write(std::vector<uint8_t>& buf, PrivateCircuitPublicInputs<NCT> const& private_circuit_public_inputs)
{
    using serialize::write;

    PrivateCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;

    write(buf, pis.call_context);
    write(buf, pis.custom_public_inputs);
    write(buf, pis.emitted_public_inputs);
    write(buf, pis.executed_callback);
    write(buf, pis.output_commitments);
    write(buf, pis.input_nullifiers);
    write(buf, pis.private_call_stack);
    write(buf, pis.public_call_stack);
    write(buf, pis.contract_deployment_call_stack);
    write(buf, pis.partial_l1_call_stack);
    write(buf, pis.callback_stack);
    write(buf, pis.old_private_data_tree_root);
    write(buf, pis.is_fee_payment);
    write(buf, pis.pay_fee_from_l1);
    write(buf, pis.pay_fee_from_public_l2);
    write(buf, pis.called_from_l1);
};

template <typename NCT>
std::ostream& operator<<(std::ostream& os, PrivateCircuitPublicInputs<NCT> const& private_circuit_public_inputs)

{
    PrivateCircuitPublicInputs<NCT> const& pis = private_circuit_public_inputs;
    return os << "call_context: " << pis.call_context << "\n"
              << "custom_public_inputs: " << pis.custom_public_inputs << "\n"
              << "emitted_public_inputs: " << pis.emitted_public_inputs << "\n"
              << "executed_callback: " << pis.executed_callback << "\n"
              << "output_commitments: " << pis.output_commitments << "\n"
              << "input_nullifiers: " << pis.input_nullifiers << "\n"
              << "private_call_stack: " << pis.private_call_stack << "\n"
              << "public_call_stack: " << pis.public_call_stack << "\n"
              << "contract_deployment_call_stack: " << pis.contract_deployment_call_stack << "\n"
              << "partial_l1_call_stack: " << pis.partial_l1_call_stack << "\n"
              << "callbck_stack: " << pis.callback_stack << "\n"
              << "old_private_data_tree_root: " << pis.old_private_data_tree_root << "\n"
              << "is_fee_payment: " << pis.is_fee_payment << "\n"
              << "pay_fee_from_l1: " << pis.pay_fee_from_l1 << "\n"
              << "pay_fee_from_public_l2: " << pis.pay_fee_from_public_l2 << "\n"
              << "called_from_l1: " << pis.called_from_l1 << "\n";
}

} // namespace aztec3::circuits::abis