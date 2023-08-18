#pragma once

#include "call_context.hpp"
#include "contract_deployment_data.hpp"

#include "aztec3/circuits/abis/historic_block_data.hpp"
#include "aztec3/constants.hpp"
#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include "barretenberg/common/serialize.hpp"
#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using plonk::stdlib::witness_t;

template <typename NCT> class PrivateCircuitPublicInputs {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

  public:
    CallContext<NCT> call_context{};

    fr args_hash = 0;
    std::array<fr, RETURN_VALUES_LENGTH> return_values{};

    std::array<fr, MAX_READ_REQUESTS_PER_CALL> read_requests{};

    std::array<fr, MAX_NEW_COMMITMENTS_PER_CALL> new_commitments{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_CALL> new_nullifiers{};
    std::array<fr, MAX_NEW_NULLIFIERS_PER_CALL> nullified_commitments{};

    std::array<fr, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL> private_call_stack{};
    std::array<fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> public_call_stack{};
    std::array<fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> new_l2_to_l1_msgs{};

    std::array<fr, NUM_FIELDS_PER_SHA256> encrypted_logs_hash{};
    std::array<fr, NUM_FIELDS_PER_SHA256> unencrypted_logs_hash{};

    // Here so that the gas cost of this request can be measured by circuits, without actually needing to feed in the
    // variable-length data.
    fr encrypted_log_preimages_length = 0;
    fr unencrypted_log_preimages_length = 0;

    HistoricBlockData<NCT> historic_block_data{};

    ContractDeploymentData<NCT> contract_deployment_data{};

    fr chain_id = 0;
    fr version = 0;

    // For serialization, update with new fields
    MSGPACK_FIELDS(call_context,
                   args_hash,
                   return_values,
                   read_requests,
                   new_commitments,
                   new_nullifiers,
                   nullified_commitments,
                   private_call_stack,
                   public_call_stack,
                   new_l2_to_l1_msgs,
                   encrypted_logs_hash,
                   unencrypted_logs_hash,
                   encrypted_log_preimages_length,
                   unencrypted_log_preimages_length,
                   historic_block_data,
                   contract_deployment_data,
                   chain_id,
                   version);

    boolean operator==(PrivateCircuitPublicInputs<NCT> const& other) const
    {
        return call_context == other.call_context && args_hash == other.args_hash &&
               return_values == other.return_values && read_requests == other.read_requests &&
               new_commitments == other.new_commitments && new_nullifiers == other.new_nullifiers &&
               nullified_commitments == other.nullified_commitments && private_call_stack == other.private_call_stack &&
               public_call_stack == other.public_call_stack && new_l2_to_l1_msgs == other.new_l2_to_l1_msgs &&
               encrypted_logs_hash == other.encrypted_logs_hash &&
               unencrypted_logs_hash == other.unencrypted_logs_hash &&
               encrypted_log_preimages_length == other.encrypted_log_preimages_length &&
               unencrypted_log_preimages_length == other.unencrypted_log_preimages_length &&
               historic_block_data == other.historic_block_data &&
               contract_deployment_data == other.contract_deployment_data && chain_id == other.chain_id &&
               version == other.version;
    };

    template <typename Builder>
    PrivateCircuitPublicInputs<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
        auto to_circuit_type = [&](auto& e) { return e.to_circuit_type(builder); };

        PrivateCircuitPublicInputs<CircuitTypes<Builder>> pis = {
            to_circuit_type(call_context),

            to_ct(args_hash),
            to_ct(return_values),

            to_ct(read_requests),

            to_ct(new_commitments),
            to_ct(new_nullifiers),
            to_ct(nullified_commitments),

            to_ct(private_call_stack),
            to_ct(public_call_stack),
            to_ct(new_l2_to_l1_msgs),

            to_ct(encrypted_logs_hash),
            to_ct(unencrypted_logs_hash),

            to_ct(encrypted_log_preimages_length),
            to_ct(unencrypted_log_preimages_length),

            to_circuit_type(historic_block_data),

            to_circuit_type(contract_deployment_data),

            to_ct(chain_id),
            to_ct(version),
        };

        return pis;
    };

    template <typename Builder> PrivateCircuitPublicInputs<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        PrivateCircuitPublicInputs<NativeTypes> pis = {
            to_native_type(call_context),

            to_nt(args_hash),
            to_nt(return_values),

            to_nt(read_requests),

            to_nt(new_commitments),
            to_nt(new_nullifiers),
            to_nt(nullified_commitments),

            to_nt(private_call_stack),
            to_nt(public_call_stack),
            to_nt(new_l2_to_l1_msgs),

            to_nt(encrypted_logs_hash),
            to_nt(unencrypted_logs_hash),

            to_nt(encrypted_log_preimages_length),
            to_nt(unencrypted_log_preimages_length),

            to_native_type(historic_block_data),

            to_native_type(contract_deployment_data),

            to_nt(chain_id),
            to_nt(version),
        };

        return pis;
    };

    fr hash() const
    {
        // auto to_hashes = []<typename T>(const T& e) { return e.hash(); };

        std::vector<fr> inputs;

        inputs.push_back(call_context.hash());

        inputs.push_back(args_hash);
        spread_arr_into_vec(return_values, inputs);

        spread_arr_into_vec(read_requests, inputs);

        spread_arr_into_vec(new_commitments, inputs);
        spread_arr_into_vec(new_nullifiers, inputs);
        spread_arr_into_vec(nullified_commitments, inputs);

        spread_arr_into_vec(private_call_stack, inputs);
        spread_arr_into_vec(public_call_stack, inputs);
        spread_arr_into_vec(new_l2_to_l1_msgs, inputs);

        spread_arr_into_vec(encrypted_logs_hash, inputs);
        spread_arr_into_vec(unencrypted_logs_hash, inputs);

        inputs.push_back(encrypted_log_preimages_length);
        inputs.push_back(unencrypted_log_preimages_length);

        spread_arr_into_vec(historic_block_data.to_array(), inputs);

        inputs.push_back(contract_deployment_data.hash());

        inputs.push_back(chain_id);
        inputs.push_back(version);

        if (inputs.size() != PRIVATE_CIRCUIT_PUBLIC_INPUTS_HASH_INPUT_LENGTH) {
            throw_or_abort("Incorrect number of input fields when hashing PrivateCircuitPublicInputs");
        }
        return NCT::hash(inputs, GeneratorIndex::PRIVATE_CIRCUIT_PUBLIC_INPUTS);
    }

    template <size_t SIZE> void spread_arr_into_vec(std::array<fr, SIZE> const& arr, std::vector<fr>& vec) const
    {
        const auto arr_size = sizeof(arr) / sizeof(fr);
        vec.insert(vec.end(), arr.data(), arr.data() + arr_size);
    }
};

