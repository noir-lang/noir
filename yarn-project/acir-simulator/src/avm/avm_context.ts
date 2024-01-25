import { AvmExecutionEnvironment } from './avm_execution_environment.js';
import { AvmMachineState } from './avm_machine_state.js';
import { AvmMessageCallResult } from './avm_message_call_result.js';
import { AvmInterpreter } from './interpreter/index.js';
import { AvmJournal } from './journal/journal.js';
import { decodeBytecode } from './opcodes/decode_bytecode.js';
import { Instruction } from './opcodes/index.js';

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
  public call(): AvmMessageCallResult {
    // NOTE: the following is mocked as getPublicBytecode does not exist yet
    // const bytecode = journal.journal.hostStorage.contractsDb.getBytecode(this.executionEnvironment.address);
    const bytecode = Buffer.from('0x01000100020003');

    const instructions: Instruction[] = decodeBytecode(bytecode);

    const context = new AvmMachineState(this.executionEnvironment);
    const interpreter = new AvmInterpreter(context, this.journal, instructions);

    return interpreter.run();
  }
}
