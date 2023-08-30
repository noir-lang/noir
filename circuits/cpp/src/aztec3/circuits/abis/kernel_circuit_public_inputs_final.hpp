#pragma once
#include "combined_constant_data.hpp"
#include "final_accumulated_data.hpp"

#include "aztec3/utils/types/circuit_types.hpp"
#include "aztec3/utils/types/convert.hpp"
#include "aztec3/utils/types/native_types.hpp"

#include <barretenberg/barretenberg.hpp>

namespace aztec3::circuits::abis {

using aztec3::utils::types::CircuitTypes;
using aztec3::utils::types::NativeTypes;
using std::is_same;

template <typename NCT> struct KernelCircuitPublicInputsFinal {
    using fr = typename NCT::fr;
    using boolean = typename NCT::boolean;

    FinalAccumulatedData<NCT> end{};
    CombinedConstantData<NCT> constants{};

    boolean is_private = true;  // TODO: might need to instantiate from witness!

    // for serialization, update with new fields
    MSGPACK_FIELDS(end, constants, is_private);

    boolean operator==(KernelCircuitPublicInputsFinal<NCT> const& other) const
    {
        return msgpack_derived_equals<boolean>(*this, other);
    }

    template <typename Builder>
    KernelCircuitPublicInputsFinal<CircuitTypes<Builder>> to_circuit_type(Builder& builder) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the circuit builder:
        auto to_ct = [&](auto& e) { return aztec3::utils::types::to_ct(builder, e); };

        KernelCircuitPublicInputsFinal<CircuitTypes<Builder>> private_inputs = {
            end.to_circuit_type(builder),
            constants.to_circuit_type(builder),

            to_ct(is_private),
        };

        return private_inputs;
    };

    template <typename Builder> KernelCircuitPublicInputsFinal<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Builder>, NCT>::value);
        auto to_nt = [&](auto& e) { return aztec3::utils::types::to_nt<Builder>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Builder>(); };

        KernelCircuitPublicInputsFinal<NativeTypes> pis = {
            to_native_type(end),
            to_native_type(constants),

            to_nt(is_private),
        };

        return pis;
    };

    void set_public()
    {
        static_assert(!(std::is_same<NativeTypes, NCT>::value));

        end.set_public();
        constants.set_public();

        fr(is_private).set_public();
    }
};

}  // namespace aztec3::circuits::abis