// It's been extremely useful for all members here to be std::optional. It allows test app circuits to be very
// quickly drafted without worrying about any of the public inputs which aren't relevant to that circuit. Any values
// which aren't set by the circuit can then be safely set to zero when calling `set_public` (by checking for
// std::nullopt)
template <typename NCT> class OptionalPrivateCircuitPublicInputs {
    using fr = typename NCT::fr;
    using opt_fr = typename std::optional<fr>;

  public:
    std::optional<CallContext<NCT>> call_context;

    opt_fr args_hash;
    std::array<opt_fr, RETURN_VALUES_LENGTH> return_values;

    std::array<opt_fr, MAX_READ_REQUESTS_PER_CALL> read_requests;

    std::array<opt_fr, MAX_NEW_COMMITMENTS_PER_CALL> new_commitments;
    std::array<opt_fr, MAX_NEW_NULLIFIERS_PER_CALL> new_nullifiers;
    std::array<opt_fr, MAX_NEW_NULLIFIERS_PER_CALL> nullified_commitments;

    std::array<opt_fr, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL> private_call_stack;
    std::array<opt_fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> public_call_stack;
    std::array<opt_fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> new_l2_to_l1_msgs;

    std::array<opt_fr, NUM_FIELDS_PER_SHA256> encrypted_logs_hash;
    std::array<opt_fr, NUM_FIELDS_PER_SHA256> unencrypted_logs_hash;

    opt_fr encrypted_log_preimages_length;
    opt_fr unencrypted_log_preimages_length;

    std::optional<HistoricBlockData<NCT>> historic_block_data;

    std::optional<ContractDeploymentData<NCT>> contract_deployment_data;

    opt_fr chain_id;
    opt_fr version;

    // For serialization, update with new fields
    MSGPACK_FIELDS(call_context,
                   args_hash,
                   return_values,
                   read_requests,
                   new_commitments,
                   new_nullifiers,
                   nullified_commitments,
                   private_call_stack,
                   public_call_stack,
                   new_l2_to_l1_msgs,
                   encrypted_logs_hash,
                   unencrypted_logs_hash,
                   encrypted_log_preimages_length,
                   unencrypted_log_preimages_length,
                   historic_block_data,
                   contract_deployment_data,
                   chain_id,
                   version);

    OptionalPrivateCircuitPublicInputs<NCT>() = default;

    OptionalPrivateCircuitPublicInputs<NCT>(
        std::optional<CallContext<NCT>> const& call_context,

        opt_fr const& args_hash,
        std::array<opt_fr, RETURN_VALUES_LENGTH> const& return_values,

        std::array<opt_fr, MAX_READ_REQUESTS_PER_CALL> const& read_requests,

        std::array<opt_fr, MAX_NEW_COMMITMENTS_PER_CALL> const& new_commitments,
        std::array<opt_fr, MAX_NEW_NULLIFIERS_PER_CALL> const& new_nullifiers,
        std::array<opt_fr, MAX_NEW_NULLIFIERS_PER_CALL> const& nullified_commitments,

        std::array<opt_fr, MAX_PRIVATE_CALL_STACK_LENGTH_PER_CALL> const& private_call_stack,
        std::array<opt_fr, MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL> const& public_call_stack,
        std::array<opt_fr, MAX_NEW_L2_TO_L1_MSGS_PER_CALL> const& new_l2_to_l1_msgs,

        std::array<opt_fr, NUM_FIELDS_PER_SHA256> const& encrypted_logs_hash,
        std::array<opt_fr, NUM_FIELDS_PER_SHA256> const& unencrypted_logs_hash,

        opt_fr const& encrypted_log_preimages_length,
        opt_fr const& unencrypted_log_preimages_length,

        std::optional<HistoricBlockData<NCT>> const& historic_block_data,

        std::optional<ContractDeploymentData<NCT>> const& contract_deployment_data,

        opt_fr const& chain_id,
        opt_fr const& version)
        : call_context(call_context)
        , args_hash(args_hash)
        , return_values(return_values)
        , read_requests(read_requests)
        , new_commitments(new_commitments)
        , new_nullifiers(new_nullifiers)
        , nullified_commitments(nullified_commitments)
        , private_call_stack(private_call_stack)
        , public_call_stack(public_call_stack)
        , new_l2_to_l1_msgs(new_l2_to_l1_msgs)
        , encrypted_logs_hash(encrypted_logs_hash)
        , unencrypted_logs_hash(unencrypted_logs_hash)
        , encrypted_log_preimages_length(encrypted_log_preimages_length)
        , unencrypted_log_preimages_length(unencrypted_log_preimages_length)
        , historic_block_data(historic_block_data)
        , contract_deployment_data(contract_deployment_data)
        , chain_id(chain_id)
        , version(version){};

    bool operator==(OptionalPrivateCircuitPublicInputs<NCT> const&) const = default;

    static OptionalPrivateCircuitPublicInputs<NCT> create()
    {
        auto new_inputs = OptionalPrivateCircuitPublicInputs<NCT>();

        new_inputs.call_context = std::nullopt;

        new_inputs.args_hash = std::nullopt;
        new_inputs.return_values.fill(std::nullopt);

        new_inputs.read_requests.fill(std::nullopt);

        new_inputs.new_commitments.fill(std::nullopt);
        new_inputs.new_nullifiers.fill(std::nullopt);
        new_inputs.nullified_commitments.fill(std::nullopt);

        new_inputs.private_call_stack.fill(std::nullopt);
        new_inputs.public_call_stack.fill(std::nullopt);
        new_inputs.new_l2_to_l1_msgs.fill(std::nullopt);

        new_inputs.encrypted_logs_hash.fill(std::nullopt);
        new_inputs.unencrypted_logs_hash.fill(std::nullopt);

        new_inputs.encrypted_log_preimages_length = std::nullopt;
        new_inputs.unencrypted_log_preimages_length = std::nullopt;

        new_inputs.historic_block_data = std::nullopt;

        new_inputs.contract_deployment_data = std::nullopt;

        new_inputs.chain_id = std::nullopt;
        new_inputs.version = std::nullopt;

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

    void set_nullified_commitments(std::vector<fr> input_nullified_commitments)
    {
        if (input_nullified_commitments.size() > nullified_commitments.size()) {
            throw_or_abort("Too many commitments nullified for the number supported by the public inputs ABI.");
        }
        for (size_t i = 0; i < input_nullified_commitments.size(); ++i) {
            nullified_commitments[i] = input_nullified_commitments[i];
        }
    }

    template <typename Builder> void make_unused_inputs_zero(Builder& builder)
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        make_unused_element_zero(builder, call_context);

        make_unused_element_zero(builder, args_hash);
        make_unused_array_elements_zero(builder, return_values);

        make_unused_array_elements_zero(builder, read_requests);

        make_unused_array_elements_zero(builder, new_commitments);
        make_unused_array_elements_zero(builder, new_nullifiers);
        make_unused_array_elements_zero(builder, nullified_commitments);

        make_unused_array_elements_zero(builder, private_call_stack);
        make_unused_array_elements_zero(builder, public_call_stack);
        make_unused_array_elements_zero(builder, new_l2_to_l1_msgs);

        make_unused_array_elements_zero(builder, encrypted_logs_hash);
        make_unused_array_elements_zero(builder, unencrypted_logs_hash);

        make_unused_element_zero(builder, encrypted_log_preimages_length);
        make_unused_element_zero(builder, unencrypted_log_preimages_length);

        make_unused_element_zero(builder, historic_block_data);

        make_unused_element_zero(builder, contract_deployment_data);

        make_unused_element_zero(builder, chain_id);
        make_unused_element_zero(builder, version);

        all_elements_populated = true;
    }

    template <typename Builder> void set_public(Builder& builder)
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        make_unused_inputs_zero(builder);

        // Optional members are guaranteed to be nonempty from here.

        (*call_context).set_public();

        (*args_hash).set_public();
        set_array_public(return_values);

        set_array_public(read_requests);

        set_array_public(new_commitments);
        set_array_public(new_nullifiers);
        set_array_public(nullified_commitments);

        set_array_public(private_call_stack);
        set_array_public(public_call_stack);
        set_array_public(new_l2_to_l1_msgs);

        set_array_public(encrypted_logs_hash);
        set_array_public(unencrypted_logs_hash);

        (*encrypted_log_preimages_length).set_public();
        (*unencrypted_log_preimages_length).set_public();

        (*historic_block_data).set_public();

        (*contract_deployment_data).set_public();

        (*chain_id).set_public();
        (*version).set_public();
    }

    template <typename Builder>
    OptionalPrivateCircuitPublicInputs<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };
        auto to_circuit_type = [&](auto& e) {
            return e ? std::make_optional((*e).to_circuit_type(builder)) : std::nullopt;
        };

        OptionalPrivateCircuitPublicInputs<CircuitTypes<Builder>> pis = {
            to_circuit_type(call_context),

            to_ct(args_hash),
            to_ct(return_values),

            to_ct(read_requests),

            to_ct(new_commitments),
            to_ct(new_nullifiers),
            to_ct(nullified_commitments),

            to_ct(private_call_stack),
            to_ct(public_call_stack),
            to_ct(new_l2_to_l1_msgs),

            to_ct(encrypted_logs_hash),
            to_ct(unencrypted_logs_hash),

            to_ct(encrypted_log_preimages_length),
            to_ct(unencrypted_log_preimages_length),

            to_circuit_type(historic_block_data),

            to_circuit_type(contract_deployment_data),

            to_ct(chain_id),
            to_ct(version),
        };

        return pis;
    };

    template <typename Builder> OptionalPrivateCircuitPublicInputs<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };
        auto to_native_type = []<typename T>(const std::optional<T>& e) {
            return e ? std::make_optional((*e).template to_native_type<Builder>()) : std::nullopt;
        };
        // auto to_native_type = [&]<typename T>(T& e) { return e.to_native_type(); };

        OptionalPrivateCircuitPublicInputs<NativeTypes> pis = { to_native_type(call_context),

                                                                to_nt(args_hash),
                                                                to_nt(return_values),

                                                                to_nt(read_requests),

                                                                to_nt(new_commitments),
                                                                to_nt(new_nullifiers),
                                                                to_nt(nullified_commitments),

                                                                to_nt(private_call_stack),
                                                                to_nt(public_call_stack),
                                                                to_nt(new_l2_to_l1_msgs),

                                                                to_nt(encrypted_logs_hash),
                                                                to_nt(unencrypted_logs_hash),

                                                                to_nt(encrypted_log_preimages_length),
                                                                to_nt(unencrypted_log_preimages_length),

                                                                to_native_type(historic_block_data),

                                                                to_native_type(contract_deployment_data),

                                                                to_nt(chain_id),
                                                                to_nt(version) };

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

        inputs.push_back(*args_hash);
        spread_arr_opt_into_vec(return_values, inputs);

        spread_arr_opt_into_vec(read_requests, inputs);

        spread_arr_opt_into_vec(new_commitments, inputs);
        spread_arr_opt_into_vec(new_nullifiers, inputs);
        spread_arr_opt_into_vec(nullified_commitments, inputs);

        spread_arr_opt_into_vec(private_call_stack, inputs);
        spread_arr_opt_into_vec(public_call_stack, inputs);
        spread_arr_opt_into_vec(new_l2_to_l1_msgs, inputs);

        spread_arr_into_vec(encrypted_logs_hash, inputs);
        spread_arr_into_vec(unencrypted_logs_hash, inputs);

        inputs.push_back(*encrypted_log_preimages_length);
        inputs.push_back(*unencrypted_log_preimages_length);

        spread_arr_opt_into_vec((*historic_block_data).to_array(), inputs);

        inputs.push_back((*contract_deployment_data).hash());

        inputs.push_back(*chain_id);
        inputs.push_back(*version);

        return NCT::compress(inputs, GeneratorIndex::PRIVATE_CIRCUIT_PUBLIC_INPUTS);
    }

    // We can remove optionality when using the inputs in a kernel or rollup circuit, for ease of use.
    PrivateCircuitPublicInputs<NCT> remove_optionality() const
    {
        auto get_value = [&](auto& e) { return e.value(); };

        return PrivateCircuitPublicInputs<NCT>{
            .call_context = call_context.value(),

            .args_hash = args_hash.value(),
            .return_values = map(return_values, get_value),

            .read_requests = map(read_requests, get_value),

            .new_commitments = map(new_commitments, get_value),
            .new_nullifiers = map(new_nullifiers, get_value),
            .nullified_commitments = map(nullified_commitments, get_value),

            .private_call_stack = map(private_call_stack, get_value),
            .public_call_stack = map(public_call_stack, get_value),
            .new_l2_to_l1_msgs = map(new_l2_to_l1_msgs, get_value),

            .encrypted_logs_hash = map(encrypted_logs_hash, get_value),
            .unencrypted_logs_hash = map(unencrypted_logs_hash, get_value),

            .encrypted_log_preimages_length = encrypted_log_preimages_length.value(),
            .unencrypted_log_preimages_length = unencrypted_log_preimages_length.value(),

            .historic_block_data = historic_block_data.value(),

            .contract_deployment_data = contract_deployment_data.value(),

            .chain_id = chain_id.value(),
            .version = version.value(),
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
        vec.insert(vec.end(), arr_values.data(), arr_values.data() + arr_size);
    }

    template <size_t SIZE> void spread_arr_into_vec(std::array<fr, SIZE> const& arr, std::vector<fr>& vec) const
    {
        const auto arr_size = sizeof(arr) / sizeof(fr);
        vec.insert(vec.end(), arr.data(), arr.data() + arr_size);
    }

    template <typename Builder, typename T, size_t SIZE>
    void make_unused_array_elements_zero(Builder& builder, std::array<std::optional<T>, SIZE>& arr)
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        for (std::optional<T>& e : arr) {
            make_unused_element_zero(builder, e);
        }
    }

    template <typename Builder, typename T> void make_unused_element_zero(Builder& builder, std::optional<T>& element)
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        if (!element) {
            element =
                T(witness_t<Builder>(&builder, 0));  // convert the nullopt value to a circuit witness value of `0`
            fr(*element).assert_is_zero();
        }
    }

    // ABIStruct is a template for any of the structs in the abis/ dir. E.g. ExecutedCallback, CallbackStackItem.
    template <typename Builder, template <class> class ABIStruct>
    void make_unused_element_zero(Builder& builder, std::optional<ABIStruct<CircuitTypes<Builder>>>& element)
    {
        static_assert((std::is_same<CircuitTypes<Builder>, NCT>::value));

        if (!element) {
            element = ABIStruct<NativeTypes>().to_circuit_type(
                builder);  // convert the nullopt value to a circuit witness value of `0`
            (*element).template assert_is_zero<Builder>();
        }
    }

    // Make sure this is only called by functions which have implemented a "CT only" check.
    template <typename T, size_t SIZE> void set_array_public(std::array<std::optional<T>, SIZE>& arr)
    {
        for (std::optional<T>& e : arr) {
            fr(*e).set_public();
        }
    }
};
}  // namespace aztec3::circuits::abis
