// #pragma once
// #include <aztec3/constants.hpp>
// #include <stdlib/types/convert.hpp>
// #include "nullifier_preimage.hpp"
// #include "private_state_note.hpp"
// #include "private_state_var.hpp"
// #include "oracle_wrapper.hpp"

// namespace aztec3::circuits::apps {

// // using plonk::stdlib::witness_t;
// using plonk::stdlib::types::CircuitTypes;
// using NT = plonk::stdlib::types::NativeTypes;

// template <typename Composer> class PrivateStateFactory {
//     typedef CircuitTypes<Composer> CT;
//     typedef typename CT::fr fr;

//   public:
//     Composer& composer; // TODO: can we remove this?
//     OracleWrapperInterface<Composer>& oracle;
//     const std::string contract_name;
//     fr private_state_counter = 0;

//     std::map<std::string, PrivateStateVar<Composer>> private_state_vars;
//     // std::vector<PrivateStateNote<Composer>> new_private_state_notes;
//     // std::vector<fr> new_commitments;
//     // std::vector<NullifierPreimage<CT>> new_nullifier_preimages;
//     // std::vector<fr> new_nullifiers;

// };

// } // namespace aztec3::circuits::apps