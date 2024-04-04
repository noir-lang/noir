import { type AztecAddress, FunctionSelector } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

import { type AvmExecutionEnvironment } from './avm_execution_environment.js';
import { type Gas, gasToGasLeft } from './avm_gas.js';
import { AvmMachineState } from './avm_machine_state.js';
import { type AvmPersistableStateManager } from './journal/journal.js';

/**
 * An execution context includes the information necessary to initiate AVM
 * execution along with all state maintained by the AVM throughout execution.
 */
export class AvmContext {
  /**
   * Create a new AVM context
   * @param persistableState - Manages world state and accrued substate during execution - (caching, fetching, tracing)
   * @param environment - Contains constant variables provided by the kernel
   * @param machineState - VM state that is modified on an instruction-by-instruction basis
   * @returns new AvmContext instance
   */
  constructor(
    public persistableState: AvmPersistableStateManager,
    public environment: AvmExecutionEnvironment,
    public machineState: AvmMachineState,
  ) {}

  /**
   * Prepare a new AVM context that will be ready for an external/nested call
   * - Fork the world state journal
   * - Derive a machine state from the current state
   *   - E.g., gas metering is preserved but pc is reset
   * - Derive an execution environment from the caller/parent
   *   - Alter both address and storageAddress
   *
   * @param address - The contract instance to initialize a context for
   * @param calldata - Data/arguments for nested call
   * @param allocatedGas - Gas allocated for the nested call
   * @param callType - Type of call (CALL or STATICCALL)
   * @returns new AvmContext instance
   */
  public createNestedContractCallContext(
    address: AztecAddress,
    calldata: Fr[],
    allocatedGas: Gas,
    callType: 'CALL' | 'STATICCALL',
    temporaryFunctionSelector: FunctionSelector = FunctionSelector.empty(),
  ): AvmContext {
    const deriveFn =
      callType === 'CALL'
        ? this.environment.deriveEnvironmentForNestedCall
        : this.environment.deriveEnvironmentForNestedStaticCall;
    const newExecutionEnvironment = deriveFn.call(this.environment, address, calldata, temporaryFunctionSelector);
    const forkedWorldState = this.persistableState.fork();
    const machineState = AvmMachineState.fromState(gasToGasLeft(allocatedGas));
    return new AvmContext(forkedWorldState, newExecutionEnvironment, machineState);
  }
}
