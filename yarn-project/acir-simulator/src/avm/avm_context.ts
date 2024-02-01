import { AztecAddress, FunctionSelector } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import { AvmExecutionEnvironment } from './avm_execution_environment.js';
import { AvmMachineState } from './avm_machine_state.js';
import { AvmMessageCallResult } from './avm_message_call_result.js';
import { AvmInterpreterError, executeAvm } from './interpreter/index.js';
import { AvmJournal } from './journal/journal.js';
import { Instruction } from './opcodes/instruction.js';
import { decodeFromBytecode } from './serialization/bytecode_serialization.js';

// FIXME: dependency cycle.

/**
 * Avm Executor manages the execution of the AVM
 *
 * It stores a state manager
 */
export class AvmContext {
  /** Contains constant variables provided by the kernel */
  private executionEnvironment: AvmExecutionEnvironment;
  /** Manages mutable state during execution - (caching, fetching) */
  private journal: AvmJournal;

  constructor(executionEnvironment: AvmExecutionEnvironment, journal: AvmJournal) {
    this.executionEnvironment = executionEnvironment;
    this.journal = journal;
  }

  /**
   * Call a contract with the given calldata
   *
   * - We get the contract from storage
   * - We interpret the bytecode
   * - We run the interpreter
   *
   */
  async call(): Promise<AvmMessageCallResult> {
    // NOTE: the following is mocked as getPublicBytecode does not exist yet
    const selector = new FunctionSelector(0);
    const bytecode = await this.journal.hostStorage.contractsDb.getBytecode(
      this.executionEnvironment.address,
      selector,
    );

    // This assumes that we will not be able to send messages to accounts without code
    // Pending classes and instances impl details
    if (!bytecode) {
      throw new NoBytecodeFoundInterpreterError(this.executionEnvironment.address);
    }

    const instructions: Instruction[] = decodeFromBytecode(bytecode);

    const machineState = new AvmMachineState(this.executionEnvironment);
    return executeAvm(machineState, this.journal, instructions);
  }

  /**
   * Create a new forked avm context - for internal calls
   */
  public newWithForkedState(): AvmContext {
    const forkedState = AvmJournal.branchParent(this.journal);
    return new AvmContext(this.executionEnvironment, forkedState);
  }

  /**
   * Create a new forked avm context - for external calls
   */
  public static newWithForkedState(executionEnvironment: AvmExecutionEnvironment, journal: AvmJournal): AvmContext {
    const forkedState = AvmJournal.branchParent(journal);
    return new AvmContext(executionEnvironment, forkedState);
  }

  /**
   * Prepare a new AVM context that will be ready for an external call
   * - It will fork the journal
   * - It will set the correct execution Environment Variables for a call
   *    - Alter both address and storageAddress
   *
   * @param address - The contract to call
   * @param executionEnvironment - The current execution environment
   * @param journal - The current journal
   * @returns new AvmContext instance
   */
  public static prepExternalCallContext(
    address: AztecAddress,
    calldata: Fr[],
    executionEnvironment: AvmExecutionEnvironment,
    journal: AvmJournal,
  ): AvmContext {
    const newExecutionEnvironment = executionEnvironment.newCall(address, calldata);
    const forkedState = AvmJournal.branchParent(journal);
    return new AvmContext(newExecutionEnvironment, forkedState);
  }

  /**
   * Prepare a new AVM context that will be ready for an external static call
   * - It will fork the journal
   * - It will set the correct execution Environment Variables for a call
   *    - Alter both address and storageAddress
   *
   * @param address - The contract to call
   * @param executionEnvironment - The current execution environment
   * @param journal - The current journal
   * @returns new AvmContext instance
   */
  public static prepExternalStaticCallContext(
    address: AztecAddress,
    calldata: Fr[],
    executionEnvironment: AvmExecutionEnvironment,
    journal: AvmJournal,
  ): AvmContext {
    const newExecutionEnvironment = executionEnvironment.newStaticCall(address, calldata);
    const forkedState = AvmJournal.branchParent(journal);
    return new AvmContext(newExecutionEnvironment, forkedState);
  }

  /**
   * Merge the journal of this call with it's parent
   * NOTE: this should never be called on a root context - only from within a nested call
   */
  public mergeJournalSuccess() {
    this.journal.mergeSuccessWithParent();
  }

  /**
   * Merge the journal of this call with it's parent
   * For when the child call fails ( we still must track state accesses )
   */
  public mergeJournalFailure() {
    this.journal.mergeFailureWithParent();
  }
}

class NoBytecodeFoundInterpreterError extends AvmInterpreterError {
  constructor(contractAddress: AztecAddress) {
    super(`No bytecode found at: ${contractAddress}`);
    this.name = 'NoBytecodeFoundInterpreterError';
  }
}
