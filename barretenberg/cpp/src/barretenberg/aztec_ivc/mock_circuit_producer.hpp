
#include "barretenberg/aztec_ivc/aztec_ivc.hpp"
#include "barretenberg/common/op_count.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"

using namespace bb;

namespace {

/**
 * @brief Test utility for coordinating passing of databus data between mocked private function execution circuits
 * @details Facilitates testing of the databus consistency checks that establish the correct passing of databus data
 * between circuits. Generates arbitrary return data for each app/kernel. Sets the kernel calldata and
 * secondary_calldata based respectively on the previous kernel return data and app return data.
 */
class MockDatabusProducer {
  private:
    using ClientCircuit = AztecIVC::ClientCircuit;
    using Flavor = MegaFlavor;
    using FF = Flavor::FF;
    using BusDataArray = std::vector<FF>;

    static constexpr size_t BUS_ARRAY_SIZE = 3; // arbitrary length of mock bus inputs
    BusDataArray app_return_data;
    BusDataArray kernel_return_data;

    FF dummy_return_val = 1; // use simple return val for easier test debugging

    BusDataArray generate_random_bus_array()
    {
        BusDataArray result;
        for (size_t i = 0; i < BUS_ARRAY_SIZE; ++i) {
            result.emplace_back(dummy_return_val);
        }
        dummy_return_val += 1;
        return result;
    }

  public:
    /**
     * @brief Update the app return data and populate it in the app circuit
     */
    void populate_app_databus(ClientCircuit& circuit)
    {
        app_return_data = generate_random_bus_array();
        for (auto& val : app_return_data) {
            circuit.add_public_return_data(circuit.add_variable(val));
        }
    };

    /**
     * @brief Populate the calldata and secondary calldata in the kernel from respectively the previous kernel and app
     * return data. Update and populate the return data for the present kernel.
     */
    void populate_kernel_databus(ClientCircuit& circuit)
    {
        for (auto& val : kernel_return_data) { // populate calldata from previous kernel return data
            circuit.add_public_calldata(circuit.add_variable(val));
        }
        for (auto& val : app_return_data) { // populate secondary_calldata from app return data
            circuit.add_public_secondary_calldata(circuit.add_variable(val));
        }
        kernel_return_data = generate_random_bus_array(); // update the return data for the present kernel circuit
        for (auto& val : kernel_return_data) {
            circuit.add_public_return_data(circuit.add_variable(val));
        }
    };

    /**
     * @brief Add an arbitrary value to the app return data. This leads to a descrepency between the values used by the
     * app itself and the secondary_calldata values in the kernel that will be set based on these tampered values.
     */
    void tamper_with_app_return_data() { app_return_data.emplace_back(17); }
};

/**
 * @brief Manage the construction of mock app/kernel circuits for the private function execution setting
 * @details Per the medium complexity benchmark spec, the first app circuit is size 2^19. Subsequent app and kernel
 * circuits are size 2^17. Circuits produced are alternatingly app and kernel. Mock databus data is passed between the
 * circuits in a manor conistent with the real architecture in order to facilitate testing of databus consistency
 * checks.
 */
class PrivateFunctionExecutionMockCircuitProducer {
    using ClientCircuit = AztecIVC::ClientCircuit;
    using Flavor = MegaFlavor;
    using VerificationKey = Flavor::VerificationKey;

    size_t circuit_counter = 0;

    MockDatabusProducer mock_databus;

  public:
    /**
     * @brief Create a the next circuit (app/kernel) in a mocked private function execution stack
     */
    ClientCircuit create_next_circuit(AztecIVC& ivc)
    {
        circuit_counter++;

        bool is_kernel = (circuit_counter % 2 == 0); // Every other circuit is a kernel, starting from the second

        ClientCircuit circuit{ ivc.goblin.op_queue };
        if (is_kernel) {
            GoblinMockCircuits::construct_mock_folding_kernel(circuit); // construct mock base logic
            mock_databus.populate_kernel_databus(circuit);              // populate databus inputs/outputs
            ivc.complete_kernel_circuit_logic(circuit);                 // complete with recursive verifiers etc
        } else {
            bool use_large_circuit = (circuit_counter == 1);                            // first circuit is size 2^19
            GoblinMockCircuits::construct_mock_app_circuit(circuit, use_large_circuit); // construct mock app
            mock_databus.populate_app_databus(circuit);                                 // populate databus outputs
        }
        return circuit;
    }

    /**
     * @brief Tamper with databus data to facilitate failure testing
     */
    void tamper_with_databus() { mock_databus.tamper_with_app_return_data(); }

    /**
     * @brief Compute and return the verification keys for a mocked private function execution IVC
     * @details For testing/benchmarking only. This method is robust at the cost of being extremely inefficient. It
     * simply executes a full IVC for a given number of circuits and stores the verification keys along the way. (In
     * practice these VKs will be known to a client prover in advance).
     *
     * @param num_circuits
     * @param trace_structure Trace structuring must be known in advance because it effects the VKs
     * @return set of num_circuits-many verification keys
     */
    auto precompute_verification_keys(const size_t num_circuits, TraceStructure trace_structure)
    {
        AztecIVC ivc; // temporary IVC instance needed to produce the complete kernel circuits
        ivc.trace_structure = trace_structure;

        std::vector<std::shared_ptr<VerificationKey>> vkeys;

        for (size_t idx = 0; idx < num_circuits; ++idx) {
            ClientCircuit circuit = create_next_circuit(ivc); // create the next circuit
            ivc.accumulate(circuit);                          // accumulate the circuit
            vkeys.emplace_back(ivc.instance_vk);              // save the VK for the circuit
        }
        circuit_counter = 0; // reset the internal circuit counter back to 0

        return vkeys;
    }
};

} // namespace