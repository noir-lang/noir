// #include <stdlib/primitives/witness/witness.hpp>
// #include <stdlib/types/native_types.hpp>
// #include <stdlib/types/circuit_types.hpp>
// #include <stdlib/types/convert.hpp>
// #include "contract.hpp"
// #include "l1_promise.hpp"
// #include "l1_result.hpp"

// namespace aztec3::circuits::apps {

// using plonk::stdlib::witness_t;
// using plonk::stdlib::types::CircuitTypes;
// using plonk::stdlib::types::NativeTypes;

// template <typename Composer>
// std::pair<L1Promise<Composer>, L1Result> L1FunctionInterface<Composer>::call(std::vector<fr> args)
// {
//     if (args.size() != num_params) {
//         throw_or_abort("Incorrect number of args");
//     }

//     auto promise = L1Promise<Composer>(*contract);
//     L1Result result;
//     return std::make_pair(promise, result);
// }

// } // namespace aztec3::circuits::apps