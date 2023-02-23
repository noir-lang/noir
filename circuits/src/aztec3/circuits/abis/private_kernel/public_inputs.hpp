#pragma once
#include "accumulated_data.hpp"
#include "constant_data.hpp"
#include <stdlib/primitives/witness/witness.hpp>
#include <stdlib/types/native_types.hpp>
#include <stdlib/types/circuit_types.hpp>
#include <stdlib/types/convert.hpp>

namespace aztec3::circuits::abis::private_kernel {

using plonk::stdlib::witness_t;
using plonk::stdlib::types::CircuitTypes;
using plonk::stdlib::types::NativeTypes;
using std::is_same;

template <typename NCT> struct PublicInputs {
    typedef typename NCT::fr fr;
    typedef typename NCT::boolean boolean;

    AccumulatedData<NCT> end;
    ConstantData<NCT> constants;

    boolean is_private = true; // TODO: might need to instantiate from witness!

    template <typename Composer> PublicInputs<CircuitTypes<Composer>> to_circuit_type(Composer& composer) const
    {
        static_assert((std::is_same<NativeTypes, NCT>::value));

        // Capture the composer:
        auto to_ct = [&](auto& e) { return plonk::stdlib::types::to_ct(composer, e); };

        PublicInputs<CircuitTypes<Composer>> private_inputs = {
            end.to_circuit_type(composer),
            constants.to_circuit_type(composer),

            to_ct(is_private),
        };

        return private_inputs;
    };

    template <typename Composer> PublicInputs<NativeTypes> to_native_type() const
    {
        static_assert(std::is_same<CircuitTypes<Composer>, NCT>::value);
        auto to_nt = [&](auto& e) { return plonk::stdlib::types::to_nt<Composer>(e); };
        auto to_native_type = []<typename T>(T& e) { return e.template to_native_type<Composer>(); };

        PublicInputs<NativeTypes> pis = {
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

} // namespace aztec3::circuits::abis::private_kernel