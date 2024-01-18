import { Fr } from '@aztec/foundation/fields';

import { AvmMachineState } from './avm_machine_state.js';
import { AvmMessageCallResult } from './avm_message_call_result.js';
import { AvmStateManager } from './avm_state_manager.js';
import { AvmInterpreter } from './interpreter/index.js';
import { decodeBytecode } from './opcodes/decode_bytecode.js';
import { Instruction } from './opcodes/index.js';

/**
 * Avm Executor manages the execution of the AVM
 *
 * It stores a state manager
 */
export class AvmContext {
  private stateManager: AvmStateManager;

  constructor(stateManager: AvmStateManager) {
    this.stateManager = stateManager;
  }

  /**
   * Call a contract with the given calldata
   *
   * - We get the contract from storage
   * - We interpret the bytecode
   * - We run the interpreter
   *
   * @param contractAddress -
   * @param calldata -
   */
  public call(contractAddress: Fr, calldata: Fr[]): AvmMessageCallResult {
    // NOTE: the following is mocked as getPublicBytecode does not exist yet
    // const bytecode = stateManager.journal.hostStorage.contractsDb.getBytecode(contractAddress);
    const bytecode = Buffer.from('0x01000100020003');

    const instructions: Instruction[] = decodeBytecode(bytecode);

    const context = new AvmMachineState(calldata);
    const interpreter = new AvmInterpreter(context, this.stateManager, instructions);

    return interpreter.run();
  }
}
